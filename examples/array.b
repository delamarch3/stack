.entry main

#include "examples/array_include.b"
#include "examples/system.b"

main:
    call array_new
    store.d 0

    load.d 0
    push 65
    call array_push_byte

    load.d 0
    push 66
    call array_push_byte

    load.d 0
    push 67
    call array_push_byte

    load.d 0
    push 10
    call array_push_byte

    load.d 0
    push.d 0
    aload.d
    ptr
    push.d 4
    call print

    ret
