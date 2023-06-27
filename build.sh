# Build Sloth
cargo build 

# Compile standard library
./target/debug/sloth std/stdio.sloth std/stdlib.sloth std/stdmath.sloth "$1"

# Generate binary
clang  output.o std/stdio.c std/stdlib.c std/stdmath.c -o program
