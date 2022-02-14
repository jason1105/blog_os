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
use bootloader::{BootInfo, entry_point};
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
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
                .expect("heap initialization failed");
    
    use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

    // allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);
    
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());
    
    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));
    
    
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