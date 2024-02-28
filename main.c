#include <stdio.h>

extern int test_func(void);

int main() {
	printf("%d\n", test_func());
}
