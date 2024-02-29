a.out: main.o asm.o
	gcc main.o asm.o

main.o: main.c
	gcc -c main.c

asm.o: out.asm
	as out.asm -o asm.o

out.asm: src/* Cargo.toml
	cargo run

clean:
	rm out.asm *.o a.out temp.c gcc.out

gcc:
	@echo "#include <stdio.h>" > temp.c
	@echo "int test_func() {" >> temp.c
	@cat src/test.c >> temp.c
	@echo "}" >> temp.c
	@echo 'int main() { printf("%d\n", test_func()); }' >> temp.c
	gcc temp.c -o gcc.out

test: gcc a.out
	@echo "GCC output:"
	@./gcc.out
	@echo "My output:"
	@./a.out
