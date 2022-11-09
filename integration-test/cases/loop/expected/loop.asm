test_condition:
    addi sp, sp, -36
    sw a0, 0(sp)
    sw a1, 4(sp)
    li t0, 0
    sw t0, 8(sp)
    lw t2, 0(sp)
    sw t2, 12(sp)
    j loop_0_condition
loop_0_condition:
    lw t3, 12(sp)
    lw t4, 4(sp)
    slt t5, t3, t4
    li t1, 0
    bne t5, t1, loop_0_success
    j loop_0_fail
loop_0_success:
    lw t6, 8(sp)
    lw t0, 12(sp)
    sw t0, 16(sp)
    lw t1, 16(sp)
    add t0, t6, t1
    sw t0, 20(sp)
    lw t0, 20(sp)
    sw t0, 8(sp)
    lw t0, 12(sp)
    sw t0, 24(sp)
    lw t0, 24(sp)
    li t1, 1
    add t0, t0, t1
    sw t0, 28(sp)
    lw t0, 28(sp)
    sw t0, 12(sp)
    j loop_0_condition
loop_0_fail:
    lw t0, 8(sp)
    sw t0, 32(sp)
    lw a0, 32(sp)
    j test_condition_end
test_condition_end:
    addi sp, sp, 36
    ret
