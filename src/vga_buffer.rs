use volatile::Volatile;
use core::fmt::{Error, Write};
use core::*;
use lazy_static::lazy_static;
use spin::Mutex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>;BUFFER_WIDTH];BUFFER_HEIGHT]
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer // vga buffer
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                
                let ascii_character = byte;
                let color_code = self.color_code;
                
                // call method write() of variable of volatile instead of equal sign (=).
                self.buffer.chars[BUFFER_HEIGHT-1][self.column_position].write( ScreenChar {
                    ascii_character,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }
    
    pub fn write_string(&mut self, string: &str) {
        for byte in string.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }
    
    fn new_line(&mut self) {
        // move displayed character one line up
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row-1][col].write(self.buffer.chars[row][col].read());
            }
        }
        
        self.clear_row();
        
        // carriage return
        self.column_position = 0;
    }
    
    fn clear_row(&mut self) {    
        // clear last line
        for col in 0..BUFFER_WIDTH {
            let ascii_character = b' ';
            let color_code = self.color_code;
            self.buffer.chars[BUFFER_HEIGHT-1][col].write(ScreenChar{ascii_character, color_code});
        }
    }
}

// We can write utf-8-encoded data to Writer, as implement core::fmt::Write trait. 
impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::Yellow, Color::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

pub fn print_something() {
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello!\n");
    // We format string and write it into variable writer that has implemented core::fmt::Write trait.
    write!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap();
}


#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}