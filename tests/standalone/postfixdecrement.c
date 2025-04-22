#include <stdbool.h>

// Function to test basic suffix decrement on integers
bool test_int_suffix_decrement() {
    int a = 10;
    int b = a--;
    
    // b should be 10 (original value)
    // a should be 9 (decremented value)
    if (b != 10 || a != 9) {
        return false;
    }
    
    // Test in more complex expressions
    int c = 5;
    int d = c-- + 3;
    
    // d should be 8 (5+3)
    // c should be 4 (decremented value)
    if (d != 8 || c != 4) {
        return false;
    }
    
    return true;
}

// Function to test suffix decrement with pointers
bool test_pointer_suffix_decrement() {
    int arr[5] = {1, 2, 3, 4, 5};
    int *ptr = &arr[2]; // Points to element 3
    
    int val = *ptr--;
    
    // val should be 3 (original value at ptr)
    // ptr should now point to arr[1] (2)
    if (val != 3 || *ptr != 2) {
        return false;
    }
    
    return true;
}

// Function to test suffix decrement in loop control
bool test_loop_suffix_decrement() {
    int sum = 0;
    int i = 5;
    
    while (i--) {
        sum = sum + i;
    }
    
    // i should be -1 after the loop
    // sum should be 0+1+2+3+4 = 10
    if (i != -1 || sum != 10) {
        return false;
    }
    
    return true;
}

// Function to test multiple suffix decrements in one statement
bool test_multiple_suffix_decrements() {
    int a = 5, b = 10;
    int c = a-- + b--;
    
    // c should be 15 (5+10)
    // a should be 4
    // b should be 9
    if (c != 15 || a != 4 || b != 9) {
        return false;
    }
    
    return true;
}

// Function to test suffix decrement with different data types
bool test_other_types_suffix_decrement() {
    // Test with char
    char ch = 'c';
    char prev_ch = ch--;
    if (prev_ch != 'c' || ch != 'b') {
        return false;
    }
    
    // Test with unsigned int
    unsigned int ui = 10;
    unsigned int prev_ui = ui--;
    if (prev_ui != 10 || ui != 9) {
        return false;
    }
    
    return true;
}

// Function to test prefix vs suffix decrement
bool test_prefix_vs_suffix() {
    int a = 10;
    int b = 10;
    
    int result_suffix = a--;
    int result_prefix = --b;
    
    // result_suffix should be 10, a should be 9
    // result_prefix should be 9, b should be 9
    if (result_suffix != 10 || a != 9 || result_prefix != 9 || b != 9) {
        return false;
    }
    
    return true;
}

// Function to test operator precedence with suffix decrement
bool test_operator_precedence() {
    int a = 10;
    int b = 5;
    int c = a-- * b;
    
    // c should be 50 (10*5)
    // a should be 9
    if (c != 50 || a != 9) {
        return false;
    }
    
    a = 10;
    c = (a--) * b;
    
    // c should still be 50 (10*5)
    // a should be 9
    if (c != 50 || a != 9) {
        return false;
    }
    
    return true;
}

// Function to test array indexing with suffix decrement
bool test_array_indexing() {
    int arr[5] = {10, 20, 30, 40, 50};
    int i = 3;
    
    int val = arr[i--];
    
    // val should be 40 (arr[3])
    // i should be 2
    if (val != 40 || i != 2) {
        return false;
    }
    
    return true;
}

// Function to test nested suffix decrements
bool test_nested_suffix_decrements() {
    int a = 5;
    int b = 10;
    int c = a-- - --b;
    
    // Order should be: get value of a (5), decrement a to 4, decrement b to 9, subtract 9 from 5
    // c should be -4 (5-9)
    // a should be 4
    // b should be 9
    if (c != -4 || a != 4 || b != 9) {
        return false;
    }
    
    return true;
}

int main() {
    // Test 1
    if (!test_int_suffix_decrement()) {
        return 1;
    }
    
    // Test 2
    if (!test_pointer_suffix_decrement()) {
        return 2;
    }
    
    // Test 3
    if (!test_loop_suffix_decrement()) {
        return 3;
    }
    
    // Test 4
    if (!test_multiple_suffix_decrements()) {
        return 4;
    }
    
    // Test 5
    if (!test_other_types_suffix_decrement()) {
        return 5;
    }
    
    // Test 6
    if (!test_prefix_vs_suffix()) {
        return 6;
    }
    
    // Test 7
    if (!test_operator_precedence()) {
        return 7;
    }
    
    // Test 8
    if (!test_array_indexing()) {
        return 8;
    }
    
    // Test 9
    if (!test_nested_suffix_decrements()) {
        return 9;
    }
    
    // All tests passed
    return 0;
}