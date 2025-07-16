#include <stdio.h>

// Original structs
struct x {int a;}; // 4 bytes - passed by 1 reg
struct y {int a; int b; char c;}; // 12 bytes - passed by 2 regs  
struct z {long int a; int b; long int c;}; // 24 bytes - passed by memory

// Additional test structs
struct small_char {char a;}; // 1 byte - passed by 1 reg
struct two_chars {char a; char b;}; // 2 bytes - passed by 1 reg
struct eight_bytes {long a;}; // 8 bytes - passed by 1 reg
struct sixteen_bytes {long a; long b;}; // 16 bytes - passed by 2 regs
struct seventeen_bytes {long a; long b; char c;}; // 17 bytes - passed by memory
struct mixed_alignment {char a; int b; char c;}; // tests padding/alignment
struct float_struct {float a; float b;}; // floating point in structs
struct double_struct {double a;}; // 8-byte float
struct mixed_float {int a; float b; double c;}; // mixed int/float
struct nested {struct x inner; int outer;}; // nested structs

// Array tests
struct small_array {int arr[2];}; // 8 bytes
struct large_array {int arr[10];}; // 40 bytes - should go to memory

// Original function declarations
extern int f1(int p1, struct x p2);
extern int f2(int p1, struct x *p2_ptr);
extern int f3(int p1, struct x p2, int p3, int p4, int p5, int p6, int p7);
extern int f4(int p1, struct y p2, int p3, int p4, int p5, int p6, int p7);
extern int f5(int p1, int p2, int p3, int p4, int p5, struct y p6, int p7);
extern int f6(struct z p1, int p2);
extern int f7(struct y p1);

// New function declarations
extern int f8(struct small_char p1, int p2);
extern int f9(struct two_chars p1, int p2);
extern int f10(struct eight_bytes p1, int p2);
extern int f11(struct sixteen_bytes p1, int p2);
extern int f12(struct seventeen_bytes p1, int p2);
extern int f13(struct mixed_alignment p1, int p2);
extern int f14(struct float_struct p1, int p2);
extern int f15(struct double_struct p1, int p2);
extern int f16(struct mixed_float p1, int p2);
extern int f17(struct nested p1, int p2);
extern int f20(struct small_array p1, int p2);
extern int f21(struct large_array p1, int p2);

// Floating point parameter tests
extern int f22(float f1, struct float_struct s1, double d1);
extern int f23(double d1, double d2, double d3, double d4, double d5, double d6, double d7, double d8, struct double_struct s1); // exhaust FP regs

// Variadic function tests
extern int f29(int count, ...); // variadic with structs

int main() {
    printf("Starting ABI compliance tests...\n");
    
    // Original tests
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
    
    if(f1(1, foo)) return 1;
    if(f2(1, &foo)) return 2;
    if(f3(1, foo, 3, 4, 5, 6, 7)) return 3;
    if(f4(1, bar, 3, 4, 5, 6, 7)) return 4;
    if(f5(1, 2, 3, 4, 5, bar, 7)) return 5;
    if(f6(baz, 2)) return 6;
    if(f7(bar)) return 7;
    
    // New tests
    struct small_char sc;
    sc.a = 42;
    if(f8(sc, 100)) return 8;
    
    struct two_chars tc;
    tc.a = 65;
    tc.b = 66;
    if(f9(tc, 100)) return 9;
    
    struct eight_bytes eb;
    eb.a = 0x123456789ABCDEF0L;
    if(f10(eb, 100)) return 10;
    
    struct sixteen_bytes sb;
    sb.a = 0x1111111111111111L;
    sb.b = 0x2222222222222222L;
    if(f11(sb, 100)) return 11;
    
    struct seventeen_bytes svb;
    svb.a = 0x1111111111111111L;
    svb.b = 0x2222222222222222L;
    svb.c = 99;
    if(f12(svb, 100)) return 12;
    
    struct mixed_alignment ma;
    ma.a = 10;
    ma.b = 20;
    ma.c = 30;
    if(f13(ma, 100)) return 13;
    
    struct float_struct fs;
    fs.a = 3.14f;
    fs.b = 2.71f;
    if(f14(fs, 100)) return 14;
    
    struct double_struct ds;
    ds.a = 3.14159;
    if(f15(ds, 100)) return 15;
    
    struct mixed_float mf;
    mf.a = 42;
    mf.b = 3.14f;
    mf.c = 2.71828;
    if(f16(mf, 100)) return 16;
    
    struct nested n;
    n.inner.a = 99;
    n.outer = 88;
    if(f17(n, 100)) return 17;
    
    struct small_array sa;
    sa.arr[0] = 10;
    sa.arr[1] = 20;
    if(f20(sa, 100)) return 20;
    
    struct large_array la;
    la.arr[0] = 1;
    la.arr[1] = 2;
    la.arr[2] = 3;
    la.arr[3] = 4;
    la.arr[4] = 5;
    la.arr[5] = 6;
    la.arr[6] = 7;
    la.arr[7] = 8;
    la.arr[8] = 9;
    la.arr[9] = 10;
    if(f21(la, 100)) return 21;
    
    if(f22(1.5f, fs, 2.5)) return 22;
    if(f23(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, ds)) return 23;
    
    // Variadic test
    if(f29(3, foo, bar, baz)) return 29;
    
    printf("All tests passed!\n");
    return 0;
}