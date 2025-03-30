int x = 1;

int main() {
    int x = 5;

    {
        int x = 0;

        if(x) {
            return 1;
        } else {
            int x = 12;
        }

        if(x) {
            return 2;
        }
    }

    if(x != 5) {
        return 3;
    }

    return 0;
}