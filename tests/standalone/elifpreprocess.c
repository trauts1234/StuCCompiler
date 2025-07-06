#include <stdio.h>

/* Test 1: Basic #elif functionality */
#define TEST1 2

#if TEST1 == 1
    #define RESULT1 "FAIL: TEST1 == 1"
#elif TEST1 == 2
    #define RESULT1 "PASS: TEST1 == 2"
#elif TEST1 == 3
    #define RESULT1 "FAIL: TEST1 == 3"
#else
    #define RESULT1 "FAIL: TEST1 default"
#endif

/* Test 2: Multiple #elif chains */
#define TEST2 4

#if TEST2 == 1
    #define RESULT2 "FAIL"
#elif TEST2 == 2
    #define RESULT2 "FAIL"
#elif TEST2 == 3
    #define RESULT2 "FAIL"
#elif TEST2 == 4
    #define RESULT2 "PASS"
#elif TEST2 == 5
    #define RESULT2 "FAIL"
#else
    #define RESULT2 "FAIL"
#endif

/* Test 3: #elif with comments and whitespace */
#define TEST3 7

#if TEST3 == 6 /* comment here */
    #define RESULT3 "FAIL"
#elif /* comment */ TEST3 == 7 /* another comment */
    #define RESULT3 "PASS"
#elif TEST3 == 8
    #define RESULT3 "FAIL"
#endif

/* Test 4: Nested #if/#elif inside #elif */
#define OUTER 2
#define INNER 3

#if OUTER == 1
    #define RESULT4 "FAIL: OUTER == 1"
#elif OUTER == 2
    #if INNER == 1
        #define RESULT4 "FAIL: INNER == 1"
    #elif INNER == 2
        #define RESULT4 "FAIL: INNER == 2"
    #elif INNER == 3
        #define RESULT4 "PASS: INNER == 3"
    #else
        #define RESULT4 "FAIL: INNER default"
    #endif
#else
    #define RESULT4 "FAIL: OUTER default"
#endif

/* Test 5: #elif with complex expressions */
#define A 5
#define B 3

#if (A + B) == 7
    #define RESULT5 "FAIL: A + B == 7"
#elif (A * B) == 15
    #define RESULT5 "PASS: A * B == 15"
#elif (A - B) == 2
    #define RESULT5 "FAIL: A - B == 2"
#endif

/* Test 6: #elif with defined() operator */
#define DEFINED_MACRO 1
#undef UNDEFINED_MACRO

#if defined(UNDEFINED_MACRO)
    #define RESULT6 "FAIL: UNDEFINED_MACRO defined"
#elif defined(DEFINED_MACRO)
    #define RESULT6 "PASS: DEFINED_MACRO defined"
#else
    #define RESULT6 "FAIL: neither defined"
#endif


/* Test 8: #elif with logical operators */
#define X 10
#define Y 20

#if X > 15 && Y > 15
    #define RESULT8 "FAIL: both > 15"
#elif X < 15 && Y > 15
    #define RESULT8 "PASS: X < 15 && Y > 15"
#elif X > 15 || Y < 15
    #define RESULT8 "FAIL: X > 15 || Y < 15"
#else
    #define RESULT8 "FAIL: default"
#endif

/* Test 9: #elif with #ifdef and #ifndef */
#define SOME_MACRO

#ifdef SOME_MACRO
    #if 0
        #define RESULT9 "FAIL: unreachable"
    #elif 1
        #define RESULT9 "PASS: elif 1"
    #endif
#else
    #define RESULT9 "FAIL: SOME_MACRO not defined"
#endif



/* Test 12: #elif with character constants */
#define CHAR_TEST 'B'

#if CHAR_TEST == 'A'
    #define RESULT12 "FAIL: A"
#elif CHAR_TEST == 'B'
    #define RESULT12 "PASS: B"
#elif CHAR_TEST == 'C'
    #define RESULT12 "FAIL: C"
#endif

int main() {
    
    printf("Testing #elif preprocessor directive handling...\n\n");
    
    printf("Test 1 - Basic #elif: %s\n", RESULT1);
    if (RESULT1[0] == 'F') return 1;;
    
    printf("Test 2 - Multiple #elif: %s\n", RESULT2);
    if (RESULT2[0] == 'F') return 2;;
    
    printf("Test 3 - Comments/whitespace: %s\n", RESULT3);
    if (RESULT3[0] == 'F') return 3;
    
    printf("Test 4 - Nested conditionals: %s\n", RESULT4);
    if (RESULT4[0] == 'F') return 4;;
    
    printf("Test 5 - Complex expressions: %s\n", RESULT5);
    if (RESULT5[0] == 'F') return 5;;
    
    printf("Test 6 - defined() operator: %s\n", RESULT6);
    if (RESULT6[0] == 'F') return 6;;
    
    printf("Test 8 - Logical operators: %s\n", RESULT8);
    if (RESULT8[0] == 'F') return 7;;
    
    printf("Test 9 - ifdef/ifndef: %s\n", RESULT9);
    if (RESULT9[0] == 'F') return 8;;
    
    printf("Test 12 - Character constants: %s\n", RESULT12);
    if (RESULT12[0] == 'F') return 9;;

    return 0;
}