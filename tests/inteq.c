//this code previously didn't work, so it is now a test
int char_to_int(char x){
    return x - 48;
}

int str_to_int(char* str, int len) {
    if (len == 3) {
        return char_to_int(str[0])*100 + char_to_int(str[1])*10 + char_to_int(str[2]);
    } else if (len == 2) {
        return char_to_int(str[0])*10 + char_to_int(str[1]);
    } else {
        return char_to_int(str[0]);
    }
}

int main() {
    return char_to_int(49);
}