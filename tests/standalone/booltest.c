#include <stdbool.h>

int test_boolean_promotion() {
    bool b = true;
    if (~b != -2) {
        return 1; // Test failed
    }
    return 0; // Test passed
}

int test_boolean_zero() {
    bool b = false;
    if (~b != -1) {
        return 1; // Test failed
    }
    return 0; // Test passed
}

int test_boolean_large_value() {
    bool b = 255; // Any non-zero value is treated as 1 in bool
    if (~b != -2) {
        return 1; // Test failed
    }
    return 0; // Test passed
}

int main() {
    if (test_boolean_promotion()) {
        return 1;
    }
    if (test_boolean_zero()) {
        return 1;
    }
    if (test_boolean_large_value()) {
        return 1;
    }
    return 0;
}