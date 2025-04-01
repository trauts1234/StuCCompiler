struct x {int a;};//passed by 1 reg

extern int f1(int p1, struct x p2);//test simple struct pass by value

extern int f2(int p1, struct x *p2_ptr);//test pass by pointer

extern int f3(int p1, struct x p2, int p3, int p4, int p5, int p6, int p7);//struct gets passed by register

struct y {int a;int b;char c;};//passed by 2 regs

extern int f4(int p1, struct y p2, int p3, int p4, int p5, int p6, int p7); // uses 2 registers

extern int f5(int p1, int p2, int p3, int p4, int p5, struct y p6, int p7); // cannot fit in 1 reg, so goes on stack, and p7 goes in reg

struct z {long int a;int b;long int c;};//does not fit in registers

extern int f6(struct z p1, int p2);

int main() {
    struct x foo;
    foo.a = 1;
    struct y bar;
    bar.a = 1;
    bar.b = 2;
    bar.c = 3;
    struct z baz;
    baz.a = 1;
    baz.b = 2;
    baz.c = 3;

    if(f1(1,foo)) {
        return 1;
    }
    if(f2(1, &foo)) {
        return 2;
    }
    if(f3(1, foo, 3, 4, 5, 6, 7)) {
        return 3;
    }
    if(f4(1,bar, 3, 4, 5, 6, 7)) {
        return 4;
    }
    
    if(f5(1,2,3,4,5,bar,7)) {
        return 5;
    }
    if(f6(baz, 2)) {
        return 6;
    }
    return 0;
}