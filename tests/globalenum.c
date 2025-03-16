enum e {x,y};

enum f {A,B} bar = A;

enum e baz = x;

int main(){
    enum e foo = x;

    if(foo != 0){
        return 1;
    }
    foo = y;
    if(foo == x){
        return 2;
    }

    if(bar != A){
        return 3;
    }

    foo = x;bar=A;
    if(foo != bar){
        return 4;
    }

    if (baz != x){
        return 5;
    }

    return 0;
}