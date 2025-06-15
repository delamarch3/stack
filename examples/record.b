.entry main

.data record
    .byte 1,2,3,'b',0
    .string "Hello, World!"
    .word
    .word 5

main:
    push.d record
    push.d 0
    get.b         ; 1
    push.d record
    push.d 5
    get.b         ; H
    push.d record
    push.d 18
    get           ; 0
    push.d record
    push.d 22
    get           ; 5
    ret
