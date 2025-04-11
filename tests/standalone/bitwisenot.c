
int main() {
    // Test 1: Basic bitwise NOT of 0
    {
        int x = 0;
        if (~x != -1) {
            return 1;
        }
    }
    
    // Test 2: Basic bitwise NOT of -1
    {
        int x = -1;
        if (~x != 0) {
            return 2;
        }
    }
    
    // Test 3: Bitwise NOT of 1
    {
        int x = 1;
        // ~00000001 = 11111110 (two's complement)
        if (~x != -2) {
            return 3;
        }
    }
    
    // Test 4: Double bitwise NOT (should return original value)
    {
        int x = 42;
        if (~~x != x) {
            return 4;
        }
    }
    
    // Test 5: Bitwise NOT of INT_MAX
    {
        int x = 2147483647;  // 01111...1
        // ~INT_MAX = 10000...0 = INT_MIN
        if (~x != -2147483648) {
            return 5;
        }
    }
    
    // Test 6: Bitwise NOT of INT_MIN
    {
        int x = -2147483647 -1;  // 10000...0
        // ~INT_MIN = 01111...1 = INT_MAX
        if (~x != 2147483647) {
            return 6;
        }
    }
    
    // Test 7: Bitwise NOT of unsigned int
    {
        unsigned int x = 1;
        // For unsigned int with 32 bits: ~1 = UINT_MAX - 1
        if (~x != 2147483647 *2U +1U - 1) {
            return 7;
        }
    }
    
    // Test 8: Bitwise NOT of unsigned 0
    {
        unsigned int x = 0;
        if (~x != 2147483647 *2U +1U) {
            return 8;
        }
    }
    
    // Test 9: Bitwise NOT of unsigned max value
    {
        unsigned int x = 2147483647 *2U +1U;
        if (~x != 0) {
            return 9;
        }
    }
    
    // Test 10: Char promotion to int before bitwise NOT
    {
        char c = 1;  // 00000001 in 8 bits
        // When promoted to int (assuming 32-bit int): 00000000 00000000 00000000 00000001
        // After NOT: 11111111 11111111 11111111 11111110 (-2 in two's complement)
        if (~c != -2) {
            return 10;
        }
    }
    
    // Test 11: Unsigned char promotion to int before bitwise NOT
    {
        unsigned char c = 1;
        // When promoted to int: 00000000 00000000 00000000 00000001
        // After NOT: 11111111 11111111 11111111 11111110 (-2 in two's complement)
        if (~c != -2) {
            return 11;
        }
    }
    
    // Test 12: Short promotion to int before bitwise NOT
    {
        short s = 1;
        if (~s != -2) {
            return 12;
        }
    }
    
    // Test 13: Unsigned short promotion to int before bitwise NOT
    {
        unsigned short s = 1;
        if (~s != -2) {
            return 13;
        }
    }
    
    // Test 14: Bitwise NOT of specific bit pattern
    {
        int x = 1431655765;  // 01010101...
        if (~x != 2863311530U) {  // 10101010...
            return 14;
        }
    }
    
    // Test 15: Bitwise NOT with 8-bit integer
    {
        char x = 15;  // 00001111
        // After NOT: 11110000 (which is -16 in two's complement for 8-bit)
        if (~x != -16) {
            return 15;
        }
    }
    
    // Test 16: Bitwise NOT with 16-bit integer
    {
        short x = 255;  // 0000000011111111
        // After NOT: 1111111100000000 (which is -256 in two's complement for 16-bit)
        if (~x != -256) {
            return 16;
        }
    }
    
    // Test 17: Bitwise NOT with 64-bit integer
    {
        signed long long int x = 1;
        // For 64-bit int: ~1 = -2 in two's complement
        if (~x != -2) {
            return 17;
        }
    }
    
    // Test 18: Bitwise NOT with unsigned 64-bit
    {
        unsigned long long int x = 1;
        if (~x != 18446744073709551615UL - 1) {
            return 18;
        }
    }
    
    // Test 19: Bitwise NOT in compound expression
    {
        int x = 5, y = 3;
        // ~5 & 3 = (-6) & 3 = 2
        if ((~x & y) != 2) {
            return 19;
        }
    }
    
    // Test 20: Bitwise NOT with assignment
    {
        int x = 42;
        x = ~x;
        if (x != -43) {
            return 20;
        }
    }
    
    // Test 21: Bitwise NOT of boolean values
    {
        _Bool b = 1;
        // _Bool is promoted to int (1) before ~, result should be -2
        if (~b != -2) {
            return 21;
        }
    }
    
    // Test 22: Bitwise NOT combined with shift operation
    {
        int x = 1;
        // ~(1 << 3) = ~(8) = -9
        if (~(x << 3) != -9) {
            return 22;
        }
    }
    
    // Test 23: Interaction between ~ and !
    {
        int x = 1;
        // ~(!0) = ~0 = -1
        if (~(!x) != -1) {
            return 23;
        }
    }
    
    // All tests passed
    return 0;
}