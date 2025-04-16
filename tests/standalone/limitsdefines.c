#include <limits.h>

int main() {
    // Check CHAR_BIT
    if (CHAR_BIT != 8) {
        return 1;
    }
    
    // Check SCHAR_MIN
    if (SCHAR_MIN != -128) {
        return 1;
    }
    
    // Check SCHAR_MAX
    if (SCHAR_MAX != 127) {
        return 1;
    }
    
    // Check UCHAR_MAX
    if (UCHAR_MAX != 255) {
        return 1;
    }
    
    // Check SHRT_MIN
    if (SHRT_MIN != -32768) {
        return 1;
    }
    
    // Check SHRT_MAX
    if (SHRT_MAX != 32767) {
        return 1;
    }
    
    // Check USHRT_MAX
    if (USHRT_MAX != 65535) {
        return 1;
    }
    
    // Check INT_MIN
    if (INT_MIN != -2147483648) {
        return 1;
    }
    
    // Check INT_MAX
    if (INT_MAX != 2147483647) {
        return 1;
    }
    
    // Check UINT_MAX
    if (UINT_MAX != 4294967295U) {
        return 1;
    }
    
    // Check LONG_MIN (for x64 Linux, long is 64-bit)
    if (LONG_MIN != -9223372036854775807L - 1) {
        return 1;
    }
    
    // Check LONG_MAX (for x64 Linux, long is 64-bit)
    if (LONG_MAX != 9223372036854775807L) {
        return 1;
    }
    
    // Check ULONG_MAX (for x64 Linux, unsigned long is 64-bit)
    if (ULONG_MAX != 18446744073709551615UL) {
        return 1;
    }
    
    // Check LLONG_MIN
    if (LLONG_MIN != -9223372036854775807LL - 1) {
        return 1;
    }
    
    // Check LLONG_MAX
    if (LLONG_MAX != 9223372036854775807LL) {
        return 1;
    }
    
    // Check ULLONG_MAX
    if (ULLONG_MAX != 18446744073709551615ULL) {
        return 1;
    }
    
    // All checks passed
    return 0;
}