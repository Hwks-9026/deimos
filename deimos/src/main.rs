#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
mod interrupts;
mod emulation;
mod serial;

#[cfg(test)]
mod tests;

use core::panic::PanicInfo;
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use crate::emulation::{QemuExitCode, exit_qemu};

    serial_println!("[Failed]\n");
    println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);

    loop {} //Still need a loop - Compiler does not know that exit_qemu will terminate the code
}

fn init() {
    interrupts::init_idt();
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    
    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    main();

    loop {}
}

fn main() {
    println!("Booting deimOS...");
    init();
    fn stack_overflow() {
        stack_overflow();
    }
    stack_overflow();

    println!("It did not crash!");
}

