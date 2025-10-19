.entry main

main:
    push 10 ; a
    push 11 ; b
    cmp
    jmp.lt a1
    push 0
    jmp a2
a1:
    push 1
    jmp a2
a2:
    store 0 ; x = a op b

    load 0
    ret
