test_code:
    addi sp, sp, -28
test_code_entry:
    sw a0, 0(sp)
    sw a1, 4(sp)
    lw t6, 0(sp)
    lw t0, 4(sp)
    sw t0, 24(sp)
    lw t1, 24(sp)
    add t5, t6, t1
    sw t5, 12(sp)
    lw t2, 0(sp)
    lw t3, 4(sp)
    slt t4, t2, t3
    li t1, 0
    bne t4, t1, if_0_success
    j if_0_fail
if_0_success:
    lw t6, 0(sp)
    lw t0, 4(sp)
    sw t0, 24(sp)
    lw t1, 24(sp)
    add t5, t6, t1
    sw t5, 16(sp)
    lw t4, 16(sp)
    sw t4, 8(sp)
    j if_0_end
if_0_fail:
    lw t6, 0(sp)
    lw t0, 0(sp)
    sw t0, 24(sp)
    lw t1, 24(sp)
    add t5, t6, t1
    sw t5, 20(sp)
    lw t4, 20(sp)
    sw t4, 8(sp)
    j if_0_end
if_0_end:
    lw t6, 8(sp)
    lw t0, 12(sp)
    sw t0, 24(sp)
    lw t1, 24(sp)
    add t5, t6, t1
    mv a0, t5
    j test_code_end
test_code_end:
    addi sp, sp, 28
    ret
