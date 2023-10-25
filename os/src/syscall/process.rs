//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus,
    },
};
use crate::mm::page_table::{PageTable};
use crate::mm::{mmap, PhysAddr, unmmap, VirtAddr, VirtPageNum};
use crate::task::{current_user_token, set_task_info};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let virt_addr = VirtAddr(_ts as usize);
    let phys_addr = translate_va(virt_addr);
    if let Some(phys_addr) = phys_addr {
        let us = get_time_us();
        let kernel_ts = phys_addr.0 as *mut TimeVal;
        unsafe {
            *kernel_ts = TimeVal {
                sec: us / 1_000_000,
                usec: us % 1_000_000,
            };
        }
        0
    } else {
        -1
    }
}

fn translate_va(virt_addr: VirtAddr) -> Option<PhysAddr> {
    PageTable::from_token(current_user_token()).translate_va(virt_addr)
}

pub fn translate_ptr<T>(ptr: *const T) -> *mut T {
    let page_table: PageTable = PageTable::from_token(current_user_token());

    let start: usize = ptr as usize;
    let start_va: VirtAddr = VirtAddr::from(start);
    let vpn: VirtPageNum = start_va.floor();
    let ppn: PhysAddr = page_table.translate(vpn).unwrap().ppn().into();

    let offset: usize = start_va.page_offset();
    let phys_addr: usize = ppn.into();
    let phys_ptr: *mut T = (offset + phys_addr) as *mut T;

    phys_ptr
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");


    let ti = translate_ptr( _ti);
    // let ti = translate_va(current_user_token().into());

    // if let Some(ti)=ti{
    //     let ti=ti.0 as *mut TaskInfo;
        set_task_info(ti);
    // }

    return 0;
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    let ok = mmap(start, len, port);
    if ok {
        return 0;
    }
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let ok = unmmap(start, len);
    if ok {
        return 0;
    }
    -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
