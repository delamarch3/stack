.entry main

main:
    ret ; no return value

func:
    push   100 ; 32 (1 unit)
    pushd 100 ; 64 (2 units)
    pushb 8   ; 8  (1 unit)

    ; stack size is 16

    storeb 0
    stored 1  ; dword takes up two units of space, hence we skipped 2
    store   3

    loadd 1
    loadd 1
    addd

    ; cmp will still produce a 1 unit value
    cmp.d 0
    ; jmp will still consume a 1 unit value
    jmp.lt func

    load   3
    loadd 1
    loadb 0

    ret.b
