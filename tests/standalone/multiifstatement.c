//this code found a bug in the label generator, so I added a regression test for it

void int_to_str(int i, char* out) {
    int upper_digit = i / 10;
    int lower_digit = i % 10;

    if(upper_digit) {
        out[0] = 48 + upper_digit;
    }
    out[1] = 48 + lower_digit;
}

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