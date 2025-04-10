int main() {
    if(0 && 0) {
        return 1;
    }

    _Bool a=0, b=0;

    if(a && b) {
        return 2;
    }

    if((a && b) == 1) {
        return 3;
    }

    if(1 && 1 - 1) {
        return 4;
    }

    a = 1;
    if(a && b){
        return 5;
    }

    a = 0;b=1;
    if(a&& b) {
        return 6;
    }

    if(2 && 4) {
        a = 1;

        if(a && b) {
            return 0;
        }
    } else {
        return 8;
    }

    return 7;
}