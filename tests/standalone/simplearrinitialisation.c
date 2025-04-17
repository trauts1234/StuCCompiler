#include <stdio.h>
#include <limits.h>

//TODO test trailing commas

int test_basic_initialization() {
    int x[2] = {1, 2};
    if (x[0] != 1) return 1;
    if (x[1] != 2) return 2;
    
    char y[2] = {1u, 2ull + 1ull};
    if (y[0] != 1) return 3;
    if (y[1] != 3) return 4;
    
    int a = 1;
    int b = 2;
    int z[2] = {a, a+b};
    if (z[0] != 1) return 5;
    if (z[1] != 3) return 6;
    
    return 0;
}

/*int test_partial_initialization() {
    // Test partial initialization (remaining elements should be 0)
    int x[5] = {1, 2};
    if (x[0] != 1) return 10;
    if (x[1] != 2) return 11;
    if (x[2] != 0) return 12;
    if (x[3] != 0) return 13;
    if (x[4] != 0) return 14;
    
    // Test empty initialization (all elements should be 0)
    int y[3] = {};
    if (y[0] != 0) return 15;
    if (y[1] != 0) return 16;
    if (y[2] != 0) return 17;
    
    return 0;
}*/

int test_multidimensional() {
    // Test multi-dimensional array initialization
    int matrix[2][3] = {{1, 2, 3}, {4, 5, 6}};
    if (matrix[0][0] != 1) return 20;
    if (matrix[0][1] != 2) return 21;
    if (matrix[0][2] != 3) return 22;
    if (matrix[1][0] != 4) return 23;
    if (matrix[1][1] != 5) return 24;
    if (matrix[1][2] != 6) return 25;
    
    // Test partial multidimensional initialization
    /*int partial[2][3] = {{1, 2}};
    if (partial[0][0] != 1) return 26;
    if (partial[0][1] != 2) return 27;
    if (partial[0][2] != 0) return 28;
    if (partial[1][0] != 0) return 29;
    if (partial[1][1] != 0) return 30;
    if (partial[1][2] != 0) return 31;*/
    
    return 0;
}

/*int test_designated_initializers() {
    // Test designated initializers (C99 feature)
    int x[5] = {[0] = 1, [2] = 5, [4] = 10};
    if (x[0] != 1) return 40;
    if (x[1] != 0) return 41;
    if (x[2] != 5) return 42;
    if (x[3] != 0) return 43;
    if (x[4] != 10) return 44;
    
    // Test non-sequential designated initializers
    int y[5] = {[3] = 7, [1] = 3, [4] = 9};
    if (y[0] != 0) return 45;
    if (y[1] != 3) return 46;
    if (y[2] != 0) return 47;
    if (y[3] != 7) return 48;
    if (y[4] != 9) return 49;
    
    return 0;
}*/

int test_expressions_and_constants() {
    // Test expressions in initializers
    int a = 10;
    int x[4] = {a, a+5, a*2, a/2};
    if (x[0] != 10) return 50;
    if (x[1] != 15) return 51;
    if (x[2] != 20) return 52;
    if (x[3] != 5) return 53;
    
    // Test with different constant types
    int y[6] = {INT_MAX, INT_MIN, 0xFF, 077, 3u, -1};
    if (y[0] != INT_MAX) return 54;
    if (y[1] != INT_MIN) return 55;
    if (y[2] != 0xFF) return 56;
    if (y[3] != 077) return 57;
    if (y[4] != 3u) return 58;
    if (y[5] != -1) return 59;
    
    return 0;
}

int test_type_conversion() {
    // Test type conversion in initializers
    char c[5] = {65, 66, 67, 68, 69};
    if (c[0] != 'A') return 60;
    if (c[1] != 'B') return 61;
    if (c[2] != 'C') return 62;
    if (c[3] != 'D') return 63;
    if (c[4] != 'E') return 64;
    
    // Test overflow behavior in char arrays
    char o[2] = {257, -1};
    if (o[0] != 1) return 65;  // 257 % 256 = 1
    if (o[1] != -1) return 66;  // -1 in two's complement
    
    return 0;
}

int test_nested_scope() {
    int outer = 10;
    {
        // Test array initialization in nested scope
        int inner = 20;
        int x[3] = {outer, inner, outer + inner};
        if (x[0] != 10) return 70;
        if (x[1] != 20) return 71;
        if (x[2] != 30) return 72;
        
        // Test shadowing
        {
            int inner = 5;
            int outer = 15;
            int y[3] = {outer, inner, outer + inner};
            if (y[0] != 15) return 73;
            if (y[1] != 5) return 74;
            if (y[2] != 20) return 75;
        }
    }
    
    return 0;
}

/*int test_string_initialization() {
    // Test string initialization
    char s1[] = "Hello";
    if (s1[0] != 'H') return 80;
    if (s1[1] != 'e') return 81;
    if (s1[2] != 'l') return 82;
    if (s1[3] != 'l') return 83;
    if (s1[4] != 'o') return 84;
    if (s1[5] != '\0') return 85;
    
    // Test partial string initialization
    char s2[10] = "Hello";
    if (s2[0] != 'H') return 86;
    if (s2[5] != '\0') return 87;
    if (s2[6] != '\0') return 88;
    if (s2[9] != '\0') return 89;
    
    return 0;
}*/

int test_char_arrays() {
    // Test char array initialization with char constants
    char c1[5] = {'a', 'b', 'c', 'd', 'e'};
    if (c1[0] != 'a') return 90;
    if (c1[1] != 'b') return 91;
    if (c1[2] != 'c') return 92;
    if (c1[3] != 'd') return 93;
    if (c1[4] != 'e') return 94;
    
    // Test mixed character and integer values
    char c2[5] = {'a', 98, 'c', 100, 'e'};
    if (c2[0] != 'a') return 95;
    if (c2[1] != 'b') return 96;
    if (c2[2] != 'c') return 97;
    if (c2[3] != 'd') return 98;
    if (c2[4] != 'e') return 99;
    
    return 0;
}

int test_complex_expression_initializers() {
    int a = 5, b = 7;
    
    // Test complex expressions in initializers
    int x[5] = {++a, ++b, a*b, a%b, a<<2};
    if (x[0] != 6) return 100;  // ++a increments a to 6
    if (x[1] != 8) return 101;  // ++b increments b to 8, then returns 8
    if (x[2] != 48) return 102; // 6*8 = 48
    if (x[3] != 6) return 103;  // 6%8 = 6
    if (x[4] != 24) return 104; // 6<<2 = 24
    
    // Test function calls in initializers
    int y[2] = {printf(""), printf("")};
    if (y[0] != 0) return 105; // printf("") returns 0
    if (y[1] != 0) return 106;
    
    return 0;
}

/*int test_compound_literals() {
    // Test using compound literals for initialization
    int (*p)[3] = &(int[3]){1, 2, 3};
    if ((*p)[0] != 1) return 110;
    if ((*p)[1] != 2) return 111;
    if ((*p)[2] != 3) return 112;
    
    // Test array initialization with compound literals
    int x[2] = {((int[]){10, 20})[0], ((int[]){30, 40})[1]};
    if (x[0] != 10) return 113;
    if (x[1] != 40) return 114;
    
    return 0;
}*/

/*int test_edge_cases() {
    // Test initialization with maximum array bounds
    int x[1] = {INT_MAX};
    if (x[0] != INT_MAX) return 120;
    
    // Test initialization with minimum array bounds
    int y[1] = {INT_MIN};
    if (y[0] != INT_MIN) return 121;
    
    // Test large array initialization with zeros
    int large[1000] = {0};
    if (large[0] != 0) return 123;
    if (large[999] != 0) return 124;
    
    return 0;
}*/

int main() {
    int result;
    
    result = test_basic_initialization();
    if (result != 0) return result;
    
    //result = test_partial_initialization();
    //if (result != 0) return result;
    
    result = test_multidimensional();
    if (result != 0) return result;
    
    //result = test_designated_initializers();
    //if (result != 0) return result;
    
    result = test_expressions_and_constants();
    if (result != 0) return result;
    
    result = test_type_conversion();
    if (result != 0) return result;
    
    result = test_nested_scope();
    if (result != 0) return result;
    
    //result = test_string_initialization();
    //if (result != 0) return result;
    
    result = test_char_arrays();
    if (result != 0) return result;
    
    result = test_complex_expression_initializers();
    if (result != 0) return result;
    
    //result = test_compound_literals();
    //if (result != 0) return result;
    
    //result = test_edge_cases();
    //if (result != 0) return result;
    
    return 0;
}