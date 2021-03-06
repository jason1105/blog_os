use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use linked_list_allocator::LockedHeap;

pub mod bump; // new
pub mod fixed_size_block; // new
pub mod linked_list; // new

// virtual address range
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

use bump::BumpAllocator;
use linked_list::ListAllocator;

use self::fixed_size_block::FixedSizeBlockAllocator;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
// static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
// static ALLOCATOR: Locked<ListAllocator> = Locked::new(ListAllocator::new());
// static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());
// static ALLOCATOR:Dummy = Dummy;

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        panic!("dealloc should be never called")
    }
}

// 初始堆内存
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    // prepare page range
    let heap_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in heap_range {
        // prepare frame
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::<Size4KiB>::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        // map page to frame
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    // 初始化内存分配器
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

/* fn align_up (addr: usize, align: usize) -> usize{
    let remainder = addr % align;

    if remainder == 0 {
        addr
    } else {
        addr - remainder + align
    }
} */

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
