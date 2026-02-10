use crate::{print, println};
use crate::memory_management::linked_list::LinkedListAllocator;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 4194304; // 2^22

pub struct Locked<A> {
    inner: spin::Mutex<A>
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner)
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<'_, A> {
        self.inner.lock()
    }
}

#[global_allocator]
static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    print!("\n    computing page range...");
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    print!("[ok]\n    Allocating page range...");
    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }
    print!("[ok]\n    Initializing Allocator...");
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    println!("[ok]");

    print!("    Testing Allocator Coalescence...");
    test_coalescence();
    println!("[ok]");

    Ok(())
}


use alloc::vec::Vec;
use core::alloc::Layout;

fn test_coalescence() {
    let chunk_size = 256;
    let iterations = 14000;
    let mut chunks = Vec::new();

    for _ in 0..iterations {
        let mut v = Vec::<u8>::with_capacity(chunk_size);
        v.push(1);
        chunks.push(v);
    }

    drop(chunks);

    let giant_layout = Layout::from_size_align(chunk_size * iterations, 8).unwrap();
    unsafe {
        let ptr = alloc::alloc::alloc(giant_layout);
        
        if ptr.is_null() {
            panic!("Fragmentation Test Failed: Could not coalesce blocks!");
        } else {
            alloc::alloc::dealloc(ptr, giant_layout);
        }
    }
}
