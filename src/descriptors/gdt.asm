[bits 64]
load:
    lgdt [rdi]
    retfq

GLOBAL load