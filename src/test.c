// Compute the nth fibonnaci number
int start(int n) {
	if (n < 2) {
		return n;
	}
	int n_minus_1;
	int n_minus_2;
	n = n - 1;
	n_minus_1 = start(n);
	n = n - 2;
	n_minus_2 = start(n);
	return  n_minus_1 + n_minus_2;
}
