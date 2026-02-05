use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;


use lazy_static::lazy_static;

//THIS IS UNSAFE (we don't care) BUT LAZY_STATIC ABSTRACTS IT AWAY
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("EXCEPTION WITH ERROR CODE {:?}: DOUBLE FAULT\n{:#?}", error_code, stack_frame);
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}

#[test_case]
fn trigger_page_fault() {
    unsafe {
        *(0xdeadbeef as *mut u8) = 42;
    };
}
