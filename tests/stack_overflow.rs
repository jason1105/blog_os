#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
use lazy_static::lazy_static;
use core::panic::PanicInfo;
use blog_os::{QemuExitCode, exit_qemu, serial_print, serial_println};
use blog_os::gdt;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! { // 集成测试入口, 这个集成测试没有使用测试框架, 因为double fault 不会返回, 所以会影响其他的测试. (在 Cargo.toml 中的 [[test]] 中关闭)

    serial_print!("stack_overflow::stack_overflow...\t");
    
    blog_os::gdt::init();
    init_test_idt();
    
    stack_overflow();
    
    loop {}
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}

lazy_static! {
    pub static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt_temp = InterruptDescriptorTable::new();
        
        // 在 IDT 中注册测试用的 double fault handler, 并指明使用 IST 中的第 0 号堆栈
        unsafe {
            idt_temp.double_fault
                .set_handler_fn(test_doublefault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_STACK_IST_INDEX);
        }
        idt_temp
    };
}

// 测试用 double fault handler
extern "x86-interrupt" fn test_doublefault_handler (
    _: InterruptStackFrame, _error_code: u64) -> !
{
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop{}
}

fn init_test_idt() {
    TEST_IDT.load();
}