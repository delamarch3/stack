.entry main

main:
    push 8
    call fib
    ret

fib:
    load 0
    push 2
    cmp
    jmp.lt base

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

base:
    load 0
    ret
