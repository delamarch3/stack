.entry main

.data ptr .dword
.data input .word 9

.data str .string "abc"
.data end .byte 0

main:
    push.d str
    store.d 0

    push 0
    store 2
loop:
    ; break if c == 0
    load.d 0
    get.b 0
    dup
    cmp 0
    jmp.eq done

    ; local[2] += c
    load 2
    add
    store 2

    ; str += 1
    load.d 0
    push.d 1
    add.d
    store.d 0
    jmp loop

done:
    pop
    load 2
    ret
