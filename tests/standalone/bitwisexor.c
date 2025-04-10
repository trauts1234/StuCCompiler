int main() {
    // Basic XOR operations
    if ((0 ^ 0) != 0) return 1;
    if ((0 ^ 1) != 1) return 2;
    if ((1 ^ 0) != 1) return 3;
    if ((1 ^ 1) != 0) return 4;
    if ((1 ^ 2) != 3) return 5;
    if ((1024 ^ 0) != 1024) return 6;

    // Tests with unsigned integers
    unsigned int u1 = 1, u2 = 2;
    if ((u1 ^ u2) != 3) return 7;

    // Tests with short integers
    short s1 = 4, s2 = 8;
    // 4 (0100) ^ 8 (1000) = 12 (1100)
    if ((s1 ^ s2) != 12) return 8;

    // Test type promotion with char (promoted to int)
    char c1 = 1, c2 = 2;
    if ((c1 ^ c2) != 3) return 9;

    // Mixing signed and unsigned types
    if ((1 ^ 2u) != 3) return 10;

    // Test with negative numbers
    // Assuming two's complement representation:
    // -1 (all bits set) ^ 0 = -1 and vice versa.
    if (((-1) ^ 0) != -1) return 11;
    if ((0 ^ (-1)) != -1) return 12;
    // -1 ^ -1 should equal 0 since all bits cancel out.
    if (((-1) ^ (-1)) != 0) return 13;

    // Additional tests for bit patterns
    // 5 (0101) ^ 3 (0011) should equal 6 (0110)
    if ((5 ^ 3) != 6) return 14;
    // 10 (1010) ^ 4 (0100) should equal 14 (1110)
    if ((10 ^ 4) != 14) return 15;

    return 0;
}
