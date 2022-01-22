#![no_std] // 不使用 Rust 标准库
#![no_main] // 不使用 main 函数
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"] // Rust 生成一个 main 方法调用 test_runner, 但我们的程序不使用 main 函数, 所以只能在 _start 中调用 main 函数, 但 main 函数是被系统调用的, 这里把 main 改名为 test_main, 在 test_main 中调用 test_runner, 但程序的入口还是 _start, 所以要在 _start 中调用 test_main

#![allow(warnings)]

use core::panic::PanicInfo;
use blog_os::println; // 引用宏不用包含 module 名称
#[cfg(test)]
use blog_os::serial_println; // 引用宏不用包含 module 名称

    
/// This function is called on panic.
#[cfg(not(test))] // conditional compilation
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler] // conditional compilation
fn panic(_info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(_info);
    // loop {}
}

#[no_mangle] // 不要重命名函数的名称
pub extern "C" fn _start() -> ! { // cargo run 和 cargo test 都会进入这里
    println!("Hello world!");
    
    // init os
    blog_os::init();
    
    #[cfg(test)]
    {
        serial_println!("Start unittests for main!");
        test_main(); // binary crate 单元测试入口
    }
    
    println!("It did not crash!");
    
    //loop {
        
        //use blog_os::print;
        //for _ in 0..30000 {}
        //print!("-");
        
        blog_os::hlt_loop();
        
    //}
}
#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}