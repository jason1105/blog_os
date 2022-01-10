#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::println;
use blog_os::serial_println;

// tests 文件夹中的文件只有在 cargo test 中执行

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! { // 集成测试入口
    
    serial_println!("Start integration tests for basic_root.");
    
    test_main();

    loop {}
}


#[test_case]
fn test_println() {
    println!("test_println output");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}