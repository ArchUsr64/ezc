// Compute the 10th fibonnaci number
int func1() {
	int first;
	int second;
	int i;
	i = 1;
	first = 0;
	second = 1;
	while (i < 10) {
		second = first + second;
		first = second - first;
		i = i + 1;
	}
	return second;
}
// Computes 10!
int func2() {
	int i;
	int res;
	i = 1;
	res = 1;
	while (i < 10) {
		res = i * res;
		i = i + 1;
	}
	return res;
}
int start() {
	int res1;
	int res2;
	res1 = func1();
	res2 = func2();
	return res1 + res2;
}
