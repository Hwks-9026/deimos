#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_run"]

extern crate alloc;

mod vga_buffer;
mod interrupts;
mod emulation;
mod allocator;
mod serial;
mod memory;
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

use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr, structures::paging::{Page}};

use crate::memory::BootInfoFrameAllocator;


#[cfg(test)]
entry_point!(test_main);

#[cfg(test)]
fn test_main(_boot_info: &'static BootInfo) -> ! {
    test_run(); loop{}
}

#[cfg(not(test))]
entry_point!(main);

use alloc::boxed::Box;

fn main(boot_info: &'static BootInfo) -> ! {
    vga_buffer::init();
    println!("Booting deimOS...");
    
    print!("Initializing...");
    init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    println!("[ok]");

    print!("Creating Memory Mapper...");
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    println!("[ok]");


    print!("Creating Frame Allocator...");
    let mut frame_allocator = unsafe { 
        BootInfoFrameAllocator::init(&boot_info.memory_map) 
    };
    println!("[ok]");

    print!("Initializing Global Heap Allocator...");
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("[err: heap initialization failed]");
    println!("[ok]");
    
    print!("Testing heap allocation...");
    let heap_string = Box::new("[ok]");
    print!("{}", heap_string);
    
    

    hlt(); // call halt that way when interrupts aren't firing, the CPU isn't active
}

fn hlt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
