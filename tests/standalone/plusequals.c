#include <stdio.h>
#include <stdint.h>
#include <float.h>
#include <math.h>
#include <string.h>

// Test result tracking
static int test_count = 0;
static int failed_tests = 0;

void test_assert(int condition, const char* test_name) {
    if (!condition) {
        printf("FAIL: %s (test %d)\n", test_name, test_count);
        failed_tests++;
    } else {
        printf("PASS: %s (test %d)\n", test_name, test_count);
    }
}

// Float comparison with epsilon for floating point tests
_Bool float_eq(float a, float b) {return fabs((a) - (b)) < 1e-6;}
_Bool double_eq(double a, double b) {return fabs((a) - (b)) < 1e-12;}

// Test basic integer types
void test_integer_types() {
    
    
    // char
    char c = 10;
    c += 5;
    test_assert(c == 15, "char += positive");
    
    c += -3;
    test_assert(c == 12, "char += negative");
    
    // short
    short s = 1000;
    s += 2000;
    test_assert(s == 3000, "short += positive");
    
    // int
    int i = 42;
    i += 58;
    test_assert(i == 100, "int += positive");
    
    i += -50;
    test_assert(i == 50, "int += negative");
    
    // long
    long l = 1000000L;
    l += 2000000L;
    test_assert(l == 3000000L, "long += positive");
    
    // long long
    long long ll = 9223372036854775800LL;
    ll += 7;
    test_assert(ll == 9223372036854775807LL, "long long += near max");
}

// Test unsigned integer types
void test_unsigned_types() {
    
    
    // unsigned char
    unsigned char uc = 200;
    uc += 50;
    test_assert(uc == 250, "unsigned char +=");
    
    // unsigned int
    unsigned int ui = 4000000000U;
    ui += 294967295U;
    test_assert(ui == 4294967295U, "unsigned int += near max");
    
    // size_t
    size_t sz = 1000;
    sz += 500;
    test_assert(sz == 1500, "size_t +=");
}

// Test floating point types
void test_floating_point() {
    
    
    // float
    float f = 3.14f;
    f += 2.86f;
    test_assert(float_eq(f, 6.0f), "float += normal");
    
    f += 0.1f;
    test_assert(float_eq(f, 6.1f), "float += small increment");
    
    f += -1.1f;
    test_assert(float_eq(f, 5.0f), "float += negative");
    
    // double
    double d = 123.456789;
    d += 876.543211;
    test_assert(double_eq(d, 1000.0), "double += precision");
    
    // Test special float values
    // float inf_f = INFINITY;
    // inf_f += 100.0f;
    // test_assert(isinf(inf_f), "float += infinity");
    
    // float nan_f = NAN;
    // nan_f += 100.0f;
    // test_assert(isnan(nan_f), "float += NaN");
}

// Test pointer arithmetic
void test_pointer_arithmetic() {
    
    
    int arr[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
    int *ptr = arr;
    
    ptr += 3;
    test_assert(*ptr == 3, "int pointer += 3");
    
    ptr += 2;
    test_assert(*ptr == 5, "int pointer += 2");
    
    ptr += -1;
    test_assert(*ptr == 4, "int pointer += negative");
    
    // Different sized types
    double darr[5] = {1.1, 2.2, 3.3, 4.4, 5.5};
    double *dptr = darr;
    dptr += 2;
    test_assert(double_eq(*dptr, 3.3), "double pointer +=");
    
    // char pointer
    char str[] = "hello";
    char *cptr = str;
    cptr += 1;
    test_assert(*cptr == 'e', "char pointer +=");
}

// Test array subscripts with +=
void test_array_operations() {
    
    
    int arr[5] = {10, 20, 30, 40, 50};
    
    arr[0] += 5;
    test_assert(arr[0] == 15, "array[0] +=");
    
    arr[2] += arr[1];
    test_assert(arr[2] == 50, "array[i] += array[j]");
    
    // Multi-dimensional array
    int matrix[2][3] = {{1, 2, 3}, {4, 5, 6}};
    matrix[1][2] += 10;
    test_assert(matrix[1][2] == 16, "2D array +=");
    
    // Array of floats
    float farr[3] = {1.5f, 2.5f, 3.5f};
    farr[1] += 0.5f;
    test_assert(float_eq(farr[1], 3.0f), "float array +=");
}

// Test struct member operations
void test_struct_operations() {
    
    
    struct Point {
        int x, y;
        float z;
    };
    
    struct Point p;
    p.x = 10;
    p.y = 20;
    p.z = 3.14f;
    
    p.x += 5;
    test_assert(p.x == 15, "struct member int +=");
    
    p.y += p.x;
    test_assert(p.y == 35, "struct member += another member");
    
    p.z += 2.86f;
    test_assert(float_eq(p.z, 6.0f), "struct member float +=");
}

// Test overflow and underflow behavior
void test_overflow_underflow() {
    
    
    // Unsigned overflow (well-defined wraparound)
    unsigned char uc = 255;
    uc += 1;
    test_assert(uc == 0, "unsigned char overflow wraparound");
    
    uc = 0;
    uc += 255;
    test_assert(uc == 255, "unsigned char near overflow");
    
    // Test with larger unsigned type
    unsigned short us = 65535;
    us += 1;
    test_assert(us == 0, "unsigned short overflow wraparound");
}

// Test compound expressions
void test_compound_expressions() {
    
    
    int a = 10, b = 5, c = 3;
    
    a += b * c;
    test_assert(a == 25, "+= with multiplication");
    
    c += ++a;  // a becomes 26, then c += 26
    test_assert(c == 29 && a == 26, "+= with pre-increment");
}

// Test different numeric bases and constants
void test_numeric_constants() {
    
    
    int hex = 0x10;  // 16
    hex += 0xF;      // 15
    test_assert(hex == 31, "+= with hex constants");
    
    int octal = 010;  // 8
    octal += 007;     // 7
    test_assert(octal == 15, "+= with octal constants");
    
    // Character constants
    int ascii = 'A';  // 65
    ascii += 32;      // Convert to lowercase difference
    test_assert(ascii == 97, "+= with character constant");
}

// Test with volatile and const scenarios
void test_qualifiers() {
    
    
    volatile int vol = 100;
    vol += 50;
    test_assert(vol == 150, "volatile int +=");
    
    // Test that we can't modify const (this should not compile if uncommented)
    // const int const_val = 10;
    // const_val += 5;  // This should cause compilation error
    
    // But we can modify through non-const reference
    int modifiable = 200;
    int *ptr_to_mod = &modifiable;
    *ptr_to_mod += 75;
    test_assert(modifiable == 275, "+= through pointer");
}

// Test edge cases with zero and identity
void test_edge_cases() {
    
    
    int zero_test = 42;
    zero_test += 0;
    test_assert(zero_test == 42, "+= zero (identity)");
    
    float float_zero = 3.14f;
    float_zero += 0.0f;
    test_assert(float_eq(float_zero, 3.14f), "float += 0.0");
    
    // Negative zero
    float neg_zero = 1.0f;
    neg_zero += -0.0f;
    test_assert(float_eq(neg_zero, 1.0f), "float += -0.0");
    
    // Very small increments
    double tiny = 1.0;
    tiny += DBL_EPSILON;
    test_assert(tiny > 1.0, "double += DBL_EPSILON");
}

// Test performance and optimization scenarios
void test_optimization_cases() {
    
    
    int opt_test = 0;
    
    // Multiple additions that could be optimized
    opt_test += 1;
    opt_test += 1;
    opt_test += 1;
    opt_test += 1;
    opt_test += 1;
    test_assert(opt_test == 5, "multiple small increments");
    
    // Power of 2 additions (could be optimized to shifts)
    int pow2 = 1;
    pow2 += 2;
    pow2 += 4;
    pow2 += 8;
    test_assert(pow2 == 15, "power of 2 additions");
    
    // Self-addition (doubling)
    int self = 7;
    self += self;
    test_assert(self == 14, "self addition (doubling)");
}

int main() {
    
    
    
    test_integer_types();
    test_unsigned_types();
    test_floating_point();
    test_pointer_arithmetic();
    test_array_operations();
    test_struct_operations();
    test_overflow_underflow();
    test_compound_expressions();
    test_numeric_constants();
    test_qualifiers();
    test_edge_cases();
    test_optimization_cases();
    
    
    
    
    
    
    if (failed_tests == 0) {
        
        return 0;
    } else {
        
        return 1;
    }
}