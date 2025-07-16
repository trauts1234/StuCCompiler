#include <stdarg.h>
#include <math.h>

// Struct definitions (must match program A)
struct x {int a;};
struct y {int a; int b; char c;};
struct z {long int a; int b; long int c;};
struct small_char {char a;};
struct two_chars {char a; char b;};
struct eight_bytes {long a;};
struct sixteen_bytes {long a; long b;};
struct seventeen_bytes {long a; long b; char c;};
struct mixed_alignment {char a; int b; char c;};
struct float_struct {float a; float b;};
struct double_struct {double a;};
struct mixed_float {int a; float b; double c;};
struct nested {struct x inner; int outer;};
struct small_array {int arr[2];};
struct large_array {int arr[10];};

// Helper function to compare floats
static int float_equals(float a, float b) {
    return fabsf(a - b) < 0.0001f;
}

static int double_equals(double a, double b) {
    return fabs(a - b) < 0.0001;
}

// Original functions
int f1(int p1, struct x p2) {
    return !(p1 == 1 && p2.a == 1);
}

int f2(int p1, struct x *p2_ptr) {
    return !(p1 == 1 && p2_ptr && (*p2_ptr).a == 1);
}

int f3(int p1, struct x p2, int p3, int p4, int p5, int p6, int p7) {
    return !(p1 == 1 && p2.a == 1 && p3 == 3 && p4 == 4 && p5 == 5 && p6 == 6 && p7 == 7);
}

int f4(int p1, struct y p2, int p3, int p4, int p5, int p6, int p7) {
    return !(p1 == 1 && p2.a == 1 && p2.b == 2 && p2.c == 3 && p3 == 3 && p4 == 4 && p5 == 5 && p6 == 6 && p7 == 7);
}

int f5(int p1, int p2, int p3, int p4, int p5, struct y p6, int p7) {
    return !(p1 == 1 && p2 == 2 && p3 == 3 && p4 == 4 && p5 == 5 && p6.a == 1 && p6.b == 2 && p6.c == 3 && p7 == 7);
}

int f6(struct z p1, int p2) {
    return !(p1.a == 1 && p1.b == 2 && p1.c == 3 && p2 == 2);
}

int f7(struct y p1) {
    return !(p1.a == 1 && p1.b == 2 && p1.c == 3);
}

// New functions
int f8(struct small_char p1, int p2) {
    return !(p1.a == 42 && p2 == 100);
}

int f9(struct two_chars p1, int p2) {
    return !(p1.a == 65 && p1.b == 66 && p2 == 100);
}

int f10(struct eight_bytes p1, int p2) {
    return !(p1.a == 0x123456789ABCDEF0L && p2 == 100);
}

int f11(struct sixteen_bytes p1, int p2) {
    return !(p1.a == 0x1111111111111111L && p1.b == 0x2222222222222222L && p2 == 100);
}

int f12(struct seventeen_bytes p1, int p2) {
    return !(p1.a == 0x1111111111111111L && p1.b == 0x2222222222222222L && p1.c == 99 && p2 == 100);
}

int f13(struct mixed_alignment p1, int p2) {
    return !(p1.a == 10 && p1.b == 20 && p1.c == 30 && p2 == 100);
}

int f14(struct float_struct p1, int p2) {
    return !(float_equals(p1.a, 3.14f) && float_equals(p1.b, 2.71f) && p2 == 100);
}

int f15(struct double_struct p1, int p2) {
    return !(double_equals(p1.a, 3.14159) && p2 == 100);
}

int f16(struct mixed_float p1, int p2) {
    return !(p1.a == 42 && float_equals(p1.b, 3.14f) && double_equals(p1.c, 2.71828) && p2 == 100);
}

int f17(struct nested p1, int p2) {
    return !(p1.inner.a == 99 && p1.outer == 88 && p2 == 100);
}

int f20(struct small_array p1, int p2) {
    return !(p1.arr[0] == 10 && p1.arr[1] == 20 && p2 == 100);
}

int f21(struct large_array p1, int p2) {
    int expected[] = {1,2,3,4,5,6,7,8,9,10};
    for(int i = 0; i < 10; i++) {
        if(p1.arr[i] != expected[i]) return 1;
    }
    return !(p2 == 100);
}

int f22(float f1, struct float_struct s1, double d1) {
    return !(float_equals(f1, 1.5f) && float_equals(s1.a, 3.14f) && float_equals(s1.b, 2.71f) && double_equals(d1, 2.5));
}

int f23(double d1, double d2, double d3, double d4, double d5, double d6, double d7, double d8, struct double_struct s1) {
    return !(double_equals(d1, 1.0) && double_equals(d2, 2.0) && double_equals(d3, 3.0) && 
             double_equals(d4, 4.0) && double_equals(d5, 5.0) && double_equals(d6, 6.0) && 
             double_equals(d7, 7.0) && double_equals(d8, 8.0) && double_equals(s1.a, 3.14159));
}

// Variadic function test
int f29(int count, ...) {
    va_list args;
    va_start(args, count);
    
    if(count != 3) return 1;
    
    struct x x_arg = va_arg(args, struct x);
    if(x_arg.a != 1) return 1;
    
    struct y y_arg = va_arg(args, struct y);
    if(y_arg.a != 1 || y_arg.b != 2 || y_arg.c != 3) return 1;
    
    struct z z_arg = va_arg(args, struct z);
    if(z_arg.a != 1 || z_arg.b != 2 || z_arg.c != 3) return 1;
    
    va_end(args);
    return 0;
}