#define STDIN  { 0 }
#define STDOUT { 1 }
#define STDERR { 2 }

#define WRITE  { 4 }

print:
    push @STDOUT
    load.d 0
    load.d 2
    push @WRITE
    system
    ret
