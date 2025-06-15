.entry main

.data list .byte 1,2,3,'b',0

main:
    push.d list
    get.b 3
    ret
