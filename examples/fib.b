.entry main

main:
    push 8
    call fib
    ret

; fib(n)
fib:
    ; if n < 2, return n
    load 0
    push 2
    cmp
    jmp.lt base

    ; t1 = fib(n - 1)
    load 0
    push 1
    sub
    call fib
    store 1

    ; t2 = fib(n - 2)
    load 0
    push 2
    sub
    call fib
    store 2

    ; return t1 + t2
    load 1
    load 2
    add
    ret.w

base:
    load 0
    ret.w
