.entry main

main:
    push 3
    push 2
    push 1

    cmp 1
    jmp.ne fail
    pop
    cmp 2
    jmp.ne fail
    pop
    cmp 3
    jmp.ne fail

    jmp success

fail:
    fail
success:
    ret
