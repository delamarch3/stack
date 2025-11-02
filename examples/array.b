.entry main

#include "examples/array_include.b"
#include "examples/io_include.b"

main:
    call array_new
    store.d 0

    load.d 0
    push 'A'
    call array_push_byte

    load.d 0
    push 'B'
    call array_push_byte

    load.d 0
    push 'C'
    call array_push_byte

    load.d 0
    push '\n'
    call array_push_byte

    load.d 0
    push '\n'
    call array_push_byte

    load.d 0
    push.d 0
    aload.d
    push.d 5
    call print

    ret
