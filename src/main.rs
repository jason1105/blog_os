#![no_std] // 不使用 Rust 标准库
#![no_main] // 不使用 main 函数

use core::panic::PanicInfo;
use core::fmt::Write;
mod vga_buffer;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}


static HELLO: &[u8] = b"Hello World!";

#[no_mangle] // 不要重命名函数的名称
pub extern "C" fn _start() -> ! {
    //vga_buffer::print_something();
    //vga_buffer::WRITER.lock().write_byte(b'H');
    //vga_buffer::WRITER.lock().write_string("ello!\n");
    //write!(vga_buffer::WRITER.lock(), "welcome to {}", "Handan").unwrap();
    //println!("2022-01-04");
    println!("Hello world!");
    panic!("Some panic message.");

    loop {}
}
