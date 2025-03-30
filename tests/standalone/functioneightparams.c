int func(int a, int b, char c, int d, int e, int f, int s1, char s2) {
    if(s1 != 7){
        return 1;
    }
    if(s2 != 8) {
        return 2;
    }

    if (a != 1){
        return 3;
    }

    if (b != 2){
        return 4;
    }
    return 0;
}

int main() {
    int result = func(1,2,3,4,5,6, 7,8);

    return result;
}