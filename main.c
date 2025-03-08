#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <unistd.h>

void write_results(char *student_name, int score) {
    int fd = open("student_scores.txt", 1025);

    char* grade_text;

    if(score >= 80) {
        grade_text = "A";
    } else if(score >= 70) {
        grade_text = "B";
    } else if (score >= 60) {
        grade_text = "C";
    } else if (score >= 50) {
        grade_text = "D";
    } else {
        grade_text = "F";
    }

    dprintf(fd, "student name: %s grade: %s\n", student_name, grade_text);  // Write to the file

    close(fd);  // Close file descriptor
}

void print_current_results() {
    int fd = open("student_scores.txt", O_RDONLY);

    puts("reading file");

    char buffer[1024];
    for(long bytes_read = 1;bytes_read > 0;) {
        bytes_read = read(fd, buffer, 1024 - 1);
        if (bytes_read > 0) {
            buffer[bytes_read] = 0; // Null-terminate the buffer
            puts(buffer);
        }
    }

    close(fd);
}

int main(int argc, char** argv) {

    for(int i=0;i<10;i = i+1) {
        puts("select:\n1. read student scores\n2. save a student score\n3. quit");
        int option=0;
        scanf("%d", &option);

        if (option == 1) {
            print_current_results();
        } else if (option == 2) {
            puts("enter a student name:");

            char student_name[100];
            scanf("%99s", student_name);

            int first = student_name[0];
            int last = student_name[3];

            puts("enter 3 scores");
            int grade_count = 0;
            int grade_sum = 0;
            for(int j=0;j<3;j = j+1) {
                grade_count = grade_count + 1;
                int next;
                scanf("%d", &next);
                grade_sum = grade_sum + next;
            }
            write_results(student_name, grade_sum/grade_count);
        } else if (option == 3) {
            return 0;
        } else {
            puts("unknown option");
            return 1;
        }
    }

    return 0;
}