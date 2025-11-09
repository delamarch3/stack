.entry main

#include "examples/io_include.b"

.data message .string "Hello, World!\n"

main:
    dataptr message
    push.d sizeof message
    call print
    ret
