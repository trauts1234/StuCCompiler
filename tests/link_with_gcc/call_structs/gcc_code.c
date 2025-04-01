struct x { int a; };
struct y { int a; int b; char c; };
struct z {long int a;int b;long int c;};

int f1(int p1, struct x p2) {
    return (p1 == 1 && p2.a == 1) ? 0 : 1;
}

int f2(int p1, struct x *p2_ptr) {
    return (p1 == 1 && p2_ptr && p2_ptr->a == 1) ? 0 : 1;
}

int f3(int p1, struct x p2, int p3, int p4, int p5, int p6, int p7) {
    return (p1 == 1 && p2.a == 1 && p3 == 3 && p4 == 4 && p5 == 5 && p6 == 6 && p7 == 7) ? 0 : 1;
}

int f4(int p1, struct y p2, int p3, int p4, int p5, int p6, int p7) {
    return (p1 == 1 && p2.a == 1 && p2.b == 2 && p2.c == 3 && p3 == 3 && p4 == 4 && p5 == 5 && p6 == 6 && p7 == 7) ? 0 : 1;
}

int f5(int p1, int p2, int p3, int p4, int p5, struct y p6, int p7) {
    return (p1 == 1 && p2 == 2 && p3 == 3 && p4 == 4 && p5 == 5 && p6.a == 1 && p6.b == 2 && p6.c == 3 && p7 == 7) ? 0 : 1;
}

int f6(struct z p1, int p2){
    return (p1.a == 1 && p1.b == 2 && p1.c == 3 && p2 == 2) ? 0 : 1;
}