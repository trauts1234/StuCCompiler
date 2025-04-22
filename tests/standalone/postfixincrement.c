int main() {
    int x=1;
    int y = x++;
    if(y != 1) {
        return 1;
    }
    if (x != 2) {
        return 2;
    }
    
    y = 5 + x++;
    if(y != 7) {
        return 3;
    }
    if (x != 3){
        return 4;
    }
    return 0;
}