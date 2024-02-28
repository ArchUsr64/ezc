a.out: main.o asm.o
	gcc main.o asm.o

main.o:
	gcc -c main.c

asm.o: out.asm
	as out.asm -o asm.o

out.asm: src/*
	cargo run

clean:
	rm out.asm *.o a.out
