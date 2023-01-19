.section .text
.global test_code
test_code:
    addi sp, sp, -24
test_code_entry:
    sw a0, 0(sp)
    sw a1, 4(sp)
    lw t3, 0(sp)
    li t1, 2
    add t2, t3, t1
    sw t2, 8(sp)
    lw t5, 4(sp)
    li t1, 1
    add t4, t5, t1
    sw t4, 12(sp)
    lw t0, 8(sp)
    sw t0, 16(sp)
    lw t0, 12(sp)
    sw t0, 20(sp)
    lw t0, 16(sp)
    lw t1, 20(sp)
    add t6, t0, t1
    mv a0, t6
    j test_code_end
test_code_end:
    addi sp, sp, 24
    ret
