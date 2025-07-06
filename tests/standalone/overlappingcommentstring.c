#include <string.h>
/*
"hello world */

char* x = "/* */";

int main() {
    if (strlen(x) != 5){return 1;}
    return 0;
}

#if 1/* multiline
*/

#elif 0 //single comment

/*#error //panic!
*/

#endif