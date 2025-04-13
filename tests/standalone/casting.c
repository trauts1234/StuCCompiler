#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <limits.h>

// Test counter to help with debugging
int testNumber = 0;

// Function to check a test and return error code immediately if it fails
#define CHECK(condition, error_code) do { \
    testNumber++; \
    if (!(condition)) { \
        /* Uncomment for debugging: printf("Test %d failed with code %d\n", testNumber, error_code); */ \
        return error_code; \
    } \
} while (0)

// Function to test basic integer casting
int test_integer_casting() {
    // Basic int casts
    int x = 42;
    long y = (long)x;
    CHECK(y == 42, 100);
    
    // Casting to smaller type (potential truncation)
    int large_val = INT_MAX;
    short s = (short)large_val;
    CHECK(s == (short)INT_MAX, 101);  // Should be truncated
    
    // Negative values
    int neg = -5;
    unsigned int u = (unsigned int)neg;
    CHECK(u == (unsigned int)-5, 102);  // Should wrap around
    
    // Zero edge case
    int zero = 0;
    long long_zero = (long)zero;
    CHECK(long_zero == 0, 103);
    
    // Casting between signed and unsigned
    unsigned int uint_max = UINT_MAX;
    int int_cast = (int)uint_max;
    CHECK(int_cast == -1, 104);  // Should wrap to -1
    
    // Casting with operations inside the cast
    int a = 5, b = 3;
    long result = (long)(a + b);
    CHECK(result == 8, 105);
    
    return 0;
}

// Function to test pointer casting
int test_pointer_casting() {
    // Basic pointer cast
    int x = 42;
    int* p_int = &x;
    void* p_void = (void*)p_int;
    int* p_back = (int*)p_void;
    CHECK(p_back == p_int, 300);
    CHECK(*p_back == 42, 301);
    
    // Cast between different sized pointers
    int arr[5] = {1, 2, 3, 4, 5};
    int* p_arr = arr;
    char* p_char = (char*)p_arr;
    CHECK((void*)p_char == (void*)arr, 302);
    
    // Check byte-by-byte access after casting
    unsigned char* bytes = (unsigned char*)arr;
    // On little-endian machines, the first byte of the first int should be 1
    // This is implementation-dependent, so we'll check both possibilities
    CHECK(bytes[0] == 1 || bytes[0] == 0, 303);
    
    // Double indirection
    int y = 100;
    int* py = &y;
    int** ppy = &py;
    void** ppv = (void**)ppy;
    int** ppy_back = (int**)ppv;
    CHECK(ppy_back == ppy, 305);
    CHECK(*ppy_back == py, 306);
    CHECK(**ppy_back == 100, 307);
    
    
    // Cast through multiple pointer types
    int z = 255;
    int* pz = &z;
    char* pc = (char*)pz;
    short* ps = (short*)pc;
    int* pz_back = (int*)ps;
    CHECK(pz_back == pz, 309);
    
    return 0;
}

// Function to test structure and union casting
int test_struct_cast() {
    // Basic struct with same memory layout
    struct A {
        int x;
        int y;
    };
    
    struct B {
        int a;
        int b;
    };
    
    struct A sa = {1, 2};
    struct B* sb = (struct B*)&sa;
    CHECK(sb->a == 1, 400);
    CHECK(sb->b == 2, 401);
    
    // Casting between struct and primitive
    struct SingleInt {
        int value;
    };
    
    struct SingleInt si = {42};
    int* pi_struct = (int*)&si;
    CHECK(*pi_struct == 42, 404);
    
    // Cast to and from structs with bit fields
    struct BitField {
        unsigned int a : 4;
        unsigned int b : 4;
        unsigned int c : 8;
    };
    
    struct BitField bf = {0xF, 0x5, 0xAA};
    unsigned short* pus = (unsigned short*)&bf;
    // The exact layout depends on endianness and compiler packing
    // Just checking that the cast is possible
    CHECK(pus != NULL, 405);
    
    return 0;
}

// Function to test casts involving enum types
int test_enum_casting() {
    // Basic enum cast
    enum Color { RED, GREEN, BLUE };
    enum Color c = GREEN;
    int i_enum = (int)c;
    CHECK(i_enum == 1, 500);  // GREEN is typically 1
    
    // Cast back from int to enum
    int val = 2;
    enum Color back = (enum Color)val;
    CHECK(back == BLUE, 501);
    
    // Cast out-of-range value to enum
    int invalid = 999;
    enum Color invalid_color = (enum Color)invalid;
    CHECK((int)invalid_color == 999, 502);
    
    // Define enum inside a cast (very odd edge case)
    int weird = (enum { X, Y, Z })Y;
    CHECK(weird == 1, 503);
    
    // Anonymous enum with specific values
    int val2 = (enum { A = 5, B = 10, C = 15 })B;
    CHECK(val2 == 10, 504);
    
    // Enum with negative values
    enum Signs { NEGATIVE = -1, ZERO = 0, POSITIVE = 1 };
    unsigned int u_sign = (unsigned int)NEGATIVE;
    CHECK(u_sign == (unsigned int)-1, 505);
    
    // Cast enum to pointer (weird but possible)
    enum SmallVals { SMALL1 = 1, SMALL2 = 2 };
    void* ptr = (void*)(uintptr_t)SMALL2;
    enum SmallVals back_val = (enum SmallVals)(uintptr_t)ptr;
    CHECK(back_val == SMALL2, 507);
    
    return 0;
}

// Test casts involving const qualifiers
int test_const_casting() {
    // Basic const casting
    const int ci = 10;
    int* mutable_ptr = (int*)&ci;
    // Modifying *mutable_ptr is undefined behavior, but the cast itself is valid
    CHECK(&ci == (const int*)mutable_ptr, 600);
    
    // Double pointer const casting
    int x = 20;
    int* px = &x;
    const int* const* ppci = (const int* const*)&px;
    int** ppx_back = (int**)ppci;
    CHECK(ppx_back == &px, 601);
    
    // Array and const
    int arr[3] = {1, 2, 3};
    const int* c_arr = (const int*)arr;
    int* arr_back = (int*)c_arr;
    CHECK(arr_back == arr, 602);
    CHECK(arr_back[1] == 2, 603);
    
    // Const struct member access
    struct Point {
        int x;
        int y;
    };
    
    const struct Point p = {5, 10};
    int* px_field = (int*)&p.x;
    CHECK(*px_field == 5, 604);
    
    return 0;
}

// Test casts involving volatile
int test_volatile_casting() {
    // Basic volatile cast
    int i = 42;
    volatile int* vi_ptr = (volatile int*)&i;
    int* normal_ptr = (int*)vi_ptr;
    CHECK(normal_ptr == &i, 700);
    CHECK(*normal_ptr == 42, 701);
    
    // Multiple qualifiers
    int j = 99;
    const volatile int* cvj = (const volatile int*)&j;
    int* j_back = (int*)cvj;
    CHECK(j_back == &j, 702);
    
    return 0;
}

// Test weird compound casts
int test_compound_casts() {
    // Cast involving an anonymous struct
    int weird_struct_val = (struct { int x; int y; }){.x = 5, .y = 10}.y;
    CHECK(weird_struct_val == 10, 800);
    
    // Cast with sizeof inside
    int size_cast = (int)sizeof(char);
    CHECK(size_cast == 1, 802);
    
    // Cast with ternary operator
    int ternary_val = (int)(1 ? 10.5 : 20.5);
    CHECK(ternary_val == 10, 803);
    
    // Cast array to pointer to different type
    int arr[5] = {1, 2, 3, 4, 5};
    short* short_arr = (short*)arr;
    // On little-endian machines, if int is 4 bytes and short is 2:
    // The first two shorts might be 1 and 0
    CHECK(short_arr[0] == 1 || short_arr[1] == 0 || short_arr[0] == 0, 804);
    
    // Cast involving comma operator
    int comma_val = (int)(1.1, 2.2, 3.3);
    CHECK(comma_val == 3, 805);
    
    // Multiple casts in single expression
    int base = 65;
    char c1 = (char)base;
    int back = (int)(char)(unsigned char)(signed char)c1;
    CHECK(back == 65, 806);
    
    // Cast involving bitfields
    struct BF {
        unsigned int a : 3;
        unsigned int b : 5;
    } bf = {.a = 7, .b = 31};
    
    unsigned char* bf_bytes = (unsigned char*)&bf;
    // Can't check exact values as it depends on endianness and packing
    CHECK(bf_bytes != NULL, 807);
    
    return 0;
}

// Main function to run all tests
int main() {
    // Run all test functions
    int result;
    
    result = test_integer_casting();
    if (result != 0) return result;
    
    result = test_pointer_casting();
    if (result != 0) return result;
    
    result = test_struct_cast();
    if (result != 0) return result;
    
    result = test_enum_casting();
    if (result != 0) return result;
    
    result = test_const_casting();
    if (result != 0) return result;
    
    result = test_volatile_casting();
    if (result != 0) return result;
    
    result = test_compound_casts();
    if (result != 0) return result;
    
    // If we got here, all tests passed
    return 0;
}