#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
mod emulation;
mod serial;

#[cfg(test)]
mod tests;

use core::panic::PanicInfo;
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
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
}

