//! Memory management implementation
//!
//! SV39 page-based virtual-memory architecture for RV64 systems, and
//! everything about memory management, like frame allocator, page table,
//! map area and memory set, is implemented here.
//!
//! Every task or process has a memory_set to control its virtual memory.

mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
pub(crate) mod page_table;

use address::VPNRange;
pub use address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
pub use frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};
pub use memory_set::remap_test;
pub use memory_set::{kernel_token, MapPermission, MemorySet, KERNEL_SPACE};
use page_table::PTEFlags;
pub use page_table::{
    translated_byte_buffer, translated_ref, translated_refmut, translated_str, PageTable,
    PageTableEntry, UserBuffer, UserBufferIterator,
};
use crate::task::{add_current_memory_set, remove_current_memory_set};

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}

/// mmap
///
/// # Arguments
///
/// * `start`:
/// * `len`:
/// * `port`:
///
/// returns: ()
///
/// # Examples
///
/// ```
///
/// ```
pub fn mmap(start: usize, len: usize, port: usize) -> bool {
    let start_va: VirtAddr = start.into();
    let end_va: VirtAddr = (start + len).into();
    if start_va > end_va {
        return false;
    }
    if !start_va.aligned() {
        return false;
    }
    if port & !0x7 != 0 || port & 0x7 == 0 {
        return false;
    }
    // println!("before mmap: start_va: {:x}, end_va: {:x}", start_va.0, end_va.0);
    // let start_va:VirtAddr= (start_va.0 -start_va.page_offset()).into();
    // if end_va.page_offset()>0 {
    //     end_va = (end_va .0+ PAGE_SIZE - end_va.page_offset()).into();
    // }
    // println!("after mmap: start_va: {:x}, end_va: {:x}", start_va.0, end_va.0);
    let mut map_perm = MapPermission::U;
    let ph_flags = Flag(port);
    if ph_flags.is_read() {
        map_perm |= MapPermission::R;
    }
    if ph_flags.is_write() {
        map_perm |= MapPermission::W;
    }
    if ph_flags.is_execute() {
        map_perm |= MapPermission::X;
    }
    return add_current_memory_set(start_va, end_va, map_perm);
}

/// unmmap
pub fn unmmap(start: usize, len: usize) -> bool {
    let start_va: VirtAddr = start.into();
    let end_va: VirtAddr = (start + len).into();
    if !start_va.aligned() {
        debug!("unmap fail don't aligned");
        return false;
    }
    // println!("before unmmap: start_va: {:x}, end_va: {:x}", start_va.0, end_va.0);
    // let start_va:VirtAddr= (start_va.0 -start_va.page_offset()).into();
    // if end_va.page_offset()>0 {
    //     end_va = (end_va .0+ PAGE_SIZE - end_va.page_offset()).into();
    // }
    // println!("after unmmap: start_va: {:x}, end_va: {:x}", start_va.0, end_va.0);
    remove_current_memory_set(start_va, end_va)
}
/// Flag
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Flag(usize);
/// Flag
impl Flag {
    /// is_execute
    pub fn is_execute(&self) -> bool {
        self.0 & 0x4 == 0x4
    }
    /// is_write
    pub fn is_write(&self) -> bool {
        self.0 & 0x2 == 0x2
    }
    /// is_read
    pub fn is_read(&self) -> bool {
        self.0 & 0x1 == 0x1
    }
}