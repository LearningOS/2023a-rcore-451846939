use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec;

/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}

/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        for tid in 0..process_inner.tasks.len() {
            process_inner.allocation[tid][0][id] = 0;
            process_inner.need[tid][0][id] = 0;
        }
        process_inner.available[0][id] = 1;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        for tid in 0..process_inner.tasks.len() {
            process_inner.allocation[tid][0].push(0);
            process_inner.need[tid][0].push(0);
        }
        process_inner.available[0].push(1);
        process_inner.mutex_list.len() as isize - 1
    };
    id
}

/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    // println!("sys_mutex_lock process_inner.available :{:?} tid :{:?} mutex_id :{:?} allocation:{:?} need:{:?}", process_inner.available.clone(), tid, mutex_id, process_inner.allocation.clone(), process_inner.need.clone());
    if process_inner.enable_deadlock_detect {
        // process_inner.allocation[tid][0]=1;
        // 定义如下三个数据结构：
        //
        // 可利用资源向量 Available ：含有 m 个元素的一维数组，每个元素代表可利用的某一类资源的数目， 其初值是该类资源的全部可用数目，其值随该类资源的分配和回收而动态地改变。
        // Available[j] = k，表示第 j 类资源的可用数量为 k。
        // 分配矩阵 Allocation：n * m 矩阵，表示每类资源已分配给每个线程的资源数。 Allocation[i,j] = g，则表示线程 i 当前己分得第 j 类资源的数量为 g。
        // 需求矩阵 Need：n * m 的矩阵，表示每个线程还需要的各类资源数量。 Need[i,j] = d，则表示线程 i 还需要第 j 类资源的数量为 d 。
        //算法运行过程如下：
        //
        // 设置两个向量: 工作向量 Work，表示操作系统可提供给线程继续运行所需的各类资源数目，它含有 m 个元素。
        // 初始时，Work = Available ；结束向量 Finish，表示系统是否有足够的资源分配给线程， 使之运行完成。初始时 Finish[0..n-1] = false，表示所有线程都没结束；
        // 当有足够资源分配给线程时， 设置 Finish[i] = true。
        // 从线程集合中找到一个能满足下述条件的线程
        // 1Finish[i] == false;
        // 2Need[i,j] ≤ Work[j];
        // 若找到，执行步骤 3，否则执行步骤 4。
        //
        // 当线程 thr[i] 获得资源后，可顺利执行，直至完成，并释放出分配给它的资源，故应执行:
        // 1Work[j] = Work[j] + Allocation[i, j];
        // 2Finish[i] = true;
        // 跳转回步骤2
        //
        // 如果 Finish[0..n-1] 都为 true，则表示系统处于安全状态；否则表示系统处于不安全状态，即出现死锁。
        process_inner.need[tid][0][mutex_id] += 1;

        let mut work = process_inner.available[0].clone();
        let mut finish = vec![false; process_inner.need.len()];
        let mut flag = true;
        while flag {
            flag = false;
            let thread_count = process_inner.need.len();
            for thread_id in 0..thread_count {
                if !finish[thread_id] {
                    let mut flag2 = true;
                    for j in 0..process_inner.need[thread_id][0].len() {
                        if process_inner.need[thread_id][0][j] > work[j] {
                            flag2 = false;
                            break;
                        }
                    }
                    if flag2 {
                        for j in 0..process_inner.need[thread_id][0].len() {
                            work[j] += process_inner.allocation[thread_id][0][j];
                        }
                        finish[thread_id] = true;
                        flag = true;
                    }
                }
            }
        }
        if finish.iter().all(|x| *x) {
            process_inner.available[0][mutex_id] -= 1;
            process_inner.allocation[tid][0][mutex_id] += 1;
            process_inner.need[tid][0][mutex_id] -= 1;
            drop(process_inner);
            drop(process);
            mutex.lock();
            return 0;
        } else {
            process_inner.need[tid][0][mutex_id] -= 1;
            return -0xDEAD;
        }
    } else {
        drop(process_inner);
        drop(process);
        mutex.lock();
    }

    0
}

/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    if process_inner.enable_deadlock_detect {
        process_inner.allocation[tid][0][mutex_id] -= 1;
        process_inner.available[0][mutex_id] += 1;
    }

    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}

/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        for tid in 0..process_inner.tasks.len() {
            process_inner.allocation[tid][1][id] = 0;
            process_inner.need[tid][1][id] = 0;
        }

        process_inner.available[1][id] = res_count;
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        for tid in 0..process_inner.tasks.len() {
            process_inner.allocation[tid][1].push(0);
            process_inner.need[tid][1].push(0);
        }
        process_inner.available[1].push(res_count);
        process_inner.semaphore_list.len() - 1
    };
    // println!("sys_semaphore_create process_inner.available :{:?} pid:{:?}", process_inner.available.clone(), tid);
    id as isize
}

/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    if process_inner.enable_deadlock_detect {
        // println!("sys_semaphore_up process_inner.available :{:?} tid :{:?} sem_id :{:?} allocation:{:?} need:{:?}", process_inner.available.clone(), tid, sem_id, process_inner.allocation.clone(), process_inner.need.clone());
        process_inner.available[1][sem_id] += 1;
        process_inner.allocation[tid][1][sem_id] -= 1;
    }
    drop(process_inner);
    sem.up();

    0
}

/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());

    if process_inner.enable_deadlock_detect {
        // 定义如下三个数据结构：
        //
        // 可利用资源向量 Available ：含有 m 个元素的一维数组，每个元素代表可利用的某一类资源的数目， 其初值是该类资源的全部可用数目，其值随该类资源的分配和回收而动态地改变。
        // Available[j] = k，表示第 j 类资源的可用数量为 k。
        // 分配矩阵 Allocation：n * m 矩阵，表示每类资源已分配给每个线程的资源数。 Allocation[i,j] = g，则表示线程 i 当前己分得第 j 类资源的数量为 g。
        // 需求矩阵 Need：n * m 的矩阵，表示每个线程还需要的各类资源数量。 Need[i,j] = d，则表示线程 i 还需要第 j 类资源的数量为 d 。
        //算法运行过程如下：
        //
        // 设置两个向量: 工作向量 Work，表示操作系统可提供给线程继续运行所需的各类资源数目，它含有 m 个元素。
        // 初始时，Work = Available ；结束向量 Finish，表示系统是否有足够的资源分配给线程， 使之运行完成。初始时 Finish[0..n-1] = false，表示所有线程都没结束；
        // 当有足够资源分配给线程时， 设置 Finish[i] = true。
        // 从线程集合中找到一个能满足下述条件的线程
        // 1Finish[i] == false;
        // 2Need[i,j] ≤ Work[j];
        // 若找到，执行步骤 3，否则执行步骤 4。
        //
        // 当线程 thr[i] 获得资源后，可顺利执行，直至完成，并释放出分配给它的资源，故应执行:
        // 1Work[j] = Work[j] + Allocation[i, j];
        // 2Finish[i] = true;
        // 跳转回步骤2
        //
        // 如果 Finish[0..n-1] 都为 true，则表示系统处于安全状态；否则表示系统处于不安全状态，即出现死锁。
        process_inner.need[tid][1][sem_id] += 1;
        let mut work = process_inner.available[1].clone();
        let mut finish = vec![false; process_inner.need.len()];
        let mut flag = true;
        // println!("sys_semaphore_down process_inner.available :{:?} tid :{:?} sem_id :{:?} allocation:{:?} need:{:?}", process_inner.available.clone(), tid, sem_id, process_inner.allocation.clone(), process_inner.need.clone());

        while flag {
            // println!("flag:{:?}", flag);
            flag = false;
            let thread_count = process_inner.need.len();
            for thread_id in 0..thread_count {
                if !finish[thread_id] {
                    let mut flag2 = true;
                    for j in 0..process_inner.need[thread_id][1].len() {
                        // println!("need:{:?}  work:{:?} thread_id:{:?} sem_id:{:?}", process_inner.need[thread_id][1][j], work[j], thread_id, j);
                        if process_inner.need[thread_id][1][j] > work[j] {
                            flag2 = false;
                            break;
                        }
                    }
                    if flag2 {
                        for j in 0..process_inner.need[thread_id][1].len() {
                            work[j] += process_inner.allocation[thread_id][1][j];
                        }
                        finish[thread_id] = true;
                        flag = true;
                    }
                }
            }
        }
        // println!("work:{:?}  finish:{:?}", work.clone(), finish.clone());
        if finish.iter().all(|x| *x == true) {
            if process_inner.available[1][sem_id] > 0 {
                process_inner.available[1][sem_id] -= 1;
                process_inner.need[tid][1][sem_id] -= 1;
                process_inner.allocation[tid][1][sem_id] += 1;
            }
            drop(process_inner);
            // drop(process);
            // println!("sem.down :{:?}", 0);
            sem.down();
            // println!("res:{:?}", 0);
            return 0;
        } else {
            process_inner.need[tid][1][sem_id] -= 1;
            // println!("res:{:?}", -0xDEAD);
            return -0xDEAD;
        }
    } else {
        drop(process_inner);
        sem.down();
    }
    // println!("  res:{:?}", 0);
    0
}

/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}

/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}

/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}

/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");

    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    if _enabled == 1 {
        process_inner.enable_deadlock_detect = true;
    } else {
        process_inner.enable_deadlock_detect = false;
    }

    0
}
