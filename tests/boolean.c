int main() {
    _Bool x = 0;

    if(x){
        return 1;
    }

    x = 1;
    if(x != 1) {
        return 2;
    }

    x = 2;//boolean converts to 1
    if(x != 1){
        return 3;
    }

    return 0;
}