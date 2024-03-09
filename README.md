# ezc
EZC (pronounced Easy-C) is a tiny subset of C

## Build and Execution
1. Clone the repository:  
`git clone https://github.com/ArchUsr64/ezc`

2. Change to newly created directory:  
`cd ezc`

3. Compile and execute the provided example at [`src/test.c`](https://github.com/ArchUsr64/ezc/blob/main/src/test.c):  
`make test`

**Dependencies:** Cargo, GCC, GNU Assembler, Make 

### Example:
[`src/test.c`](https://github.com/ArchUsr64/ezc/blob/main/src/test.c)
```c
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
```
### Output:
```python
$ make test
gcc -c main.c
gcc -c src/test.c
gcc main.o test.o -o gcc.out
cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/ezc`
as ezc.asm -o ezc.o
gcc main.o ezc.o -o ezc.out
GCC output:
55
EZC output:
55

```

## Grammar
```c
<Func>
| int Ident(int Ident) {<Stmts>*}

<Stmts>
| if (<Expression>) {<Stmts>*}
| while (<Expression>) {<Stmts>*}
| int Ident;
| Ident = <Expression>;
| break;
| continue;
| return <Expression>;

<Expression>
| Ident(<DirectValue>)
| <DirectValue>
| <DirectValue> <BinaryOperation> <DirectValue>

<DirectValue>
| Ident
| Const

<BinaryOperation>
| +, -, *, /, %, &, |, ^, <, <=, >, >=, ==, !=
```
