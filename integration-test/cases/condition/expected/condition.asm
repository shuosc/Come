.section .text
.global test_condition
test_condition:
    addi sp, sp, -8
test_condition_entry:
    sw a0, 0(sp)
    sw a1, 4(sp)
    lw t2, 0(sp)
    lw t3, 4(sp)
    slt t4, t2, t3
    li t1, 0
    bne t4, t1, if_0_success
    j if_0_fail
if_0_success:
    lw t4, 0(sp)
    mv a0, t4
    j test_condition_end
if_0_fail:
    lw t4, 4(sp)
    mv a0, t4
    j test_condition_end
if_0_end:
    j test_condition_end
test_condition_end:
    addi sp, sp, 8
    ret
