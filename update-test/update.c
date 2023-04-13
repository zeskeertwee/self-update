#include <stdio.h>

int main(int argc, char *argv[]) {
    printf("Hello from the update, args:\n");
    int i;
    for(i=0;i<argc;i++)
    {
        printf("%s\n",argv[i]);
    }
    return 0;
}