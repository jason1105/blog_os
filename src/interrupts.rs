

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use lazy_static::lazy_static;
use crate::println;
use crate::print;
use crate::gdt;
use pic8259::ChainedPics;
use spin;

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt_temp = InterruptDescriptorTable::new();
        
        // 在 IDT 中注册 handler 
        idt_temp.breakpoint.set_handler_fn(breakpoint_handler);
        idt_temp.page_fault.set_handler_fn(pagefault_handler);
        unsafe {
            idt_temp.double_fault.set_handler_fn(doublefault_handler).set_stack_index(gdt::DOUBLE_FAULT_STACK_IST_INDEX);
        }
        idt_temp[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt_temp[InterruptIndex::KeyBoard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt_temp
    };
}

extern "x86-interrupt" fn breakpoint_handler (stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn pagefault_handler (stack_frame: InterruptStackFrame, _error_code: PageFaultErrorCode) {
    
    use x86_64::registers::control::Cr2;
    use crate::hlt_loop;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", _error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

// double fault 的 handler
extern "x86-interrupt" fn doublefault_handler (
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// PIC
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    KeyBoard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

// Our timer_interrupt_handler has the same signature as our exception handlers, because the CPU reacts identically to exceptions and external interrupts
extern "x86-interrupt" fn timer_interrupt_handler (_stack_frame: InterruptStackFrame) {
    print!(".");
    
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler (_stack_frame: InterruptStackFrame) {
    
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    
    lazy_static! {
       static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }
    
    let mut port = Port::<u8>::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    let mut keyboard = KEYBOARD.lock();
    
    if let Ok(Some(key_eveny)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_eveny) {
            match key {
                DecodedKey::RawKey(key) => print!("{:?}", key),
                DecodedKey::Unicode(character) => print!("{}", character),
            }
        }
    }
    
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::KeyBoard.as_u8());
    }
}

pub fn init_idt() {
    IDT.load();
}

#[test_case]
fn test_interrupt_idt() {
    x86_64::instructions::interrupts::int3();
}