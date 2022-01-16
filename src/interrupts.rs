

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use lazy_static::lazy_static;
use crate::println;
use crate::gdt;

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt_temp = InterruptDescriptorTable::new();
        
        // 在 IDT 中注册 handler 
        idt_temp.breakpoint.set_handler_fn(breakpoint_handler);
        idt_temp.page_fault.set_handler_fn(pagefault_handler);
        unsafe {
            idt_temp.double_fault.set_handler_fn(doublefault_handler).set_stack_index(gdt::DOUBLE_FAULT_STACK_IST_INDEX);
        }
        idt_temp
    };
}

extern "x86-interrupt" fn breakpoint_handler (stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn pagefault_handler (stack_frame: InterruptStackFrame, _error_code: PageFaultErrorCode) {
    println!("EXCEPTION: PAGEFAULT\n{:#?}", stack_frame);
}

// double fault 的 handler
extern "x86-interrupt" fn doublefault_handler (
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub fn init_idt() {
    IDT.load();
}

#[test_case]
fn test_interrupt_idt() {
    x86_64::instructions::interrupts::int3();
}