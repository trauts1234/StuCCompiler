int main() {
    // Test basic left shifts
    if ((1 << 0) != 1) return 1;
    if ((1 << 1) != 2) return 2;
    if ((1 << 2) != 4) return 3;
    if ((1 << 3) != 8) return 4;
    if ((1 << 4) != 16) return 5;
    
    // Test basic right shifts
    if ((8 >> 1) != 4) return 6;
    if ((8 >> 2) != 2) return 7;
    if ((8 >> 3) != 1) return 8;
    if ((8 >> 4) != 0) return 9;

    // Test shifts with larger numbers
    if ((256 << 1) != 512) return 10;
    if ((256 >> 1) != 128) return 11;
    
    // Test multiple shifts
    if ((1 << 3 << 2) != 32) return 12;
    if ((32 >> 2 >> 1) != 4) return 13;

    // Test shifts with negative numbers
    if ((-8 >> 1) >= 0) return 14;
    if ((-8 << 1) >= 0) return 15;

    // Test shifts with zero
    if ((0 << 5) != 0) return 16;
    if ((0 >> 5) != 0) return 17;

    // Test shifts with maximum bits
    if ((1 << 31) >= 0) return 18;
    
    // Test shifts with variables
    int x = 1;
    int y = 3;
    if ((x << y) != 8) return 19;
    if ((8 >> y) != 1) return 20;

    // Test precedence
    if ((1 + 2 << 1) != 6) return 23;
    if ((4 >> 1 + 1) != 1) return 24;

    //test type promotion

    char z = 64;

    if(z << 3 != 512) {
        return 25;
    }

    return 0;
}