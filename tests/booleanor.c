int main() {
    if(0 || 0) {
        return 1;
    }

    _Bool a=0, b=0;

    if(a || b) {
        return 2;
    }

    if((a || b) == 1) {
        return 3;
    }

    if(0 || 1 - 1) {
        return 4;
    }

    a = 1;
    if(a || b){
        return 0;
    }

    return 5;
}