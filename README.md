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
int fibb_iter(int n) {
	int i, first, second;
	i = 1;
	first = 0;
	second = 1;
	while (1) {
		if (i >= n) {
			break;
		}
		second = first + second;
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
	return n_minus_1 + n_minus_2;
}
int start()
{
	int n = 10, iter = fibb_iter(n), recurse = fibb(n);
	return iter == recurse;
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
1
EZC output:
1

```

## Grammar
 ```c
 <Func>
 | int Ident(<Parmeter>*) {<Stmts>*}

 <Parameters>
 | int Ident
 | int Ident, <Parameter>

 <Stmts>
 | if (<Expression>) {<Stmts>*}
 | while (<Expression>) {<Stmts>*}
 | int <Decl>;
 | Ident = <Expression>;
 | break;
 | continue;
 | return <Expression>;

 <Decl>
 | Ident
 | Ident, <Decl>
 | Ident = <Expression>
 | Ident = <Expression>, <Decl>

 <Expression>
 | Ident(<Arguments>)
 | <DirectValue>
 | <DirectValue> <BinaryOperation> <DirectValue>

 <Arguments>
 | <DirectValue>
 | <DirectValue>, <Arguments>

 <DirectValue>
 | Ident
 | Const

 <BinaryOperation>
 | +, -, *, /, %, &, |, ^, <, <=, >, >=, ==, !=
```
