use x86_64::{
    structures::paging::{
        PageTable,
        OffsetPageTable,
    },
    VirtAddr,
};

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

