.entry main

#include "examples/system.b"

.data message .string "Hello, World!" .byte '\n'
#define MESSAGESZ { 14 }

.data file .string "text.txt" .byte '\0'

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

    dataptr message
    push.d @MESSAGESZ
    call print

    ret
