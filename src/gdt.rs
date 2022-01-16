use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::gdt::Descriptor;

pub const DOUBLE_FAULT_STACK_IST_INDEX:u16 = 0;

// 定义 TSS
lazy_static! {
    static ref TSS:TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        tss.interrupt_stack_table[DOUBLE_FAULT_STACK_IST_INDEX as usize] = {
            const SIZE:usize = 4096 * 5;
            static mut STACK:[u8; SIZE] = [0;SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + SIZE;
            stack_end
        };
        tss
    };
}

// 定义 GDT
lazy_static! {
    static ref GDT:(GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let cs_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors{cs_selector, tss_selector})
    };
}


// 封装代码段selector 和 TSS selector
struct Selectors {
    cs_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

// 初始化 GDT
pub fn init() {
    
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;
    
    // 加载 GDT
    GDT.0.load();
    
    // 重新设置 cs, 并加载 tss
    unsafe {
        set_cs(GDT.1.cs_selector);
        load_tss(GDT.1.tss_selector);
    }
}









