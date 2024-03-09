test: gcc.out ezc.out
	@echo "GCC output:"
	-@./gcc.out
	@echo "EZC output:"
	@./ezc.out

ezc.out: main.o ezc.o
	gcc main.o ezc.o -o ezc.out

main.o: main.c
	gcc -c main.c

ezc.o: ezc.asm
	as ezc.asm -o ezc.o

ezc.asm: src/* Cargo.toml src/test.c
	cargo run

clean:
	rm ezc.asm *.o *.out

gcc.out: src/test.c main.o
	gcc -c src/test.c
	gcc main.o test.o -o gcc.out
