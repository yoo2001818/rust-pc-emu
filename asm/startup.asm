bits 16
org 0xffff
; Do long jump to 7c00:0000
; Replace CS to 0x7c00, IP to 0x0000
jmp 0x7c00:0x0000
