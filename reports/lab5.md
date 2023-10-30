银行家算法：在进程中分配3个数组 但是要区分一下锁和信号量，这里用了一个数组0和1进行表示算法就是如下

定义如下三个数据结构：

可利用资源向量 Available ：含有 m 个元素的一维数组，每个元素代表可利用的某一类资源的数目， 其初值是该类资源的全部可用数目，其值随该类资源的分配和回收而动态地改变。 Available[j] = k，表示第 j 类资源的可用数量为 k。
分配矩阵 Allocation：n * m 矩阵，表示每类资源已分配给每个线程的资源数。 Allocation[i,j] = g，则表示线程 i 当前己分得第 j 类资源的数量为 g。
需求矩阵 Need：n * m 的矩阵，表示每个线程还需要的各类资源数量。 Need[i,j] = d，则表示线程 i 还需要第 j 类资源的数量为 d 。
算法运行过程如下：

设置两个向量: 工作向量 Work，表示操作系统可提供给线程继续运行所需的各类资源数目，它含有 m 个元素。初始时，Work = Available ；结束向量 Finish，表示系统是否有足够的资源分配给线程， 使之运行完成。初始时 Finish[0..n-1] = false，表示所有线程都没结束；当有足够资源分配给线程时， 设置 Finish[i] = true。
从线程集合中找到一个能满足下述条件的线程
1Finish[i] == false;
2Need[i,j] ≤ Work[j];
若找到，执行步骤 3，否则执行步骤 4。

当线程 thr[i] 获得资源后，可顺利执行，直至完成，并释放出分配给它的资源，故应执行:
1Work[j] = Work[j] + Allocation[i, j];
2Finish[i] = true;
跳转回步骤2

如果 Finish[0..n-1] 都为 true，则表示系统处于安全状态；否则表示系统处于不安全状态，即出现死锁。


需要特别注意初始化的状态还有sleep的 time记得重写，太坑了找了很久问题，结果是sleep的 get time的问题


1. 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 - 需要回收的资源有哪些？ - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？


进程结束以后需要回收内存，关闭fd，线程调度器需要移除该task，创建的信号量，锁等均需要进行回收


2. 对比以下两种 Mutex.unlock 的实现，二者有什么区别？这些区别可能会导致什么问题？
```rust
impl Mutex for Mutex1 {
    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        mutex_inner.locked = false;
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        }
    }
}

impl Mutex for Mutex2 {
    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }
}
```


第一种在unlock的时候会先释放锁标志，然后再运行add task
第二种在unlock的时候会先add task，然后再释放锁标志。

如果这时候产生time中断那么锁标识被重制了，但是可能task还没有被添加进入伍队列

# 荣誉规则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
   独立完成，无交流对象。

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
   独立完成，参考rcore文档和risc-v指令集。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。