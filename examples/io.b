.entry main

.data message .str "Hello, World!" .byte '\n'

.data file .str "text.txt" .byte '\0'

.data RDRW .word 2

main:
    push 1          ; stdout
    dataptr message
    push 14         ; message.len
    call write

    dataptr file
    push RDRW
    call open

    ret
