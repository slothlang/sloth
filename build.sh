# Build Sloth
cargo build --features=llvm-sys/prefer-dynamic
FILENAME="$1"
# Compile standard library
./target/debug/sloth std/stdio.sloth std/stdlib.sloth std/stdmath.sloth $FILENAME

# Generate binary
clang output.o std/stdio.c std/stdlib.c std/stdmath.c -o "${FILENAME%.sloth}"

# Move file
mv "${FILENAME%.sloth}" .
