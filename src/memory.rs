use x86_64::{
    structures::paging::PageTable,
    structures::paging::page_table::PageTableEntry,
    VirtAddr,
    registers::control::Cr3,
    addr::PhysAddr,
    structures::paging::frame::PhysFrame,
    structures::paging::OffsetPageTable
};

/// Initialize a new OffsetPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
// physical_memory_offset 是在虚拟地址空间中的偏移量
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}


////////////////////////////////////////
// BootInfoFrameAllocator
////////////////////////////////////////

use bootloader::bootinfo::MemoryMap;
use bootloader::bootinfo::MemoryRegionType;
use x86_64::{
    structures::paging::{Page, Mapper, Size4KiB, FrameAllocator}
};

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }
    
    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter(); // Does iter() exist in MemoryMap??? 
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

//////////////////////////////////////
// temporary testing for PAGE MAPPING
//////////////////////////////////////
/* 
use x86_64::{
    structures::paging::{Page, Mapper, Size4KiB, FrameAllocator}
};

// map to 0xb8000
pub fn create_example_mapping(
    mapper: &mut OffsetPageTable,
    page: Page<Size4KiB>,
    frame_allocator:  &mut impl FrameAllocator<Size4KiB>
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    
    let flag = Flags::PRESENT | Flags::WRITABLE;
    
    let map_to_result = unsafe { mapper.map_to(page, frame, flag, frame_allocator) };
    
    map_to_result.expect("Fail to map").flush();
}

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        这里为什么返回None也可以? 因为在测试方法中, 指定的 page 所用到的四级页表都已经存在了, 所以不需要在为 page table 分配 frame 了.
        None
    }
}
 */
 
////////////////////////////////////////////////
// Temporary test tranlate address
////////////////////////////////////////////////
/*
pub  unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr)
    -> Option<PhysAddr>
{
    translate_addr_inner(addr, physical_memory_offset)
}

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) 
-> Option<PhysAddr>
{
    use x86_64::structures::paging::page_table::FrameError;
    
    let indexes = [addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()];
    
    let offset = addr.page_offset();
    
    let (level_4_table_frame, _) = Cr3::read(); // 这个frame中放的是level 4 page table, 正好完全放得下
    let phys = level_4_table_frame.start_address();
    let mut frame = level_4_table_frame;
    
    for page_index in indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
        let page_table = unsafe{&*page_table_ptr};
        
        let mut page_table_entry: &PageTableEntry = &page_table[page_index];
        frame = match page_table_entry.frame() { // 每个entry都指向一个frame
            Ok(frame) => frame, // 这个里放的是下一级的table 或者是物理内存
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        }
    }
    
    Some(frame.start_address() + u64::from(offset))
}
*/