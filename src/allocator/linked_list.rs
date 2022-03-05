use super::Locked;
use crate::allocator::align_up;
use crate::println;
use crate::serial_print;
use alloc::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr;
use core::result::Result;

#[derive(Debug)]
// ListNode 表示一个未被使用的 region
struct ListNode {
    size: usize,                         // region 的大小
    next: Option<&'static mut ListNode>, // 静态全局引用, 意味着是一个指针, 即没有释放功能的 Box 类型
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

pub struct ListAllocator {
    head: ListNode, // 不用 &'static mut ListNode, 这样在实现的时候更方便
}

impl ListAllocator {
    /// 创建一个空列表, 在这个方法中没有用堆的确切边界去初始化 ListAllocator, 因为 ListNode 需要写入对内存, 这只有在分配内存的时候才能进行. 另外, 也因为这个方法会在静态域中被调用, 所以必须是常量方法, 常量方法的返回值必须是固定不变的. 所以我们分离出一个 init 方法用来做真正的初始化.
    pub const fn new() -> Self {
        ListAllocator {
            head: ListNode::new(0), // Fixed node as head.
        }
    }

    /// 初始化列表, 添加一个节点, 即全部的堆内存
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    /// 把一个未被使用的区域添加到列表头
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // 确保 addr 是对齐的. 最大限度利用, TODO 怎么保证?
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        // 确保有足够的空间才能写入 ListNoe
        assert!(size >= mem::size_of::<ListNode>());

        // 这个 node 很快就结束了
        let mut node = ListNode::new(size);
        node.size = size;
        node.next = self.head.next.take();

        let node_ptr = addr as *mut ListNode; // 构建指针
        node_ptr.write(node); // 写入指针指向的内存, 不影响 node

        self.head.next = Some(&mut *node_ptr);
    }

    // 返回 ListNode  已分配的区域
    //      usize:    分配的起始地址 (TODO 这个应该就是已分配区域的起始地址吧)
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut region) = current.next {
            // 先借用了可变 current.next, 所以先使用. 为什么使用 ref mut, 因为 current.next 是 &mut 类型, 其没有实现 copy 和 clone, 所以会 move 到 region 中. 使用 ref mut 可以避免 move.
            // println!("{:?}", region);
            // let a:() = &region;
            use core::ops::Deref;
            use core::ops::DerefMut;

            if let Ok(allo_start) = Self::alloc_from_region(&region, size, align) {
                // &region 可以将 &mut &mut ListNode 改变成 &ListNode.

                let next = region.next.take(); // 虽然 region 的类型是 &mut &mut ListNode, 但会自动解除多级引用, 然后再调用其方法
                let ret = Some((current.next.take().unwrap(), allo_start)); // 这里不能用 region , 因为 region 是对 current.next 的借用, Java 中只需要 current.next = other 就可以修改 current.next 指针, 但是在 Rust 中, current.next 这个变量如果已经借用给了 region, 就不能再修改了.
                current.next = next;
                return ret;
            } else {
                current = current.next.as_mut().unwrap(); // next 是 &mut 没有实现 copy, 直接 unwrap 会发生 move. as_mut() 方法内部用 match + ref mut 得到了一个引用的副本.
                                                          // variable 'next' with &mut type without implement of copy can't be unwrapped
            }
        }

        None
    }

    /// allocate memory from region
    ///     &ListNode:    region for allocation
    ///     usize:    size of allocation
    ///     usize:    alignment of allocation
    /// return:
    ///    usize:    start address of allocation
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let region_start = region.start_addr();
        let region_end = region.end_addr();
        let alloc_start = align_up(region_start, align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        // if region too small
        if alloc_end > region_end {
            return Err(());
        }

        let excess = region_end - alloc_end;

        // remaining memory of the region will be allocated again, so its size must larger than requirement of ListNode
        if excess > 0 && excess < mem::size_of::<ListNode>() {
            return Err(());
        }

        return Ok(alloc_start);
    }

    /// return
    ///     usize: size
    ///     usize: align
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>()) // 根据 CPU 不同, 对齐单位不同,
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>()); // ListNode的大小
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<ListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // perform layout adjustments
        let (size, align) = ListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // perform layout adjustments
        let (size, _) = ListAllocator::size_align(layout);

        self.lock().add_free_region(ptr as usize, size)
    }
}
