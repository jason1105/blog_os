#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"] // Rust 生成一个 main 方法调用 test_runner, 但我们的程序不使用 main 函数, 所以只能在 _start 中调用 main 函数, 但 main 函数是被系统调用的, 这里把 main 改名为 test_main, 在 test_main 中调用 test_runner, 但程序的入口还是 _start, 所以要在 _start 中调用 test_main

use core::panic::PanicInfo;

pub mod vga_buffer; // 其中标记有 #[test_case] 的 module 都会被测试
pub mod serial; // 其中标记有 #[test_case] 的 module 都会被测试

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
    serial_println!("Running {} tests ------ in lib", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

#[cfg(test)]
#[no_mangle] // 不要重命名函数的名称
pub extern "C" fn _start() -> ! { // 所有 library crate 中的单元测试入口

    serial_println!("Start unittests for lib.");

    test_main();
    
    loop {}
}

#[cfg(test)]
#[panic_handler] // conditional compilation
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("{}", _info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}








