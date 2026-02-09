use alloc::vec::Vec;
use core::alloc::Layout;

pub fn test_coalescence() {
    let chunk_size = 256;
    let iterations = 10;
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
