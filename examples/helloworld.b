.entry main

#include "examples/io_include.b"

.data message .string "Hello, World!\n"
#define MESSAGESZ 14

main:
    dataptr message
    push.d @MESSAGESZ
    call print
    ret
