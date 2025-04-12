int main() {
    
    // Basic octal literals
    if (!(010 == 8)) {
        return 1;
    }
    if (!(0123 == 83)) {
        return 1;
    }
    if (!(00 == 0)) {
        return 1;
    }
    
    // Basic hex literals
    if (!(0x10 == 16)) {
        return 1;
    }
    if (!(0X10 == 16)) {
        return 1;
    }
    if (!(0xA == 10)) {
        return 1;
    }
    if (!(0xa == 10)) {
        return 1;
    }
    if (!(0x0 == 0)) {
        return 1;
    }
    
    // Mixed case for hex literals
    if (!(0xAbCd == 43981)) {
        return 1;
    }
    if (!(0XaBcD == 43981)) {
        return 1;
    }
    
    // Large values
    if (!(0xFFFFFFFF == 0xFFFFFFFF)) {
        return 1;
    }
    if (!(037777777777 == 0xFFFFFFFF)) {
        return 1;
    }
    
    // All hex digits
    if (!(0x0123456789abcdef == 0x0123456789abcdef)) {
        return 1;
    }
    if (!(0x0123456789ABCDEF == 0x0123456789ABCDEF)) {
        return 1;
    }
    
    // Octal with suffix
    if (!(010L == 8L)) {
        return 1;
    }
    if (!(010LL == 8LL)) {
        return 1;
    }
    if (!(010U == 8U)) {
        return 1;
    }
    if (!(010UL == 8UL)) {
        return 1;
    }
    if (!(010ULL == 8ULL)) {
        return 1;
    }
    
    // Hex with suffix
    if (!(0x10L == 16L)) {
        return 1;
    }
    if (!(0x10LL == 16LL)) {
        return 1;
    }
    if (!(0x10U == 16U)) {
        return 1;
    }
    if (!(0x10UL == 16UL)) {
        return 1;
    }
    if (!(0x10ULL == 16ULL)) {
        return 1;
    }

    if (!(0b1010 == 10)) {
        return 1;
    }
    if (!(0B1010 == 10)) {
        return 1;
    }
    
    // Test hex literals with full uppercase/lowercase variation
    if (!(0xDEADBEEF == 0xdeadbeef)) {
        return 1;
    }
    
    // Test octal literal with leading zeros
    if (!(00001 == 1)) {
        return 1;
    }
    if (!(0000 == 0)) {
        return 1;
    }
    
    // Test max values for different integer types
    if (!(0x7FFFFFFF == 2147483647)) {
        return 1;
    }
    if (!(017777777777 == 2147483647)) {
        return 1;
    }
    
    return 0;
}