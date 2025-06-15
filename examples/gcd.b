.entry main

main:
    push 18  ; x
    push 30  ; y
    call gcd
    ret

gcd:
    load 0  ; x
    load 1  ; y
    cmps
    jmp.eq gcd_done

    load 0
    load 1
    cmps
    jmp.gt gcd_gt
gcd_le:
    load 1
    load 0
    sub
    store 1
    jmp gcd
gcd_gt:
    load 0
    load 1
    sub
    store 0
    jmp gcd
gcd_done:
    load 0
    ret
