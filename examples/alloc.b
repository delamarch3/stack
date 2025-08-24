.entry main

main:
    push.d 64
    push.d 8
    mul
    alloc     ; allocate a 512 byte buffer
    dup.d
    store.d 0
    push.d 0
    get       ; get the 32 bit integer at position 0
    pop
    store.d 0
    push.d 64
    get.d     ; get the 64 bit integer at position 64, etc
    pop.d

    ; Write to the buffer in a loop
    load.d 0
    store.d 2 ; copy the pointer

    push 97 ; 'a'
    dup
    dup
loop:
    ; Write the char
    load.d 2
    push 97 ; todo: should be local
    dup
    write.b
    ; Increment the pointer
    load.d 2
    push.d 1
    add
    store.d 2
    ; Compare with 'z'
    push 122
    cmp
    jmp.gt end
    ; Increment the char
    push 1
    add
    dup
    dup
end:
    ret
