#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>

void write_results(char *student_name, int score) {
    int fd = open("testout.txt", 577, 420);

    char* grade_text;

    if(score > 80) {
        grade_text = "A";
    } else if(score > 70) {
        grade_text = "B";
    } else {
        grade_text = "F";
    }

    dprintf(fd, "student name: %s grade: %s", student_name, grade_text);  // Write to the file

    close(fd);  // Close file descriptor
}

int main(int argc, char** argv) {

    char* student_name = argv[1];

    int grade_count = argc-2;
    int grade_sum = 0;

    for(int i=2;i<argc;i = i+1) {
        int grade = atoi(argv[i]);
        grade_sum = grade_sum + grade;
    }

    int avg = grade_sum/grade_count;

    write_results(student_name, avg);

    return 0;
}