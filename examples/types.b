.entry main

main:
    ret ; no return value

func:
    push   100
    push.d 100
    push.b 8

    ; stack size is 13

    store.b 0
    store.d 1
    store   3 ; dword takes up two units of space, hence we skipped 2

    load.d 1
    load.d 1
    add.d

    ; cmp will still produce a 1 unit value
    cmp.d 0
    ; jmp will still consume a 1 unit value
    jmp.lt func

    load   3
    load.d 1
    load.b 0

    ret.b
