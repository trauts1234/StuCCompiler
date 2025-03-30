int main() {
    int x=1;

    if(1) x=0;
    else {
        x = 2;
    }

    if(x != 0) {
        return 1;
    }

    int y = 1;

    if(0) {
        y = 2;
    } else y = 0;

    if (y != 0){
        return 2;
    }

    int z = 1;

    if(0) z = 2;else z = 0;

    if(z != 0){
        return 3;
    }

    return 0;
}