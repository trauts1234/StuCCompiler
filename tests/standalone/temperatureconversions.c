int main() {
    int celsius_temp = 20;

    int fahrenheit_times_ten = celsius_temp * 18 + 320;

    if(fahrenheit_times_ten){

        int fahrenheit_temp = fahrenheit_times_ten / 10;

        return fahrenheit_temp;

    } else {
        return 0;
    }
}