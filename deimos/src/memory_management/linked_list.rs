use core::{ptr, mem};

use super::allocator::Locked;
use alloc::alloc::{GlobalAlloc, Layout};
use x86_64::align_up;


pub struct LinkedListAllocator {
    head: ListNode
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0)
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        unsafe {
            self.add_free_region(heap_start, heap_size)
        }
    }

    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {

        // Assertions to verify at least one ListNode can fit in the free region
        assert_eq!(align_up(addr as u64, mem::align_of::<ListNode>() as u64), addr as u64);
        assert!(size >= mem::size_of::<ListNode>());

        let mut current = &mut self.head;
        while let Some(ref mut next_node) = current.next {
            if next_node.start_addr() > addr {
                break; // The right place to insert free node, ensures that the free regions are
                       // all sorted by address
            }
            current = current.next.as_mut().unwrap();
        }



        let mut node = ListNode::new(size);
        node.next = current.next.take();
        let node_ptr = addr as *mut ListNode;

        //safety guarentees are above with assertions
        unsafe {
            node_ptr.write(node);
            current.next = Some(&mut *node_ptr);
        }
        let current_start_addr = current.start_addr();
        let current_end_addr = current.end_addr();


        let new_ref = current.next.as_mut().unwrap();
        let new_ref_start_addr = new_ref.start_addr();
        let new_ref_end_addr = new_ref.end_addr();
        

        let next_node_data = if let Some(ref mut next_node) = new_ref.next {
            if new_ref_end_addr == next_node.start_addr() {
                // The blocks are touching!
                (Some(next_node.size), Some(next_node.next.take()))
            }
            else {(None, None)}
        }
        else {(None, None)};

        if let (Some(next_node_size), Some(next_node_next)) = next_node_data {
                new_ref.size += next_node_size;
                new_ref.next = next_node_next;
        }

        if current_start_addr != super::allocator::HEAP_START && current_end_addr == new_ref_start_addr {
            // The previous block and new block are touching!
            current.size += new_ref.size;
            current.next = new_ref.next.take();
        }
    }
    
    // Looks for a free region with the given size and alignment and removes it from the list 
    // Returns a tuple containing the ListNode and start address of the allocation
    fn find_region(&mut self, size: usize, align: usize) 
        -> Option<(&'static mut ListNode, usize)> 
    {
        let mut current = &mut self.head.next;

        loop {
            let alloc_start = if let Some(region) = current.as_deref() {
                Self::alloc_from_region(region, size, align).ok()
            } else {
                return None; 
            };

            if let Some(start) = alloc_start {
                let node = current.take().unwrap();
                *current = node.next.take();
                return Some((node, start));
            }

            current = &mut current.as_mut().unwrap().next;
        }
    }

    fn alloc_from_region(region: &ListNode, size: usize, align: usize) 
        -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr() as u64, align as u64) as usize;
        let alloc_end: usize = alloc_start.checked_add(size).ok_or(())?;
        if alloc_end > region.end_addr() {
            return Err(()); // Region too small >:(
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            return Err(());
        }

        Ok(alloc_start)
    }

    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("[Err: Overflow]");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                unsafe {
                    allocator.add_free_region(alloc_end, excess_size);
                }
            }
            alloc_start as *mut u8
        }
        else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);
        unsafe {self.lock().add_free_region(ptr as usize, size);}
    }
}

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

