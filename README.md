# Stack

`stack` is a stack-machine bytecode interpreter. It shares some similarities with the JVM.

## Instruction Set

The `stack` instruction mnemonics are specified in `./src/assembler.rs`, inside `Assembler::assemble_instruction()`.

The operators can be grouped together by behaviour:

* The operator manipulates values existing on the stack. For example, `add` will pop two values, add them, then push the result.
* The operator pushes new values onto the stack. For example, `load 0` will push the 0th local onto the stack.
* The operator manipulates frames on the call stack. For example, `call` and `ret` will push and pop frames respectively.
* The operator modifies the `pc` (program counter). For example, `jmp label` will unconditionally update the `pc` to point at `label`.

## Frames

When the interpreter starts, it bumps the `pc` to the label pointed at by the `.entry` directive at the start of the source file. It then pushes the first frame, referred to as `main`, onto the call stack. Each time a `call` instruction is encountered, the operand stack is cleared out and copied into the locals array of a newly created frame. The new frame is then pushed onto the call stack as the `pc` is updated. The `ret` instruction will pop off a frame from the call stack, returning the `pc` to it's old position, unless it's the `main` frame, in which case the program will end.

Each frame contains:

* Operand stack - Similar in functionality to registers on a CPU, this is where values are operated upon.
* Locals array - Variables can be stored and loaded when needed, using the `load` and `store` instructions.
* Shared heap reference - Objects and buffers are allocated into the heap. Their lifetime is managed with the `alloc` and `free` instructions.

## Values

Values on the operand stack or the locals array occupy "slots". These slots are four bytes in length. To operate on values of different length, different variants of some instructions are provided. For example, `load.d 0` will push the eight bytes occupying slots 0 and 1 of the locals array. Similarly, `ret.d` will pop two slots off the operand stack and push into the caller's.

## Debugger

The debugger has a few features at the moment, including but not limited to:

* Step through the program with `s` or `\n`.
* Set breakpoints with `b <label/offset>`
* Continue to a breakpoint with `c`
* View the disassembly with `dis`
* View a local variable with `v <slot idx>`
* View the backtrace with `bt`

The full list of commands can be found in `./src/bin/sdb.rs`, inside `parse_command()`.

## Examples

`./examples/` contains some simple programs. To try them out, run:

```console
# Assemble the program
$ cargo r --bin stackc examples/<example-file>
# Run the interpreter
$ cargo r --bin stack a.out
# Optional - debug the program
$ cargo r --bin sdb a.out
```

### Add Two Numbers

```
.entry add

add:
    push 0
    push 1
    add
    ret
```

### Nth Fibonacci

```
.entry main

main:
    push 8
    call fib
    ret

; fib(n)
fib:
    ; if n < 2, return n
    load 0
    push 2
    cmp
    jmp.lt base

    ; t1 = fib(n - 1)
    load 0
    push 1
    sub
    call fib
    store 1

    ; t2 = fib(n - 2)
    load 0
    push 2
    sub
    call fib
    store 2

    ; return t1 + t2
    load 1
    load 2
    add
    ret.w

base:
    load 0
    ret.w
```

### Print "Hello, World!"

```
.entry main

#include "examples/io_include.b"

.data message .string "Hello, World!\n"
#define MESSAGESZ { 14 }

main:
    dataptr message
    push.d @MESSAGESZ
    call print
    ret
```
