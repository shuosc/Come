test_code:
    addi sp, sp, -72
    sw a0, 0(sp)
    sw a1, 4(sp)
    lw t2, 0(sp)
    lw t3, 4(sp)
    add t4, t2, t3
    sw t4, 12(sp)
    lw t5, 0(sp)
    lw t6, 4(sp)
    slt t0, t5, t6
    sw t0, 16(sp)
    lw t0, 16(sp)
    li t1, 0
    bne t0, t1, if_0_success
    j if_0_fail
if_0_success:
    lw t0, 0(sp)
    sw t0, 24(sp)
    lw t0, 4(sp)
    sw t0, 28(sp)
    lw t0, 24(sp)
    lw t1, 28(sp)
    add t0, t0, t1
    sw t0, 32(sp)
    lw t0, 32(sp)
    sw t0, 20(sp)
    lw t0, 20(sp)
    sw t0, 36(sp)
    lw t0, 36(sp)
    sw t0, 8(sp)
    j if_0_end
if_0_fail:
    lw t0, 0(sp)
    sw t0, 44(sp)
    lw t0, 0(sp)
    sw t0, 48(sp)
    lw t0, 44(sp)
    lw t1, 48(sp)
    add t0, t0, t1
    sw t0, 52(sp)
    lw t0, 52(sp)
    sw t0, 40(sp)
    lw t0, 40(sp)
    sw t0, 56(sp)
    lw t0, 56(sp)
    sw t0, 8(sp)
    j if_0_end
if_0_end:
    lw t0, 8(sp)
    sw t0, 60(sp)
    lw t0, 12(sp)
    sw t0, 64(sp)
    lw t0, 60(sp)
    lw t1, 64(sp)
    add t0, t0, t1
    sw t0, 68(sp)
    lw a0, 68(sp)
    j test_code_end
test_code_end:
    addi sp, sp, 72
    ret
