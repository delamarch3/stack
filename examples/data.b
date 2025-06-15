.entry main

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
    dup.d
    get.b 0
    dup
    push 0
    cmp
    jmp.eq done

    ; result += c
    load 2
    add
    store 2

    ; str += 1
    push.d 1
    add.d
    store.d 0
    jmp loop

done:
    pop
    pop.d
    load 2
    ret
