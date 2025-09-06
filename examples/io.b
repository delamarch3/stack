.entry main

.data message .string "Hello, World!" .byte '\n'

.data file .string "text.txt" .byte '\0'

.data RDRW .word 2

.data STDOUT .word 1
.data WRITE .word 4

main:
    push.d 64
    alloc
    dup.d
    store.d 0 ; buf

              ; dup id
    push.d 0  ; offset
    push.b 65 ; data
    write

    push 1    ; stdout
    load.d 0  ; buf
    ptr
    push.d 1  ; size
    push 4    ; write
    system
    pop

    push 1          ; stdout
    dataptr message ; buf
    push.d 14       ; size
    push 4          ; write
    system
    pop

    ret
