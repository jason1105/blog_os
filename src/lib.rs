#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"] // Rust 生成一个 main 方法调用 test_runner, 但我们的程序不使用 main 函数, 所以只能在 _start 中调用 main 函数, 但 main 函数是被系统调用的, 这里把 main 改名为 test_main, 在 test_main 中调用 test_runner, 但程序的入口还是 _start, 所以要在 _start 中调用 test_main
#![feature(abi_x86_interrupt)] // 开启x86-interrupt calling convention, 因为is still unstable
#![feature(alloc_error_handler)]

#![allow(warnings)]

use core::panic::PanicInfo;
extern crate alloc; // new

pub mod vga_buffer; // 其中标记有 #[test_case] 的 module 都会被测试
pub mod serial; // 其中标记有 #[test_case] 的 module 都会被测试
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator; // new


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where T: Fn()
{
    fn run(&self) -> (){
        serial_print!("{}... \t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

// #[cfg(test)] // we should call this from main.rs or tests folder.
pub fn test_runner(tests: &[&dyn Testable]) { // slice of trait object
    serial_println!("Running {} tests.", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn testfn() {
    
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}


pub fn init() { // init os
    interrupts::init_idt();
    gdt::init();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop{
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
// #[no_mangle] // 不要重命名函数的名称
// pub extern "C" fn _start() -> ! { // 所有 library crate 中的单元测试入口
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    
    serial_println!("Start unittests for lib.");

    test_main();
    
    hlt_loop();
}

#[cfg(test)]
#[panic_handler] // conditional compilation
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("{}", _info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}








