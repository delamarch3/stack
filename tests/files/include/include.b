#define STDIN  0
#define STDOUT 1
#define STDERR 2

#define EXIT 1
#define READ 3
#define WRITE 4
#define OPEN 5
#define CLOSE 6
#define FSYNC 95

print:
    push @STDOUT
    load.d 0
    load.d 2
    push @WRITE
    system
    ret.w
