

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, Entry};

use lazy_static::lazy_static;
use crate::println;

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt_temp = InterruptDescriptorTable::new();
        idt_temp.breakpoint.set_handler_fn(breakpoint_handler);
        idt_temp
    };
}

extern "x86-interrupt" fn breakpoint_handler (stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

pub fn init_idt() {
    IDT.load();
}

#[test_case]
fn test_interrupt_idt() {
    x86_64::instructions::interrupts::int3();
}