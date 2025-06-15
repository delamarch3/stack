.entry main

.data record
    .byte 1,2,3,'b',0
    .string "Hello, World!"
    .word
    .word 5

main:
    push.d record
    get.b 0 ; 1
    push.d record
    get.b 5 ; H
    push.d record
    get 18  ; 0
    push.d record
    get 22  ; 5
    ret
