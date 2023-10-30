
特别需要注意的是 在调用spawn的时候需要创建自己进程地址空间，剩下的和fork+exec就很像，但是spawn不需要复制父进程的地址空间

创建完成Task后加入Task的调度队列




当stride值溢出时，比较操作可能会出现问题

如果所有进程的优先级都大于等于2，并且严格按照算法执行，那么即使考虑了stride值溢出的情况，最大stride值和最小stride值之间的差距也不会超过BigStride的一半。

```rust
impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_value = self.0 as i8;
        let other_value = other.0 as i8;

        let half_big_stride = (BigStride / 2) as i8;

        if self_value >= 0 && other_value < 0 && self_value - other_value > half_big_stride {
            Some(Ordering::Less)
        } else if self_value < 0 && other_value >= 0 && other_value - self_value > half_big_stride {
            Some(Ordering::Greater)
        } else {
            Some(self_value.cmp(&other_value))
        }
    }
}
```

# 荣誉规则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
   独立完成，无交流对象。

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
   独立完成，参考rcore文档和risc-v指令集。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。