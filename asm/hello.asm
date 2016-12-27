bits 16
org 0x7c00

jmp main

Message db "Hello, world!", 0x0A, 0x0

main:
  mov dx, 0x0
  mov si, Message

loopString:
  mov al, [si]
  ; Test 0 value
  test al, al
  jz exitString
  inc si
  ; Print the character (Since we don't have BIOS or something, just export
  ; the number to port 0)
  out dx, al
  jmp loopString

exitString:
  cli
  hlt
