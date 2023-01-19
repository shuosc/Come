.section .text
.global f
f:
    addi sp, sp, -104
f_entry:
    sw a0, 0(sp)
    sw a1, 4(sp)
    lw t3, 0(sp)
    lw t4, 4(sp)
    sw t3, 56(sp)
    lw t0, 0(sp)
    sw t0, 60(sp)
    lw t0, 64(sp)
    sw t0, 72(sp)
    lw t0, 56(sp)
    lw t1, 68(sp)
    add t2, t0, t1
    lw t0, 0(sp)
    sw t0, 72(sp)
    lw t0, 80(sp)
    sw t0, 0(sp)
    lw t0, 0(sp)
    sw t0, 92(sp)
    lw t0, 96(sp)
    sw t0, 104(sp)
    lw t5, 0(sp)
    lw t6, 4(sp)
    sw t5, 8(sp)
    lw t0, 100(sp)
    lw t1, 8(sp)
    add t0, t0, t1
    sw t0, 88(sp)
    lw t0, 0(sp)
    sw t0, 12(sp)
    lw t0, 12(sp)
    sw t0, 20(sp)
    lw t0, 88(sp)
    sw t0, 24(sp)
    lw t0, 20(sp)
    sw t0, 0(sp)
    lw t0, 0(sp)
    sw t0, 32(sp)
    lw t0, 32(sp)
    sw t0, 40(sp)
    lw t0, 0(sp)
    sw t0, 44(sp)
    lw t0, 48(sp)
    sw t0, 56(sp)
    lw t0, 40(sp)
    lw t1, 52(sp)
    add t0, t0, t1
    sw t0, 28(sp)
    lw a0, 28(sp)
    j f_end
f_end:
    addi sp, sp, 104
    ret
