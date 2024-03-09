// Compute the nth fibonnaci number
int fibb(int n) {
	if (n < 2) {
		return n;
	}
	int n_minus_1;
	int n_minus_2;
	n = n - 1;
	n_minus_1 = fibb(n);
	n = n - 1;
	n_minus_2 = fibb(n);
	return  n_minus_1 + n_minus_2;
}
int start(int a) {
	return fibb(10);
}
