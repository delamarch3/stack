.entry main

main:
    ; Push 5, 4, 3, 2, 1 on to the stack
    push 5
loop:
    dup
    push 1
    sub
    dup
    cmp 1
    jmp.eq done
    jmp loop
done:
    ret
