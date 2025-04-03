extern int check_stack_aligned();

void f(int a, int b, int c) {
    check_stack_aligned();
}

int main() {
    int x = check_stack_aligned();

    check_stack_aligned();//check one variable doesn't break stack

    check_stack_aligned();//check non-handled return value doesn't break the stack

    1 + check_stack_aligned();
    1 - check_stack_aligned();
    1 * check_stack_aligned();
    1 / check_stack_aligned();
    1 % check_stack_aligned();


    f(check_stack_aligned(), 2, check_stack_aligned());//check when params pushed, stack stays aligned for nested function calls
    
}