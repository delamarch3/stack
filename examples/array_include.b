#define ARRAY_DEFAULT_CAP 16

array_new:
    push.d 24
    alloc
    store.d 0

    push.d -1 ; ptr value, not allocated until an item is pushed
    load.d 0
    push.d 0 ; ptr offset
    astore.d

    push.d 0 ; len value
    load.d 0
    push.d 8 ; len offset
    astore.d

    push.d @ARRAY_DEFAULT_CAP ; cap value
    load.d 0
    push.d 16                 ; cap offset
    astore.d

    load.d 0
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
    load.d 16
    load.d 0
    push.d 0
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
    load.b 2  ; val
    load.d 16 ; ptr
    load.d 18 ; len
    astore.b

    ; self.len++
    load.d 18 ; len
    push.d 1
    add.d
    load.d 0  ; self
    push.d 8
    astore.d

    ret
