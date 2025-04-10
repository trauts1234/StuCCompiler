int main() {
    int x=1;
    int y = --x;

    if(x != 0) {
        return 1;
    }
    if(y != 0) {
        return 2;
    }
    
    return 0;
}