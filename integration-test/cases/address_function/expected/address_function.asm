.section .text
.global main
main:
main_entry:
    j loop_0_condition
loop_0_condition:
    li t0, 1
    li t1, 0
    bne t0, t1, loop_0_success
    j loop_0_fail
loop_0_success:
    li a0, 2147491840
    lw a0, 0(a0)
    mv t2, a0
    li t1, 0
    sub t3, t2, t1
    seqz t3, t3
    li t1, 0
    bne t3, t1, if_0_success
    j if_0_fail
if_0_success:
    li a1, 1
    li a0, 2147491840
    sw a1, 0(a0)
    j if_0_end
if_0_fail:
    li a1, 0
    li a0, 2147491840
    sw a1, 0(a0)
    j if_0_end
if_0_end:
    j loop_0_condition
loop_0_fail:
    ret
