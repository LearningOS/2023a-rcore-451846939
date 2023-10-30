# 总结
在`TaskControlBlock` 加入 `task_start_time` 和  `syscall_times`字段在loadApp的时候进行初始化
```rust
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The task start time
    pub task_start_time:usize,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
}
```

在`syscall`中加入`syscall_times`的更新

每次获取运行时间的时候去获取当前time根据`task_start_time`计算出运行时间

添加`get_current_task` 函数用于获取当前运行的task



# 简答作业

1. ch2b_bad_instructions
   `sret` 从 S 模式返回 U 模式：在 U 模式下执行会产生非法指令异常
2. ch2b_bad_register
   `sstatus` 指令访问了 S模式特权级下才能访问的寄存器 或内存，如表示S模式系统状态的 控制状态寄存器 sstatus 等。
3. ch2b_bad_address
   当前没有虚拟内存，都是直接访问物理内存，如果直接访问0x00是不可以的在 Qemu模拟的 virt 硬件平台上，物理内存的起始物理地址为 0x80000000
4.  在批处理系统中， L40：刚进入 __restore 时，a0 指向分配 Trap 上下文之后的内核栈栈顶 ，__restore 2个场景：1.系统调用返回 2.task任务切换返回用户态
5. ```asm
   ld t0, 32*8(sp)
   ld t1, 33*8(sp)
   ld t2, 2*8(sp)
   csrw sstatus, t0
   csrw sepc, t1
   csrw sscratch, t2
   ```
   用到了t0、t1 和 t2 三个临时寄存器，分别保存了 sstatus、sepc 和 sscratch 的值。
    - sstatus 寄存器的值保存了之前的系统状态标志，包括异常使能位和用户态/内核态切换位。在进入用户态之前，通过将之前保存的 sstatus 值写回 sstatus 寄存器，代码可以恢复之前的系统状态。
    - sepc 寄存器的值保存了之前的异常程序计数器的地址。在进入用户态之前，通过将之前保存的 sepc 值写回 sepc 寄存器，代码可以设置正确的返回地址，以便从异常或中断返回到用户程序的正确位置。
    - sscratch 寄存器的值保存了之前的用户栈的指针。在进入用户态之前，通过将之前保存的 sscratch 值写回 sscratch 寄存器，代码可以恢复之前的用户栈指针，确保从内核栈切换回用户栈后能正确执行用户程序。

   然后，通过 csrw 指令将这三个寄存器的值写入对应的 CSR 中。最后，通过 sret 指令从 S 模式返回 U 模式。
6. ```asm
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    ```
对于通用寄存器而言，两条控制流（应用程序控制流和内核控制流）运行在不同的特权级，所属的软件也可能由不同的编程语言编写，虽然在 Trap 控制流中只是会执行 Trap 处理相关的代码，但依然可能直接或间接调用很多模块，因此很难甚至不可能找出哪些寄存器无需保存。既然如此我们就只能全部保存了。但这里也有一些例外，如 x0 被硬编码为 0 ，它自然不会有变化；还有 tp(x4) 寄存器，除非我们手动出于一些特殊用途使用它，否则一般也不会被用到。
我们在这里也不保存 sp(x2)，因为我们要基于它来找到每个寄存器应该被保存到的正确的位置,x2是通常用作栈指针sp=（Stack Pointer）

7. __restore中
    ```asm
   csrrw sp, sscratch, sp
    ```
   之后交换 sscratch 和 sp， sp 重新指向用户栈栈顶，sscratch 也依然保存进入 Trap 之前的状态并指向内核栈栈顶。

8. __restore 状态切换发生在 `sret` （Supervisor Return）。当执行 "sret" 指令时，处理器会从特权级高的特权模式（如S模式）返回到特权级低的特权模式（如U模式），这样就完成了状态切换。
9. __alltraps中
```asm
    csrrw sp, sscratch, sp
```
csrrw 原型是  可以将 CSR 当前的值读到通用寄存器  中，然后将通用寄存器  的值写入该 CSR 。因此这里起到的是交换 sscratch 和 sp 的效果。在这一行之前 sp 指向用户栈， sscratch 指向内核栈（原因稍后说明），现在 sp 指向内核栈， sscratch 指向用户栈。
10. 从 U 态进入 S 态是 ecall










# 荣誉规则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
   独立完成，无交流对象。

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
   独立完成，参考rcore文档和risc-v指令集。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。