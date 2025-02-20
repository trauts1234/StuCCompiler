#ifdef x
int foo() {
    return 1;
}
#else
int foo() {
    return 69;
}
#endif

int main(){
    return foo();
}