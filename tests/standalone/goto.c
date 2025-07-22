#include <stdio.h>
#include <stdlib.h>

/* Test 1: Basic label and goto functionality */
int test_basic_goto(void) {
    int result = 1;
    
    goto skip_error;
    result = 0;  /* This should be skipped */
    return result;
    
skip_error:
    result = 0;  /* This should execute */
    return result;
}

/* Test 2: Forward and backward jumps */
int test_forward_backward_goto(void) {
    int counter = 0;
    
    goto forward;
    
backward:
    counter++;
    if (counter < 3) {
        goto forward;
    }
    goto end;
    
forward:
    counter++;
    if (counter == 1) {
        goto backward;
    }
    if (counter < 5) {
        goto backward;
    }
    
end:
    /* Should have counter = 5 at this point */
    return (counter != 4);
}

/* Test 3: Same label names in different functions (should be allowed) */
int test_function_a(void) {
    goto local_label;
    return 1;  /* Error if reached */
    
local_label:
    return 0;  /* Success */
}

int test_function_b(void) {
    goto local_label;  /* Same label name as in test_function_a */
    return 1;  /* Error if reached */
    
local_label:
    return 0;  /* Success */
}

/* Test 4: Multiple labels in sequence */
int test_multiple_labels(void) {
    int step = 0;
    
    goto label1;
    return 1;  /* Should not reach here */
    
label1:
label2:
label3:
    step = 1;
    goto label4;
    return 1;  /* Should not reach here */
    
label4:
    step = 2;
    goto end_labels;
    
end_labels:
    return (step != 2);
}

/* Test 5: Labels inside nested blocks */
int test_nested_blocks(void) {
    int result = 1;
    
    {
        goto inner_label;
        result = 0;  /* Should not execute */
        
    inner_label:
        {
            result = 0;  /* Should execute */
            goto outer_label;
        }
        result = 1;  /* Should not execute */
    }
    result = 1;  /* Should not execute */
    
outer_label:
    return result;  /* Should return 0 */
}

/* Test 6: Jump over variable declarations (potential undefined behavior test) */
int test_jump_over_declarations(void) {
    goto skip_decl;
    
    int local_var = 42;  /* This declaration is jumped over */
    
skip_decl:
    /* Note: Using local_var here would be undefined behavior in C,
       so we don't test that - we just test that the jump works */
    return 0;
}

/* Test 7: Long distance jump within function */
int test_long_jump(void) {
    int i;
    
    goto very_far_label;
    
    /* Add some distance */
    for (i = 0; i < 100; i++) {
        if (i == 50) {
            /* This should never execute */
            return 1;
        }
    }
    
    /* More distance */
    i = i + 1;
    i = i * 2;
    i = i - 3;
    
    return 1;  /* Should not reach here */
    
very_far_label:
    return 0;  /* Success */
}

/* Test 8: Label at end of function */
int test_end_label(void) {
    goto end;
    return 1;  /* Should not execute */
    
end:
    return 0;
}

/* Main test runner */
int main(void) {
    
    if (test_basic_goto() != 0) {
        printf("FAIL: Basic goto test failed\n");
        return 1;
    }
    
    if (test_forward_backward_goto() != 0) {
        printf("FAIL: Forward/backward goto test failed\n");
        return 2;
    }
    
    if (test_function_a() != 0 || test_function_b() != 0) {
        printf("FAIL: Same label names in different functions test failed\n");
        return 3;
    }
    
    if (test_multiple_labels() != 0) {
        printf("FAIL: Multiple labels test failed\n");
        return 4;
    }
    
    if (test_nested_blocks() != 0) {
        printf("FAIL: Nested blocks test failed\n");
        return 5;
    }
    
    if (test_jump_over_declarations() != 0) {
        printf("FAIL: Jump over declarations test failed\n");
        return 6;
    }
    
    if (test_long_jump() != 0) {
        printf("FAIL: Long jump test failed\n");
        return 7;
    }
    
    if (test_end_label() != 0) {
        printf("FAIL: End label test failed\n");
        return 8;
    }
    return 0;
}