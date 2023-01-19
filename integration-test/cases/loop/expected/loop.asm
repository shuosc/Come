.section .text
.global test_condition
test_condition:
    addi sp, sp, -16
test_condition_entry:
    sw a0, 0(sp)
    sw a1, 4(sp)
    li t0, 0
    sw t0, 8(sp)
    lw t6, 0(sp)
    sw t6, 12(sp)
    j loop_0_condition
loop_0_condition:
    lw t4, 12(sp)
    lw t5, 4(sp)
    slt t6, t4, t5
    li t1, 0
    bne t6, t1, loop_0_success
    j loop_0_fail
loop_0_success:
    lw t4, 8(sp)
    lw t5, 12(sp)
    add t6, t4, t5
    sw t6, 8(sp)
    lw t3, 12(sp)
    li t1, 1
    add t2, t3, t1
    sw t2, 12(sp)
    j loop_0_condition
loop_0_fail:
    lw t6, 8(sp)
    mv a0, t6
    j test_condition_end
test_condition_end:
    addi sp, sp, 16
    ret
