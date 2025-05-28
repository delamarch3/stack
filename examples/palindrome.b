.entry main

main:
    push 3
    push 2
    push 1

    dup
    cmp 1
    jmp.ne fail
    pop
    dup
    cmp 2
    jmp.ne fail
    pop
    dup
    cmp 3
    jmp.ne fail
    pop
    ret


fail:
    fail
