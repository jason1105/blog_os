#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use blog_os::allocator; // new import
use blog_os::memory::{self, BootInfoFrameAllocator};
use x86_64::VirtAddr;
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

// tests 文件夹中的文件只有在 cargo test 中执行
entry_point!(main);
#[no_mangle] // don't mangle the name of this function

fn main(boot_info: &'static BootInfo) -> ! {

    blog_os::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset); // 这个 offset 是物理地址在虚拟地址中的偏移量, 它是一个虚拟地址
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
                .expect("heap initialization failed");
    
    test_main();
    
    loop{}
}

    
    
    
use alloc::boxed::Box;

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

use alloc::vec::Vec;

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

use alloc::vec::Vec;

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

// 1 2 3 4 5 6