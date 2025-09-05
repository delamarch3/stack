.entry main

main:
    push.d 64
    push.d 8
    mul.d
    alloc     ; allocate a 512 byte buffer
    dup.d
    store.d 0

    push.d 0
    push 64
    write

    load.d 0
    push.d 0
    read

    ret
