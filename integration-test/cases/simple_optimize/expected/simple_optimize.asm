test_code:
test_code_entry:
    li t1, 2
    add t2, a0, t1
    li t1, 1
    add t3, a1, t1
    add t4, t2, t3
    mv a0, t4
    ret
