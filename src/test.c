// Compute the 10th fibonnaci number
int first;
int second;
int i;
i = 1;
first = 0;
second = 1;
while (i < 10) {
	int temp;
	temp = second;
	second = first + temp;
	first = temp;
	i = i + 1;
}
return second;
