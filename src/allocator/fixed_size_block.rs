use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::{self, NonNull},
};

use super::Locked;

const BLOCK_SIZE: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024];

struct ListNode {
    next: Option<&'static mut ListNode>,
}

pub struct FixedSizeBlockAllocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZE.len()],
    fall_back_allocator: linked_list_allocator::Heap,
}

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZE.len()],
            fall_back_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// 初始化列表, 添加一个节点, 即全部的堆内存
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fall_back_allocator.init(heap_start, heap_size);
    }

    /// allocate using the fallback allocator
    pub fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fall_back_allocator.allocate_first_fit(layout) {
            Ok(not_null) => not_null.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// returns the index of the block size
fn list_index(layout: &Layout) -> Option<usize> {
    BLOCK_SIZE.iter().position(|&s| s >= layout.size())
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // assuming that the layout is correct
        let mut allocator = self.lock();

        if let Some(index) = list_index(&layout) {
            if let Some(block) = allocator.list_heads[index].take() {
                allocator.list_heads[index] = block.next.take();
                return block as *mut ListNode as *mut u8;
            } else {
                let size = BLOCK_SIZE[index];
                let align = size;
                let layout = Layout::from_size_align(size, align).unwrap();
                return allocator.fallback_alloc(layout);
            }
        } else {
            return allocator.fallback_alloc(layout);
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        if let Some(index) = list_index(&layout) {
            let head = allocator.list_heads[index].take();
            let new_head = ptr as *mut ListNode;
            (*new_head).next = head;
            allocator.list_heads[index] = Some(&mut *new_head);
        } else {
            let ptr = NonNull::new(ptr).unwrap();
            allocator.fall_back_allocator.deallocate(ptr, layout);
        }
    }
}
