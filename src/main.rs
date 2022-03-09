#![no_std] // 不使用 Rust 标准库
#![no_main] // 不使用 main 函数
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
// Rust 生成一个 main 方法调用 test_runner, 但我们的程序不使用 main 函数, 所以只能在 _start 中调用 main 函数, 但 main 函数是被系统调用的, 这里把 main 改名为 test_main, 在 test_main 中调用 test_runner, 但程序的入口还是 _start, 所以要在 _start 中调用 test_main
#![allow(warnings)]

#[cfg(test)]
use blog_os::serial_println; // 引用宏不用包含 module 名称
use blog_os::{
    println,
    task::{executor::Executor, keyboard::keyboard_task, simple_executor::SimpleExecutor, Task},
}; // 引用宏不用包含 module 名称
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
extern crate alloc;

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

entry_point!(kernel_main); // 定义入口

// #[no_mangle] // delete this
// pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! { // cargo run 和 cargo test 都会进入这里
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::allocator; // new import
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello world!");

    // init os
    blog_os::init();

    ////////////////////////////////////
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset); // 这个 offset 是物理地址在虚拟地址中的偏移量, 它是一个虚拟地址
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

    async fn async_number() -> u32 {
        42
    }

    async fn example_task() {
        let number = async_number().await;
        println!("async number: {}", number);
    }

    let mut executor = SimpleExecutor::new();
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard_task()));
    executor.run();
    ////////////////////////////////////

    #[cfg(test)]
    {
        serial_println!("Start unittests for main!");
        test_main(); // binary crate 单元测试入口
    }

    println!("It did not crash!");

    blog_os::hlt_loop();
}
#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
