/**
 * unsigned_division_test.c
 * 
 * Tests compiler's unsigned division operations.
 * Returns 0 on success, non-zero on failure.
 */

 #include <stdint.h>
 #include <limits.h>
 
 int main() {
     int failed_tests = 0;
     
     // Basic cases
     if ((10 / 2 != 5) || (10 % 2 != 0)) failed_tests++;
     if ((10 / 3 != 3) || (10 % 3 != 1)) failed_tests++;
     
     // Power of 2 divisors (often optimized differently)
     if ((1024 / 16 != 64) || (1024 % 16 != 0)) failed_tests++;
     if ((1023 / 16 != 63) || (1023 % 16 != 15)) failed_tests++;
     
     // Boundary values
     if ((UINT_MAX / 1 != UINT_MAX) || (UINT_MAX % 1 != 0)) failed_tests++;
     if ((UINT_MAX / 2 != UINT_MAX/2) || (UINT_MAX % 2 != 1)) failed_tests++;
     if ((UINT_MAX / UINT_MAX != 1) || (UINT_MAX % UINT_MAX != 0)) failed_tests++;
     if ((UINT_MAX / (UINT_MAX-1) != 1) || (UINT_MAX % (UINT_MAX-1) != 1)) failed_tests++;
     if (((UINT_MAX-1) / UINT_MAX != 0) || ((UINT_MAX-1) % UINT_MAX != UINT_MAX-1)) failed_tests++;
     
     // Division by large values
     if ((1 / UINT_MAX != 0) || (1 % UINT_MAX != 1)) failed_tests++;
     if (((UINT_MAX-1) / UINT_MAX != 0) || ((UINT_MAX-1) % UINT_MAX != UINT_MAX-1)) failed_tests++;
     
     // Zero dividend
     if ((0 / 1 != 0) || (0 % 1 != 0)) failed_tests++;
     if ((0 / UINT_MAX != 0) || (0 % UINT_MAX != 0)) failed_tests++;
     
     // Powers of 2
     if ((0x80000000 / 0x8000 != 0x10000) || (0x80000000 % 0x8000 != 0)) failed_tests++;
     if ((0x80000001 / 0x8000 != 0x10000) || (0x80000001 % 0x8000 != 1)) failed_tests++;
     
     return failed_tests;
 }