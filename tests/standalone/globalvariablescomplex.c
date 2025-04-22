#include <stdio.h>
#include <limits.h>
#include <stdlib.h>

// Basic arithmetic operations
int global_add = 1 + 1;
int global_sub = 5 - 3;
int global_mul = 3 * 4;
int global_div = 10 / 3;
int global_mod = 10 % 3;

// More complex expressions
int global_complex1 = (1 + 2) * 3 / 4;
int global_complex2 = 5 + 2 * 3 - 4 / 2;
int global_complex3 = (10 + 5) / 3 * 2 - 1;

// Unary operations
int global_unary_plus = +5;
int global_unary_minus = -5;
int global_unary_not = !5;
int global_unary_bitwise_not = ~5;

// Bitwise operations
int global_bitwise_and = 0xF0 & 0x0F;
int global_bitwise_or = 0xF0 | 0x0F;
int global_bitwise_xor = 0xF0 ^ 0x0F;

// Edge cases with INT_MAX, INT_MIN, 0
int global_int_max = INT_MAX;
int global_int_max_plus_one = INT_MAX + 1;  // Overflow
int global_int_min = INT_MIN;
int global_int_min_minus_one = INT_MIN - 1;  // Underflow

// Type promotion
char global_char_add = 100 + 100;  // Should promote and truncate
short global_short_promotion = 30000 + 30000;  // Should promote and truncate
int global_int_promotion = 2000000000 + 2000000000;  // Should overflow

// Unsigned vs signed comparison
unsigned int global_unsigned_max = UINT_MAX;
int global_signed_comparison = UINT_MAX > INT_MAX;  // Should be 1 (true)

// Hexadecimal and octal
int global_hex = 0xFF;
int global_octal = 0777;

// Shifting
int global_shift_left = 1 << 4;
int global_shift_right = 16 >> 2;
int global_shift_negative = -16 >> 2;  // Implementation-defined
int global_shift_overflow = 1 << 31;  // Undefined behavior

int main() {
    
    // Local equivalents for comparison
    int local_add = 1 + 1;
    int local_sub = 5 - 3;
    int local_mul = 3 * 4;
    int local_div = 10 / 3;
    int local_mod = 10 % 3;
    int local_complex1 = (1 + 2) * 3 / 4;
    int local_complex2 = 5 + 2 * 3 - 4 / 2;
    int local_complex3 = (10 + 5) / 3 * 2 - 1;
    int local_unary_plus = +5;
    int local_unary_minus = -5;
    int local_unary_not = !5;
    int local_unary_bitwise_not = ~5;
    int local_bitwise_and = 0xF0 & 0x0F;
    int local_bitwise_or = 0xF0 | 0x0F;
    int local_bitwise_xor = 0xF0 ^ 0x0F;
    int local_int_max = INT_MAX;
    int local_int_max_plus_one = INT_MAX + 1;
    int local_int_min = INT_MIN;
    int local_int_min_minus_one = INT_MIN - 1;
    char local_char_add = 100 + 100;
    short local_short_promotion = 30000 + 30000;
    int local_int_promotion = 2000000000 + 2000000000;
    unsigned int local_unsigned_max = UINT_MAX;
    int local_signed_comparison = UINT_MAX > INT_MAX;
    int local_hex = 0xFF;
    int local_octal = 0777;
    int local_shift_left = 1 << 4;
    int local_shift_right = 16 >> 2;
    int local_shift_negative = -16 >> 2;
    int local_shift_overflow = 1 << 31;
    
    // Check basic arithmetic
    if (global_add != local_add) {

        return 1;
    }
    
    if (global_sub != local_sub) {

        return 2;
    }
    
    if (global_mul != local_mul) {

        return 3;
    }
    
    if (global_div != local_div) {

        return 4;
    }
    
    if (global_mod != local_mod) {

        return 5;
    }
    
    // Check complex expressions
    if (global_complex1 != local_complex1) {

        return 6;
    }
    
    if (global_complex2 != local_complex2) {

        return 7;
    }
    
    if (global_complex3 != local_complex3) {

        return 8;
    }
    
    // Check unary operations
    if (global_unary_plus != local_unary_plus) {

        return 9;
    }
    
    if (global_unary_minus != local_unary_minus) {

        return 10;
    }
    
    if (global_unary_not != local_unary_not) {

        return 11;
    }
    
    if (global_unary_bitwise_not != local_unary_bitwise_not) {

        return 12;
    }
    
    // Check bitwise operations
    if (global_bitwise_and != local_bitwise_and) {

        return 13;
    }

    if (global_bitwise_or != local_bitwise_or) {

        return 14;
    }
    
    if (global_bitwise_xor != local_bitwise_xor) {

        return 15;
    }
    
    // Check edge cases
    if (global_int_max != local_int_max) {

        return 16;
    }
    
    if (global_int_max_plus_one != local_int_max_plus_one) {

        return 17;
    }
    
    if (global_int_min != local_int_min) {

        return 18;
    }
    
    if (global_int_min_minus_one != local_int_min_minus_one) {

        return 19;
    }
    
    // Check type promotion
    if (global_char_add != local_char_add) {

        return 20;
    }
    
    if (global_short_promotion != local_short_promotion) {

        return 21;
    }
    
    if (global_int_promotion != local_int_promotion) {

        return 22;
    }
    
    // Check unsigned vs signed
    if (global_unsigned_max != local_unsigned_max) {

        return 23;
    }
    
    if (global_signed_comparison != local_signed_comparison) {
        printf("%d %d", global_signed_comparison, local_signed_comparison);
        return 24;
    }
    
    // Check hex and octal
    if (global_hex != local_hex) {

        return 25;
    }
    
    if (global_octal != local_octal) {

        return 26;
    }
    
    // Check shifting
    if (global_shift_left != local_shift_left) {

        return 27;
    }
    
    if (global_shift_right != local_shift_right) {

        return 28;
    }
    
    if (global_shift_negative != local_shift_negative) {

        return 29;
    }
    
    if (global_shift_overflow != local_shift_overflow) {

        return 30;
    }

    return 0;
}