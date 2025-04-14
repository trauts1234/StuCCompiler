#include <stdlib.h>

int test_basic_anonymous_enum() {
    enum { APPLE, BANANA, CHERRY } fruit;
    
    fruit = APPLE;
    if (fruit != 0) return 0;
    
    fruit = BANANA;
    if (fruit != 1) return 0;
    
    fruit = CHERRY;
    if (fruit != 2) return 0;
    
    return 1;
}

int test_negative_comparison() {
    enum { RED, GREEN, BLUE } color;
    
    color = RED;
    if (color < 0) return 0;
    
    if (RED != 0 || GREEN != 1 || BLUE != 2) return 0;
    
    return 1;
}

int test_variable_initialization() {
    enum { MONDAY, TUESDAY, WEDNESDAY, THURSDAY, FRIDAY } day = WEDNESDAY;
    
    if (day != 2) return 0;
    
    return 1;
}

int test_variable_assignment() {
    enum { DOG, CAT, BIRD } pet;
    int value;
    
    pet = DOG;
    value = pet;
    if (value != 0) return 0;
    
    pet = CAT;
    value = pet;
    if (value != 1) return 0;
    
    return 1;
}

int test_anonymous_enum_in_struct() {
    struct s {
        enum { SMALL, MEDIUM, LARGE } size;
        int value;
    } item;
    
    item.size = MEDIUM;
    item.value = 100;
    
    if (item.size != 1) return 0;
    if (SMALL != 0 || MEDIUM != 1 || LARGE != 2) return 0;
    
    return 1;
}

int test_enum_in_function_scope() {
    {
        enum { ALPHA, BETA, GAMMA } greek;
        greek = BETA;
        if (greek != 1) return 0;
    }
    
    {
        // Redefining the same identifiers in a different scope
        enum { ALPHA, BETA, GAMMA } greek;
        greek = GAMMA;
        if (greek != 2) return 0;
    }
    
    return 1;
}

int test_enum_comparison_operations() {
    enum { LOW, MEDIUM, HIGH } level1 = LOW;
    enum { LOW2, MEDIUM2, HIGH2 } level2 = HIGH2;
    
    if (!(level1 < level2)) return 0;
    if (!(level2 > level1)) return 0;
    if (level1 >= level2) return 0;
    if (level2 <= level1) return 0;
    if (level1 == level2) return 0;
    if (!(level1 != level2)) return 0;
    
    return 1;
}

int test_single_member_anonymous_enum() {
    enum { SINGLE } single_member;
    
    single_member = SINGLE;
    
    if (single_member != 0) return 0;
    if (SINGLE != 0) return 0;
    
    return 1;
}

int test_complex_expressions() {
    enum { A, B, C } x = B;
    enum { D, E, F } y = F;
    
    int result = x * 10 + y;
    
    if (result != 12) return 0;
    
    enum { G, H, I } z = I;
    
    // Test bitwise operations
    int bitwise_and = x & z;
    int bitwise_or = x | z;
    int bitwise_xor = x ^ z;
    
    if (bitwise_and != 0) return 0;
    if (bitwise_or != 3) return 0;  // B=1, I=2 -> 1|2=3
    if (bitwise_xor != 3) return 0; // B=1, I=2 -> 1^2=3
    
    return 1;
}

int test_casts() {
    enum { X, Y, Z } letter = Z;
    
    char c = (char)letter;
    int i = (int)letter;
    unsigned u = (unsigned)letter;
    
    if (c != 2) return 0;
    if (i != 2) return 0;
    if (u != 2) return 0;
    
    // Test casting back
    letter = (enum { x, y, z })3;  // This is out of range but should compile
    if (letter != 3) return 0;
    
    return 1;
}

int test_nested_enum() {
    enum { 
        OUTER1,
        OUTER2,
        OUTER3,
        NESTED_ENUM_TEST
    } outer;

    // Define a local scope with a nested anonymous enum
    {
        enum { INNER1, INNER2, INNER3 } inner;
        
        inner = INNER2;
        outer = OUTER3;
        
        if (inner != 1) return 0;
        if (outer != 2) return 0;
    }
    
    // Test to ensure the outer enum is still valid outside the nested scope
    outer = NESTED_ENUM_TEST;
    if (outer != 3) return 0;
    
    return 1;
}

int test_enum_arithmetic() {
    enum { ONE, TWO, THREE } num = ONE;
    
    // Increment
    ++num;
    if (num != TWO) return 0;
    
    // Decrement
    --num;
    if (num != ONE) return 0;
    
    // Assignment with addition
    num = num + 2;
    if (num != THREE) return 0;
    
    // Direct arithmetic
    int result = num * 3 / 3;  // Should still be THREE (2)
    if (result != THREE) return 0;
    
    return 1;
}

int test_using_as_array_index() {
    int array[5];
    array[0] = 10; array[1] = 20; array[2] = 30; array[3] = 40; array[4] = 50;
    enum { INDEX_0, INDEX_1, INDEX_2, INDEX_3, INDEX_4 } index;
    
    index = INDEX_2;
    
    if (array[index] != 30) return 0;
    
    index = INDEX_4;
    if (array[index] != 50) return 0;
    
    return 1;
}

int main() {
    if (!test_basic_anonymous_enum()) return 1;
    if (!test_negative_comparison()) return 2;
    if (!test_variable_initialization()) return 3;
    if (!test_variable_assignment()) return 4;
    if (!test_anonymous_enum_in_struct()) return 5;
    if (!test_enum_in_function_scope()) return 6;
    if (!test_enum_comparison_operations()) return 7;
    if (!test_single_member_anonymous_enum()) return 8;
    if (!test_complex_expressions()) return 9;
    if (!test_casts()) return 10;
    if (!test_nested_enum()) return 11;
    if (!test_enum_arithmetic()) return 12;
    if (!test_using_as_array_index()) return 13;
    
    return 0;
}