#![feature(core_intrinsics)]
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::tests::test_runner)]
#![reexport_test_harness_main = "test_run"]

extern crate alloc;


mod hardware_interface;
mod memory_management;
mod emulation;
mod logo;
use hardware_interface::vga_buffer;


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
use hardware_interface::serial;

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use crate::emulation::qemu::{QemuExitCode, exit_qemu};

    serial_println!("[Failed]\n");
    println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
}

fn init(boot_info: &'static BootInfo) {
    use hardware_interface::{gdt, interrupts};
    gdt::init();
    interrupts::init_idt();
    unsafe {
        interrupts::PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
    print!("Aquiring physical memory offset...");
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    println!("[ok]");

    print!("Creating Memory Mapper...");
    let mut mapper = unsafe { memory_management::page_table::init(phys_mem_offset) };
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
    println!("{}", heap_string);
}

use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;


#[cfg(test)]
entry_point!(test_main);

#[cfg(test)]
fn test_main(_boot_info: &'static BootInfo) -> ! {
    test_run(); loop{}
}

#[cfg(not(test))]
entry_point!(main);

use alloc::boxed::Box;
use memory_management::{page_table::BootInfoFrameAllocator, allocator};

fn main(boot_info: &'static BootInfo) -> ! {
    vga_buffer::init();
    println!("Booting deimOS...");
    
    println!("Initializing...\n"); // 2 newlines are intentional
    init(boot_info);
    logo::println_logo();
    hlt(); // call halt that way when interrupts aren't firing, the CPU isn't active
}

fn hlt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
