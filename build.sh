# Build Sloth
cargo build 

# Compile standard library
./target/debug/sloth std/stdio.sloth
mv output.o stdio.o
./target/debug/sloth std/stdlib.sloth
mv output.o stdlib.o
./target/debug/sloth std/stdmath.sloth
mv output.o stdmath.o

# Compile user program
./target/debug/sloth "$1"
mv output.o main.o

# Generate binary
clang stdio.o std/stdio.c stdlib.o std/stdlib.c stdmath.o std/stdmath.c main.o -o program
