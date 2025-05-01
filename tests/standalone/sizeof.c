#include <stddef.h>
#include <stdint.h>
#include <limits.h>

// Basic sizeof behavior verification
int test_basic_sizes() {
    // Verify fundamental types have expected relative sizes
    if (!(sizeof(char) <= sizeof(short) &&
          sizeof(short) <= sizeof(int) &&
          sizeof(int) <= sizeof(long))) {
        return 1;
    }
    
    // Verify char is exactly 1 byte
    if (sizeof(char) != 1) {
        return 1;
    }
    
    // Verify pointer sizes are consistent
    if (sizeof(void*) != sizeof(char*) ||
        sizeof(char*) != sizeof(int*)) {
        return 1;
    }
    
    return 0;
}

// Array size verification
int test_array_sizes() {
    char c[10];
    int i[5];
    
    // Verify array sizes
    if (sizeof(c) != 10 * sizeof(char)) {
        return 1;
    }
    
    if (sizeof(i) != 5 * sizeof(int)) {
        return 1;
    }
    
    // Verify sizeof multidimensional arrays
    char matrix[3][4];
    if (sizeof(matrix) != 3 * 4 * sizeof(char)) {
        return 1;
    }
    
    return 0;
}

// Structure padding/alignment verification
int test_struct_sizes() {
    
    struct Aligned {
        char c;
        int i;
        char d;
    };
    
    // Verify structure size includes alignment padding
    if (sizeof(struct Aligned) < sizeof(char) + sizeof(int) + sizeof(char)) {
        return 1;
    }
    
    return 0;
}

// sizeof in expressions
int test_sizeof_expressions() {
    int a = 10;
    int *ptr = &a;
    
    // Verify sizeof in arithmetic expressions
    if (sizeof(a) + sizeof(int) != 2 * sizeof(int)) {
        return 1;
    }
    
    // Verify sizeof with dereference operator
    if (sizeof(*ptr) != sizeof(int)) {
        return 1;
    }
    
    // Verify sizeof with arrays
    int arr[10];
    if (sizeof(arr) / sizeof(arr[0]) != 10) {
        return 1;
    }
    
    return 0;
}

// Verify fixed-width integer types
int test_fixed_width_types() {
    // Test explicit-width integer types
    if (sizeof(int8_t) != 1 || sizeof(uint8_t) != 1) {
        return 1;
    }
    
    if (sizeof(int16_t) != 2 || sizeof(uint16_t) != 2) {
        return 1;
    }
    
    if (sizeof(int32_t) != 4 || sizeof(uint32_t) != 4) {
        return 1;
    }
    
    if (sizeof(int64_t) != 8 || sizeof(uint64_t) != 8) {
        return 1;
    }
    
    return 0;
}

int main() {
    // Execute all test functions
    if (test_basic_sizes() != 0) {
        return 1;
    }
    
    if (test_array_sizes() != 0) {
        return 1;
    }
    
    if (test_struct_sizes() != 0) {
        return 1;
    }
    
    if (test_sizeof_expressions() != 0) {
        return 1;
    }
    
    if (test_fixed_width_types() != 0) {
        return 1;
    }
    
    // All tests passed
    return 0;
}