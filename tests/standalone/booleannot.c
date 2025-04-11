int main() {
    // Test 1: Basic boolean NOT of 0
    {
        _Bool x = 0;
        if (!x != 1) {
            return 1;
        }
    }
    
    // Test 2: Basic boolean NOT of 1
    {
        _Bool x = 1;
        if (!x != 0) {
            return 2;
        }
    }
    
    // Test 3: NOT of integer 0
    {
        int x = 0;
        if (!x != 1) {
            return 3;
        }
    }
    
    // Test 4: NOT of positive integer
    {
        int x = 42;
        if (!x != 0) {
            return 4;
        }
    }
    
    // Test 5: NOT of negative integer
    {
        int x = -42;
        if (!x != 0) {
            return 5;
        }
    }
    
    // Test 6: Double NOT (negation of negation)
    {
        int x = 42;
        if (!!x != 1) {
            return 6;
        }
    }
    
    // Test 7: Double NOT of zero
    {
        int x = 0;
        if (!!x != 0) {
            return 7;
        }
    }
    
    // Test 8: NOT in expressions
    {
        int x = 10, y = 0;
        if ((!x || y) != 0) {
            return 8;
        }
    }
    
    // Test 9: NOT with AND
    {
        int x = 10, y = 20;
        if ((!x && y) != 0) {
            return 9;
        }
    }
    
    // Test 10: Multiple NOTs
    {
        int x = 1;
        if (!!!x != 0) {
            return 10;
        }
    }
    
    // Test 11: NOT of INT_MAX
    {
        int x = 2147483647;
        if (!x != 0) {
            return 11;
        }
    }
    
    // Test 12: NOT of INT_MIN
    {
        int x = (2147483647+1);
        if (!x != 0) {
            return 12;
        }
    }
    
    // Test 14: NOT of a pointer
    {
        int dummy = 0;
        int *p = &dummy;
        if (!p != 0) {
            return 14;
        }
    }
    
    // Test 15: NOT of NULL pointer
    {
        int *p = 0;
        if (!p != 1) {
            return 15;
        }
    }
    
    // Test 16: NOT in assignment
    {
        int x = 42;
        int y = !x;
        if (y != 0) {
            return 16;
        }
    }
    
    // Test 18: NOT of character
    {
        char c = 'A';  // ASCII 65
        if (!c != 0) {
            return 18;
        }
    }
    
    // Test 19: NOT of '\0' character
    {
        char c = '\0';
        if (!c != 1) {
            return 19;
        }
    }
    
    // Test 20: NOT operation precedence
    {
        int x = 5, y = 10;
        // ! has higher precedence than ==
        if ((!x == !y) == 0) {
            return 20;
        }
    }
    
    // All tests passed
    return 0;
}