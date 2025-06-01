.entry main

main:
    push 22
    push 33
    call add ; local0 = 22, local1 = 33
    store 0
    ret

add:
   load 0
   load 1
   add
   ret
