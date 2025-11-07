#define ARRAY_DEFAULT_CAP 16

array_new:
    push.d 24
    alloc

    dup.d
    push.d 0 ; ptr offset
    push.d -1 ; ptr value, not allocated until an item is pushed
    astore.d

    dup.d
    push.d 8 ; len offset
    push.d 0 ; len value
    astore.d

    dup.d
    push.d 16                 ; cap offset
    push.d @ARRAY_DEFAULT_CAP ; cap value
    astore.d

    ret.d

array_realloc:
    panic

array_push_byte:
    load.d 0 ; self

    ; Load the pointer
    dup.d
    push.d 0
    aload.d    ; ptr
    dup.d
    store.d 16

    ; Allocate if the pointer is -1
    push.d -1
    cmp.d
    jmp.ne array_push_byte_noalloc
    push.d @ARRAY_DEFAULT_CAP
    alloc
    store.d 16

    ; Store the allocated pointer
    load.d 0
    push.d 0
    load.d 16
    astore.d

array_push_byte_noalloc:
    dup.d
    push.d 8
    aload.d    ; len
    store.d 18

    dup.d
    push.d 16
    aload.d    ; cap
    store.d 20

    ; Reallocate if len == cap
    load.d 18 ; len
    load.d 20 ; cap
    cmp.d
    jmp.eq array_push_byte_realloc
    jmp array_push_byte_norealloc
array_push_byte_realloc:
    call array_realloc
array_push_byte_norealloc:

    ; self.ptr[len] = val
    load.d 16 ; ptr
    load.d 18 ; len
    load.b 2  ; val
    astore.b

    ; self.len++
    load.d 0  ; self
    push.d 8
    load.d 18 ; len
    push.d 1
    add.d
    astore.d

    ret
