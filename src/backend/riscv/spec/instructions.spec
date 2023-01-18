lui         {{params[1] | bits_at(0, 20)}}{{params[0] | register}}0110111
auipc       {{params[1] | bits_at(0, 20)}}{{params[0] | register}}0010111
jal         {{params[1] | jal_form}}{{params[0] | register}}1101111
jalr        {{params[1] | bits_at(0, 12)}}{{params[2] | register}}000{{params[0] | register}}1100111
beq         {{params[2] | branch_high}}{{params[1] | register}}{{params[0] | register}}000{{params[2] | branch_low}}1100011
bne         {{params[2] | branch_high}}{{params[1] | register}}{{params[0] | register}}001{{params[2] | branch_low}}1100011
blt         {{params[2] | branch_high}}{{params[1] | register}}{{params[0] | register}}100{{params[2] | branch_low}}1100011
bge         {{params[2] | branch_high}}{{params[1] | register}}{{params[0] | register}}101{{params[2] | branch_low}}1100011
bltu        {{params[2] | branch_high}}{{params[1] | register}}{{params[0] | register}}110{{params[2] | branch_low}}1100011
bgeu        {{params[2] | branch_high}}{{params[1] | register}}{{params[0] | register}}111{{params[2] | branch_low}}1100011
lb          {{params[1] | bits_at(0, 12)}}{{params[2] | register}}000{{params[0] | register}}0000011
lh          {{params[1] | bits_at(0, 12)}}{{params[2] | register}}001{{params[0] | register}}0000011
lw          {{params[1] | bits_at(0, 12)}}{{params[2] | register}}010{{params[0] | register}}0000011
lbu         {{params[1] | bits_at(0, 12)}}{{params[2] | register}}100{{params[0] | register}}0000011
lhu         {{params[1] | bits_at(0, 12)}}{{params[2] | register}}101{{params[0] | register}}0000011
sb          {{params[1] | bits_at(5, 12)}}{{params[0] | register}}{{params[2] | register}}000{{params[1] | bits_at(0, 5)}}0100011
sh          {{params[1] | bits_at(5, 12)}}{{params[0] | register}}{{params[2] | register}}001{{params[1] | bits_at(0, 5)}}0100011
sw          {{params[1] | bits_at(5, 12)}}{{params[0] | register}}{{params[2] | register}}010{{params[1] | bits_at(0, 5)}}0100011
addi        {{params[2] | bits_at(0, 12)}}{{params[1] | register}}000{{params[0] | register}}0010011
slti        {{params[2] | bits_at(0, 12)}}{{params[1] | register}}010{{params[0] | register}}0010011
sltiu       {{params[2] | bits_at(0, 12)}}{{params[1] | register}}011{{params[0] | register}}0010011
xori        {{params[2] | bits_at(0, 12)}}{{params[1] | register}}100{{params[0] | register}}0010011
ori         {{params[2] | bits_at(0, 12)}}{{params[1] | register}}110{{params[0] | register}}0010011
andi        {{params[2] | bits_at(0, 12)}}{{params[1] | register}}111{{params[0] | register}}0010011
slli        0000000{{params[2] | bits_at(0, 5)}}{{params[1] | register}}001{{params[0] | register}}0010011
srli        0000000{{params[2] | bits_at(0, 5)}}{{params[1] | register}}101{{params[0] | register}}0010011
srai        0100000{{params[2] | bits_at(0, 5)}}{{params[1] | register}}101{{params[0] | register}}0010011
add         0000000{{params[2] | register}}{{params[1] | register}}000{{params[0] | register}}0110011
sub         0100000{{params[2] | register}}{{params[1] | register}}000{{params[0] | register}}0110011
sll         0000000{{params[2] | register}}{{params[1] | register}}001{{params[0] | register}}0110011
slt         0000000{{params[2] | register}}{{params[1] | register}}010{{params[0] | register}}0110011
sltu        0000000{{params[2] | register}}{{params[1] | register}}011{{params[0] | register}}0110011
xor         0000000{{params[2] | register}}{{params[1] | register}}100{{params[0] | register}}0110011
srl         0000000{{params[2] | register}}{{params[1] | register}}101{{params[0] | register}}0110011
sra         0100000{{params[2] | register}}{{params[1] | register}}101{{params[0] | register}}0110011
or          0000000{{params[2] | register}}{{params[1] | register}}110{{params[0] | register}}0110011
and         0000000{{params[2] | register}}{{params[1] | register}}111{{params[0] | register}}0110011
csrrw       {{params[1] | csr}}{{params[2] | register}}001{{params[0] | register}}1110011
csrrs       {{params[1] | csr}}{{params[2] | register}}010{{params[0] | register}}1110011
csrrc       {{params[1] | csr}}{{params[2] | register}}011{{params[0] | register}}1110011
csrrwi      {{params[1] | csr}}{{params[2] | bits_at(0, 5)}}101{{params[0] | register}}1110011
csrrsi      {{params[1] | csr}}{{params[2] | bits_at(0, 5)}}110{{params[0] | register}}1110011
csrrci      {{params[1] | csr}}{{params[2] | bits_at(0, 5)}}111{{params[0] | register}}1110011
