#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>

int main(void) {
    if (access("example_dir/1-never-gonna-give-you-up", R_OK | W_OK)) {
        perror("access");
        return EXIT_FAILURE;
    }
    if (!access("example_dir/1-never-gonna-give-you-up", X_OK)) {
        puts("Accessing a file with X_OK worked even though it... probably... shouldn't?");
        puts("Please run `chmod 644 example_dir/*` and try again.");
        return EXIT_FAILURE;
    }
}
