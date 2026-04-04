.entry main

main:
    push.d 64
    push.d 8
    mul.d
    alloc     ; allocate a 512 byte buffer
    store.d 0

    push 64
    load.d 0
    astore

    load.d 0
    push.d 0
    aload

    load.d 0
    free

    ret
