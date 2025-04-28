/**
 * Comprehensive test for C struct shadowing behavior
 * 
 * This test systematically verifies:
 * 1. Nested scope struct shadowing
 * 2. Multiple levels of shadow definitions
 * 3. Member access across shadowed definitions
 * 4. Array of structs behavior with shadowing
 * 5. Function parameters with shadowed structs
 * 6. Precise error identification through return codes
 */

 #include <stddef.h>
 #include <string.h>
 
 // Global struct definition
 struct Vector {
     int x, y;
 };
 
 // Function using the original struct definition
 int test_function_parameter(struct Vector v) {
     return v.x + v.y;
 }
 
 // Forward declaration
 struct Forward;
 
 /**
  * Memory validation function: verifies memory regions without relying on sizeof
  * Returns non-zero on failure
  */
 int validate_struct_memory(void* ptr1, void* ptr2, int elem_count) {
     unsigned char* p1 = (unsigned char*)ptr1;
     unsigned char* p2 = (unsigned char*)ptr2;
     
     for (int i = 0; i < elem_count; i++) {
         if (p1[i] != p2[i]) {
             return 1;
         }
     }
     return 0;
 }
 
 // Function scope shadowing test
 int test_function_scope() {
     struct Vector {
         int id;
         char type;
     };
     
     struct Vector func_struct;
     func_struct.id = 555;
     func_struct.type = 'F';
     return func_struct.id;
 }
 
 int main() {
     // Test 1: Original struct definition verification
     struct Vector original;
     original.x = 10;
     original.y = 20;
     
     // Prepare expected memory pattern for later comparison
     unsigned char expected_memory[8];
     memcpy(expected_memory, &original, 8);
     
     // Test 2: Basic shadowing
     {
         struct Vector {
             int a, b;
         };
         
         struct Vector shadowed;
         shadowed.a = 30;
         shadowed.b = 40;
         
         // Verify both structs are accessible and correct
         if (original.x != 10 || original.y != 20) {
             return 20;
         }
         
         if (shadowed.a != 30 || shadowed.b != 40) {
             return 21;
         }
         
         // Test function with original struct
         if (test_function_parameter(original) != 30) {
             return 22;
         }
         
         // Verify memory layout differs without using sizeof
         unsigned char shadowed_memory[8];
         memcpy(shadowed_memory, &shadowed, 8);
         
         // Memory should have different patterns despite similar structure
         // because field access is based on type definition
         if (memcmp(expected_memory, shadowed_memory, 8) == 0) {
             return 23;
         }
     }
     
     // Test 3: Multiple levels of shadowing
     {
         struct Vector {
             char c;
             int i;
             char pad[3]; // Padding to ensure consistent memory layout
         };
         
         struct Vector level1;
         level1.c = 'A';
         level1.i = 100;
         
         {
             struct Vector {
                 long l1;
                 long l2;
             };
             
             struct Vector level2;
             level2.l1 = 1000;
             level2.l2 = 2000;
             
             // Check original struct still intact
             if (original.x != 10 || original.y != 20) {
                 return 31;
             }
             
             // Check level1 struct still intact
             if (level1.c != 'A' || level1.i != 100) {
                 return 32;
             }
             
             // Check level2 struct values
             if (level2.l1 != 1000 || level2.l2 != 2000) {
                 return 33;
             }
             
             // Verify memory layouts differ between level1 and level2
             unsigned char level1_memory[8];
             unsigned char level2_memory[16];
             memcpy(level1_memory, &level1, 8);
             memcpy(level2_memory, &level2, 16);
             
             // Memory patterns should differ
             if (memcmp(level1_memory, level2_memory, 8) == 0) {
                 return 34;
             }
         }
         
         // After inner scope, we should be back to level1 definition
         struct Vector test_level1;
         test_level1.c = 'B';
         test_level1.i = 200;
         
         if (test_level1.c != 'B' || test_level1.i != 200) {
             return 35;
         }
     }
     
     // Test 4: Array of structs with shadowing
     {
         struct Vector array[3];
         for (int i = 0; i < 3; i++) {
             array[i].x = i * 10;
             array[i].y = i * 20;
         }
         
         {
             struct Vector {
                 long a, b, c;
             };
             
             // Verify original array still intact
             for (int i = 0; i < 3; i++) {
                 if (array[i].x != i * 10 || array[i].y != i * 20) {
                     return 40;
                 }
             }
             
             // Create array with shadowed definition
             struct Vector shadowed_array[2];
             shadowed_array[0].a = 100;
             shadowed_array[0].b = 200;
             shadowed_array[0].c = 300;
             shadowed_array[1].a = 400;
             shadowed_array[1].b = 500;
             shadowed_array[1].c = 600;
             
             // Verify shadowed array
             if (shadowed_array[0].a != 100 || shadowed_array[0].b != 200 || shadowed_array[0].c != 300 ||
                 shadowed_array[1].a != 400 || shadowed_array[1].b != 500 || shadowed_array[1].c != 600) {
                 return 41;
             }
             
             // Verify memory patterns differ between array types
             // Comparing just 8 bytes should be sufficient to detect differences
             unsigned char original_array_mem[8];
             unsigned char shadowed_array_mem[8];
             memcpy(original_array_mem, &array[0], 8);
             memcpy(shadowed_array_mem, &shadowed_array[0], 8);
             
             if (memcmp(original_array_mem, shadowed_array_mem, 8) == 0) {
                 return 42;
             }
         }
     }
     
     // Test 5: Forward declaration and shadowing
     {
         struct Forward {
             int value;
         };
         
         struct Forward fwd;
         fwd.value = 42;
         
         {
             struct Forward {
                 char *name;
                 int id;
             };
             
             struct Forward shadowed_fwd;
             shadowed_fwd.name = "test";
             shadowed_fwd.id = 99;
             
             // Verify both structs are correct
             if (fwd.value != 42) {
                 return 50;
             }
             
             if (strcmp(shadowed_fwd.name, "test") != 0 || shadowed_fwd.id != 99) {
                 return 51;
             }
         }
     }
     
     // Test 6: Test struct with identical field names but different types
     {
         struct Vector {
             long x, y;  // Same field names but different types
         };
         
         struct Vector long_vec;
         long_vec.x = 1000;
         long_vec.y = 2000;
         
         // Check original struct still has integer values
         if (original.x != 10 || original.y != 20) {
             return 60;
         }
         
         // Check the shadowed struct has long values
         if (long_vec.x != 1000 || long_vec.y != 2000) {
             return 61;
         }
         
         // Verify memory patterns differ without using sizeof
         unsigned char original_mem[8];
         unsigned char long_vec_mem[16];  // Assuming long is 8 bytes
         memcpy(original_mem, &original, 8);
         memcpy(long_vec_mem, &long_vec, 16);
         
         if (memcmp(original_mem, long_vec_mem, 8) == 0) {
             return 62;
         }
     }
     
     // Test 7: Verify field offsets differ between shadowed structs
     {
         struct MixedFields {
             char a;
             int b;
             char c;
         };
         
         struct MixedFields mix1;
         mix1.a = 'X';
         mix1.b = 100;
         mix1.c = 'Y';
         
         // Calculate offsets manually without offsetof
         char* base_addr = (char*)&mix1;
         ptrdiff_t offset_b_original = (char*)&mix1.b - base_addr;
         ptrdiff_t offset_c_original = (char*)&mix1.c - base_addr;
         
         {
             struct MixedFields {
                 int b;
                 char a;
                 char c;
             };
             
             struct MixedFields mix2;
             mix2.b = 200;
             mix2.a = 'Z';
             mix2.c = 'W';
             
             // Calculate offsets in shadowed struct
             char* shadowed_base = (char*)&mix2;
             ptrdiff_t offset_b_shadowed = (char*)&mix2.b - shadowed_base;
             ptrdiff_t offset_c_shadowed = (char*)&mix2.c - shadowed_base;
             
             // Verify offsets are different
             if (offset_b_original == offset_b_shadowed) {
                 return 70;
             }
             
             if (offset_c_original == offset_c_shadowed) {
                 return 71;
             }
         }
     }
     
     // Test 8: Function scope vs global scope shadowing
     int function_result = test_function_scope();
     if (function_result != 555) {
         return 80;
     }
     
     // Ensure original struct definition is still intact after function call
     if (original.x != 10 || original.y != 20) {
         return 81;
     }
     
     // Test 9: Shadowing with typedefs
     {
         typedef struct {
             int count;
             char code;
         } Vector;
         
         Vector typedef_struct;
         typedef_struct.count = 123;
         typedef_struct.code = 'T';
         
         // Original struct should still be intact
         if (original.x != 10 || original.y != 20) {
             return 90;
         }
         
         // Typedef struct should have correct values
         if (typedef_struct.count != 123 || typedef_struct.code != 'T') {
             return 91;
         }
     }
     
     return 0;  // Success
 }