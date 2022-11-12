test_condition:
    addi sp, sp, -8
    sw a0, 0(sp)
    sw a1, 4(sp)
    lw t2, 0(sp)
    lw t3, 4(sp)
    slt t4, t2, t3
    li t1, 0
    bne t4, t1, if_0_success
    j if_0_fail
if_0_success:
    lw t5, 0(sp)
    mv a0, t5
    j test_condition_end
if_0_fail:
    lw t6, 4(sp)
    mv a0, t6
    j test_condition_end
if_0_end:
    j test_condition_end
test_condition_end:
    addi sp, sp, 8
    ret
