int add(int a, int b) {
	int res;
	res = a + b;
	return res;
}
int fibb_iter(int n) {
	int i, first, second;
	i = 1;
	first = 0;
	second = 1;
	while (1) {
		if (i >= n) {
			break;
		}
		second = add(first, second);
		first = second - first;
		i = i + 1;
	}
	return second;
}
int fibb(int n)
{
	if (n < 2) {
		return n;
	}
	int n_minus_1, n_minus_2;
	n = n - 1;
	n_minus_1 = fibb(n);
	n = n - 1;
	n_minus_2 = fibb(n);
	return add(n_minus_1, n_minus_2);
}
int start()
{
	int n = 10, iter = fibb_iter(n), recurse = fibb(n);
	return iter == recurse;
}
