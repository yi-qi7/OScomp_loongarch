.section .text
.globl do_syscall
.align 4
do_syscall:
    # syscall(id: a0, args0: a1, args1: a2, args2: a3)
    addi.d $t0, $a0, 0    # 代替 move $t0,$a0
    addi.d $t1, $a1, 0    # 代替 move $t1,$a1
    addi.d $t2, $a2, 0    # 代替 move $t2,$a2
    addi.d $t3, $a3, 0    # 代替 move $t3,$a3
    addi.d $a7, $t0, 0    # 将系统调用号存入 $a7 (LoongArch 的 syscall 规范)
    addi.d $a0, $t1, 0    # 参数0 -> $a0
    addi.d $a1, $t2, 0    # 参数1 -> $a1
    addi.d $a2, $t3, 0    # 参数2 -> $a2
    syscall 0              # 必须带操作数 0
    jirl $zero, $ra, 0     # 代替 jr $ra (LoongArch 的返回指令)