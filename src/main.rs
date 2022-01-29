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
use bootloader::{BootInfo, entry_point}; // new
    
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

    println!("Hello world!");

    // init os
    blog_os::init();

    //////////////////////////////////////////
    // Temporary demostrate how to get virtual address regard to physical address
    //////////////////////////////////////////
    // use blog_os::memory::active_level_4_table; // now it's private.
    // use x86_64::{
        // structures::paging::PageTable,
        // VirtAddr,
    // };
    
    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let l4_table = unsafe { active_level_4_table(phys_mem_offset) };
    // for (i, entry) in l4_table.iter().enumerate() {
        // if !entry.is_unused() {
            // println!("L4 Entry {}: {:?}", i, entry);
            
            // let phys = entry.frame().unwrap().start_address();
            // let virt = phys.as_u64() + boot_info.physical_memory_offset;
            // let ptr = VirtAddr::new(virt).as_mut_ptr();
            // let l3_table: &PageTable = unsafe { &*ptr };
            // for (i, entry) in l3_table.iter().enumerate() {
                // if !entry.is_unused() {
                    // println!("L3 Entry {}: {:?}", i, entry);
                // }
            // }
        // }
    // }

    ////////////////////////////////////////////////
    // Temporary test tranlate address by tranversing page tables.
    ////////////////////////////////////////////////

    // use blog_os::memory::translate_addr;
    /* 
    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // arbitrary address
        0x0f00_1120_1abb,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];
     */
    // Translation wrote by ourselves will panic, because we didn't implements huge page
    // for address in addresses {
        // let virt = VirtAddr::new(address);
        // let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        // println!("{:?} -> {:?}", virt, phys);
    // }
    
    ////////////////////////////////////////////////
    // Temporary test tranlate address by using mapper offered by x86_64
    ////////////////////////////////////////////////
    // use blog_os::memory;
    // use x86_64::{structures::paging::Translate};
    
    // let mut mapper = unsafe { memory::init(phys_mem_offset) }; // OffsetPageTable
    
    // for address in addresses {
        // let virt = VirtAddr::new(address);
        // let phys = unsafe { mapper.translate_addr(virt) };
        // println!("{:?} -> {:?}", virt, phys);
    // }

    ///////////////////////////////////////
    // Temporary testing for PAGE MAPPING
    ///////////////////////////////////////
    /* 
    use blog_os::memory;
    use x86_64::structures::paging::page::Page;
    use x86_64::structures::paging::FrameAllocator;
    use x86_64::{
        structures::paging::PageTable,
        VirtAddr,
    };
    
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) }; // OffsetPageTable
    let page = Page::containing_address(VirtAddr::new(0x0));
    
    // let mut memory_allocator = memory::EmptyFrameAllocator; // Empty Allocator 
    let mut memory_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) }; // BootInfo Allocator

    memory::create_example_mapping(&mut mapper, page, &mut memory_allocator);

    let page_ptr: *mut u64 = VirtAddr::new(0x0).as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};
    */
 
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