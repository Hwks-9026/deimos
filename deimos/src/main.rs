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
mod gdt;

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
}

fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe {
        interrupts::PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {

    vga_buffer::init();

    #[cfg(test)]
    test_main();
    #[cfg(test)]
    loop {} //neccesary because the compiler can't realize that test_main returns !

    #[cfg(not(test))]
    main();

}

fn main() -> ! {
    println!("Booting deimOS...");
    print!("Initializing...");
    init();
    println!("[ok]");
    hlt();
}

fn hlt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
