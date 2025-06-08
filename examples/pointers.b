.entry main

; TODO: add types and constants, along with instructions to access
; fields within a referenced record
.data myRecord      ; myRecord resolves to a pointer
    .dword 100      ; 64
    .word 100       ; 32
    .byte 'a'       ; 8
    .ascii "string"

.data myNumber: .int 100

; pushing a label on to the stack pushes a pointer to it
; get will consume and push a value at an offset
main:
    push myNumber       ; push pointer to myNumber on to the stack
    get 0               ; consume and dereference pointer and push 100
    push myRecord       ; push pointer to myRecord on to the stack
    get.b 12            ; consume and dereference pointer to push byte at offset 12

    push myRecord
    push 20
    add                 ; push a pointer to the string
    store.d 0
    load.d 0
    get.b 2             ; push 'r' on to the stack

    ret
