int main() {
    // Basic AND operations
    if ((0 & 0) != 0) return 1;
    if ((0 & 1) != 0) return 2;
    if ((1 & 0) != 0) return 3;
    if ((1 & 1) != 1) return 4;
    if ((1 & 2) != 0) return 5;
    if ((1024 & 0) != 0) return 6;

    // Tests with unsigned integers
    // Example: 3 (0011) & 6 (0110) should equal 2 (0010)
    unsigned int u1 = 3, u2 = 6;
    if ((u1 & u2) != 2) return 7;

    // Tests with short integers
    // Example: 5 (0101) & 3 (0011) should equal 1 (0001)
    short s1 = 5, s2 = 3;
    if ((s1 & s2) != 1) return 8;

    // Test type promotion with char (promoted to int)
    // Example: 15 (1111) & 6 (0110) should equal 6 (0110)
    char c1 = 15, c2 = 6;
    if ((c1 & c2) != 6) return 9;

    // Mixing signed and unsigned types
    // 1 (0001) & 2u (0010) should equal 0 (0000)
    if ((1 & 2u) != 0) return 10;

    // Test with negative numbers (bitwise AND on negative values)
    // Assuming two's complement: -1 has all bits set, so:
    // (-1 & 0) should equal 0, and (0 & -1) should equal 0.
    if (((-1) & 0) != 0) return 11;
    if ((0 & (-1)) != 0) return 12;

    // Additional tests for bit patterns
    // 5 (0101) & 3 (0011) should equal 1 (0001)
    if ((5 & 3) != 1) return 13;
    // 10 (1010) & 4 (0100) should equal 0 (0000)
    if ((10 & 4) != 0) return 14;

    // Edge case: all bits set
    // (-1 & -1) should yield -1
    if ((-1 & -1) != -1) return 15;

    return 0;
}