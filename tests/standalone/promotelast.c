
#include<stdio.h>
struct y {int a;int b;char c;};//passed by 2 regs
int f4(struct y p2) {
    //p2.c should be promoted to i32, which it was, then as it was originally a byte, and because it is a varadic arg, the byte type was assumed, which was incorrect
    //this is because 2 parts of the code tried to calculate the type of a varadic arg, and only the last one was doing it correctly
    printf("p2.a=%d, p2.b=%d, p2.c=%d\n",
        p2.a, p2.b, p2.c);
    return !(p2.a == 1 && p2.b == 2 && p2.c == 3);
}

int main() {
    struct y bar;
    bar.a = 1;
    bar.b = 2;
    bar.c = 3;

    
    if(f4(bar)) {
        return 4;
    }
    return 0;
}