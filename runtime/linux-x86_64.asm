; https://en.wikipedia.org/wiki/Crt0

global _start
extern main

section .text

_start:
    ; stack should be aligned
    ; https://stackoverflow.com/questions/39234911/after-entering-start-is-rsp-aligned

    xor rbp, rbp     ; clear rbp
    mov rdi, [rsp]   ; get argc from stack
    lea rsi, [rsp+8] ; get address of argv from stack

    ; rdi, rsi -> rdx
    call main

    mov rdi, rax ; exit code
    mov rax, 60 ; syscall number
    syscall
