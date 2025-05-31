.entry main

main:
    push 22
    store 0
    push 55
    store 1
    load 0
    load 1
    add
    store 2
    ret
