#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <limits.h>
#include <float.h>
#include <math.h>
#include <string.h>

// Test counter and failure tracking
static int test_count = 0;
static int failed_tests = 0;

// Macro to run tests and track results
void TEST(_Bool condition, char* description) { \
    test_count++; \
    if (!(condition)) { \
        printf("FAIL: Test %d - %s\n", test_count, description); \
        failed_tests++; \
    } else { \
        printf("PASS: Test %d - %s\n", test_count, description); \
    } \
}

// Helper function for floating point comparison
int float_equal(double a, double b, double epsilon) {
    return fabs(a - b) < epsilon;
}

// Test basic integer types
void test_basic_integers() {
    printf("\n=== Testing Basic Integer Types ===\n");
    
    // char
    char c = 10;
    c -= 3;
    TEST(c == 7, "char -= operation");
    
    // signed char
    signed char sc = 50;
    sc -= 25;
    TEST(sc == 25, "signed char -= operation");
    
    // unsigned char
    unsigned char uc = 200;
    uc -= 50;
    TEST(uc == 150, "unsigned char -= operation");
    
    // short
    short s = 1000;
    s -= 250;
    TEST(s == 750, "short -= operation");
    
    // unsigned short
    unsigned short us = 2000;
    us -= 500;
    TEST(us == 1500, "unsigned short -= operation");
    
    // int
    int i = 100000;
    i -= 25000;
    TEST(i == 75000, "int -= operation");
    
    // unsigned int
    unsigned int ui = 4000000000U;
    ui -= 1000000000U;
    TEST(ui == 3000000000U, "unsigned int -= operation");
    
    // long
    long l = 1000000L;
    l -= 250000L;
    TEST(l == 750000L, "long -= operation");
    
    // unsigned long
    unsigned long ul = 5000000000UL;
    ul -= 1000000000UL;
    TEST(ul == 4000000000UL, "unsigned long -= operation");
    
    // long long
    long long ll = 1000000000000LL;
    ll -= 250000000000LL;
    TEST(ll == 750000000000LL, "long long -= operation");
    
    // unsigned long long
    unsigned long long ull = 10000000000000ULL;
    ull -= 2500000000000ULL;
    TEST(ull == 7500000000000ULL, "unsigned long long -= operation");
}

// Test fixed-width integer types
void test_fixed_width_integers() {
    printf("\n=== Testing Fixed-Width Integer Types ===\n");
    
    int8_t i8 = 100;
    i8 -= 25;
    TEST(i8 == 75, "int8_t -= operation");
    
    uint8_t ui8 = 200;
    ui8 -= 50;
    TEST(ui8 == 150, "uint8_t -= operation");
    
    int16_t i16 = 30000;
    i16 -= 5000;
    TEST(i16 == 25000, "int16_t -= operation");
    
    uint16_t ui16 = 60000;
    ui16 -= 10000;
    TEST(ui16 == 50000, "uint16_t -= operation");
    
    int32_t i32 = 2000000000;
    i32 -= 500000000;
    TEST(i32 == 1500000000, "int32_t -= operation");
    
    uint32_t ui32 = 4000000000U;
    ui32 -= 1000000000U;
    TEST(ui32 == 3000000000U, "uint32_t -= operation");
    
    int64_t i64 = 9000000000000000000LL;
    i64 -= 1000000000000000000LL;
    TEST(i64 == 8000000000000000000LL, "int64_t -= operation");
    
    uint64_t ui64 = 18000000000000000000ULL;
    ui64 -= 2000000000000000000ULL;
    TEST(ui64 == 16000000000000000000ULL, "uint64_t -= operation");
}

// Test floating point types
void test_floating_point() {
    printf("\n=== Testing Floating Point Types ===\n");
    
    // float
    float f = 123.456f;
    f -= 23.456f;
    TEST(float_equal(f, 100.0f, 0.001), "float -= operation");
    
    // double
    double d = 123.456789;
    d -= 23.456789;
    TEST(float_equal(d, 100.0, 0.000001), "double -= operation");
    
    // Test with negative numbers
    float neg_f = -50.5f;
    neg_f -= 10.5f;
    TEST(float_equal(neg_f, -61.0f, 0.001), "float -= with negative result");
    
    // Test subtracting negative (should add)
    double pos_d = 100.0;
    pos_d -= (-25.0);
    TEST(float_equal(pos_d, 125.0, 0.000001), "double -= negative number");
}

// Test pointer arithmetic
void test_pointer_arithmetic() {
    printf("\n=== Testing Pointer Arithmetic ===\n");
    
    int arr[] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    int *ptr = &arr[9]; // Point to last element
    
    ptr -= 3; // Move back 3 elements
    TEST(*ptr == 7, "int pointer -= operation");
    
    char str[] = "Hello World";
    char *char_ptr = &str[10]; // Point to 'd'
    char_ptr -= 5; // Move back to ' '
    TEST(*char_ptr == ' ', "char pointer -= operation");
    
    double darr[] = {1.1, 2.2, 3.3, 4.4, 5.5};
    double *dptr = &darr[4]; // Point to last element
    dptr -= 2; // Move back 2 elements
    TEST(float_equal(*dptr, 3.3, 0.001), "double pointer -= operation");
    
    // Test with larger offset
    long larr[] = {100, 200, 300, 400, 500, 600, 700, 800, 900, 1000};
    long *lptr = &larr[9];
    lptr -= 7;
    TEST(*lptr == 300, "long pointer -= large offset");
}

// Test array indexing with -= in subscript
void test_array_operations() {
    printf("\n=== Testing Array Operations ===\n");
    
    int arr[10] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
    int index = 7;
    
    index -= 3;
    TEST(arr[index] == 4, "array access with -= modified index");
    
    // Test multidimensional array
    int matrix[3][4] = {{1,2,3,4}, {5,6,7,8}, {9,10,11,12}};
    int row = 2, col = 3;
    row -= 1;
    col -= 2;
    TEST(matrix[row][col] == 6, "2D array access with -= modified indices");
    
    // Test array element modification
    int values[5] = {100, 200, 300, 400, 500};
    values[2] -= 50;
    TEST(values[2] == 250, "array element -= operation");
}

// Test overflow and underflow conditions
void test_overflow_underflow() {
    printf("\n=== Testing Overflow/Underflow Conditions ===\n");
    
    // Test unsigned underflow (wraps around)
    unsigned char uc = 5;
    uc -= 10;
    TEST(uc > 250, "unsigned char underflow wrapping");
    
    unsigned int ui = 100;
    ui -= 200;
    TEST(ui > 4000000000U, "unsigned int underflow wrapping");
    
    // Test signed overflow/underflow
    signed char sc_min = SCHAR_MIN;
    sc_min -= 1;
    TEST(sc_min == SCHAR_MAX, "signed char underflow wrapping");
    
    signed char sc_test = -100;
    sc_test -= 50;
    TEST(sc_test == -150 || sc_test > 0, "signed char near limit subtraction");
    
    // Test with maximum values
    int max_test = INT_MAX;
    max_test -= (-1); // Should overflow
    TEST(max_test < 0, "int overflow when subtracting negative");
}

// Test special floating point values
void test_special_float_values() {
    
    // Test with very small numbers
    float tiny = FLT_MIN;
    tiny -= FLT_MIN;
    TEST(float_equal(tiny, 0.0f, FLT_EPSILON), "very small float subtraction");
}

// Test complex expressions and chained operations
void test_complex_expressions() {
    printf("\n=== Testing Complex Expressions ===\n");
    
    int x = 200;
    x -= (x / 4); // x = 200 - 50 = 150
    TEST(x == 150, "-= with expression on right side");
    
    int arr[] = {10, 20, 30, 40, 50};
    int idx = 4;
    arr[idx] -= arr[idx -= 2]; // Complex: idx becomes 2, then arr[4] -= arr[2]
    TEST(arr[4] == 20 && idx == 2, "-= in array subscript with array element");
    
    float result = 100.0f;
    result -= 10.5f * 2.0f;
    TEST(float_equal(result, 79.0f, 0.001f), "-= with multiplication expression");
}

// Test with struct members
void test_struct_members() {
    printf("\n=== Testing Struct Member Operations ===\n");
    
    struct TestStruct {
        int integer;
        float floating;
        char character;
        int *pointer;
    };
    
    int target = 100;
    struct TestStruct ts;
    ts.integer = 200;
    ts.floating = 50.5f;
    ts.character = 'z';
    ts.pointer = &target;
    
    ts.integer -= 75;
    TEST(ts.integer == 125, "struct int member -= operation");
    
    ts.floating -= 25.5f;
    TEST(float_equal(ts.floating, 25.0f, 0.001f), "struct float member -= operation");
    
    ts.character -= 10;
    TEST(ts.character == 'p', "struct char member -= operation");
    
    int arr_for_ptr[] = {1, 2, 3, 4, 5};
    ts.pointer = &arr_for_ptr[4];
    ts.pointer -= 2;
    TEST(*ts.pointer == 3, "struct pointer member -= operation");
}

// Test with function parameters and return values
int subtract_and_return(int val) {
    val -= 50;
    return val;
}

void test_function_operations() {
    printf("\n=== Testing Function Parameter Operations ===\n");
    
    int result = subtract_and_return(150);
    TEST(result == 100, "parameter -= in function");
    
    // Test with global-like behavior through static
    static int static_var = 1000;
    static_var -= 250;
    TEST(static_var == 750, "static variable -= operation");
    
    static_var -= 250;
    TEST(static_var == 500, "static variable -= operation (persistent)");
}

// Test edge cases with zero and one
void test_zero_and_one() {
    printf("\n=== Testing Zero and One Operations ===\n");
    
    int zero_test = 0;
    zero_test -= 0;
    TEST(zero_test == 0, "zero -= zero");
    
    int from_zero = 0;
    from_zero -= 5;
    TEST(from_zero == -5, "zero -= positive");
    
    int to_zero = 42;
    to_zero -= 42;
    TEST(to_zero == 0, "value -= itself equals zero");
    
    int one_test = 1;
    one_test -= 1;
    TEST(one_test == 0, "one -= one");
    
    float float_one = 1.0f;
    float_one -= 0.0f;
    TEST(float_equal(float_one, 1.0f, 0.001f), "float -= zero");
}

// Test const and volatile (where applicable)
void test_qualifiers() {
    printf("\n=== Testing Type Qualifiers ===\n");
    
    volatile int vol_int = 100;
    vol_int -= 25;
    TEST(vol_int == 75, "volatile int -= operation");
    
    // Note: can't test const since const variables can't be modified
    // but we can test const expressions
    int regular = 200;
    const int const_val = 50;
    regular -= const_val;
    TEST(regular == 150, "variable -= const value");
}

int main() {
    printf("Comprehensive -= Operator Test Suite\n");
    printf("====================================\n");
    
    // Run all test suites
    test_basic_integers();
    test_fixed_width_integers();
    test_floating_point();
    test_pointer_arithmetic();
    test_array_operations();
    test_overflow_underflow();
    test_special_float_values();
    test_complex_expressions();
    test_struct_members();
    test_function_operations();
    test_zero_and_one();
    test_qualifiers();
    
    // Final results
    printf("\n=== Test Results ===\n");
    printf("Total tests: %d\n", test_count);
    printf("Passed: %d\n", test_count - failed_tests);
    printf("Failed: %d\n", failed_tests);
    
    if (failed_tests == 0) {
        printf("\nAll tests PASSED! The -= operator is working correctly.\n");
        return 0;
    } else {
        printf("\nSome tests FAILED! The -= operator may have issues.\n");
        return 1;
    }
}