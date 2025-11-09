.entry main

#include "examples/io_include.b"

.data readerror .string "error: could not read\n"
#define READERRORSZ 22

#define BUFSZ 512

main:
    push.d @BUFSZ
    alloc
    store.d 0

    push @STDIN
    load.d 0
    push.d @BUFSZ
    push @READ
    system
    push -1
    cmp
    jmp.eq read_error

    load.d 0
    push.d @BUFSZ
    call print

    ret

read_error:
    dataptr readerror
    push.d @READERRORSZ
    call print
    ret
