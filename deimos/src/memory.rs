use x86_64::{
    PhysAddr, VirtAddr, registers, structures::paging::{
        FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB
    }
};
use bootloader::bootinfo::{
    MemoryMap,
    MemoryRegionType,
};

// A FrameAllocator that returns usable frames from the Bootloader's Memory Map
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0
        }
    }
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addrs = addr_ranges
            .flat_map(|r| r.step_by(4096));
        frame_addrs.map(|addr| PhysFrame::containing_address(
                PhysAddr::new(addr)))
    }

}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

pub unsafe fn init(phys_mem_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let l4_table = active_level_4_table(phys_mem_offset);
        OffsetPageTable::new(l4_table, phys_mem_offset)
    }
}

//This function is unsafe: caller must guarentee that the complete 
//physical memory is mapped to virtual memory at the passed VirtAddr
//this function must only be called once
unsafe fn active_level_4_table(phys_mem_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    let phys = Cr3::read().0.start_address();
    let virt = phys_mem_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    unsafe { &mut *page_table_ptr }

}

