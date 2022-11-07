test_code:
    addi sp, sp, -24
    sw a0, 0(sp)
    sw a1, 4(sp)
    lw t2, 0(sp)
    li t1, 2
    add t3, t2, t1
    sw t3, 8(sp)
    lw t4, 4(sp)
    li t1, 1
    add t5, t4, t1
    sw t5, 12(sp)
    lw t6, 8(sp)
    lw t0, 12(sp)
    sw t0, 16(sp)
    lw t1, 16(sp)
    add t0, t6, t1
    sw t0, 20(sp)
    lw a0, 20(sp)
    j test_code_end
test_code_end:
    addi sp, sp, 24
    ret
