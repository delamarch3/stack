.entry main

main:
    push 8
    call fib
    ret

fib:
    load 0
    cmp 1
    jmp.lt base0
    load 0
    cmp 2
    jmp.lt base1

    load 0
    push 1
    sub
    call fib
    store 1

    load 0
    push 2
    sub
    call fib
    store 2

    load 1
    load 2
    add
    ret

base0:
    push 0
    ret
base1:
    push 1
    ret
