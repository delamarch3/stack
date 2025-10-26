.entry main

#include "examples/system.b"

.data message .string "Hello, World!\n"
#define MESSAGESZ { 14 }

.data file .string "text.txt\0"

main:
    push.d 64
    alloc
    dup.d
    dup.d
    store.d 0 ; buf

               ; dup ptr
    push.d 0   ; offset
    push.b 'A' ; data
    astore

                ; dup ptr
    push.d 1    ; offset
    push.b '\n' ; data
    astore

    push 1    ; stdout
    load.d 0  ; buf
    push.d 2  ; size
    push 4    ; write
    system
    pop

    dataptr message
    push.d @MESSAGESZ
    call print

    ret
