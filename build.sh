# Build Sloth
cargo build
FILENAME="$1"
# Compile standard library
./target/release/sloth std/stdio.sloth std/stdlib.sloth std/stdmath.sloth $FILENAME

# Generate binary
clang -lm output.o std/stdio.c std/stdlib.c std/stdmath.c -o "${FILENAME%.sloth}"

# Move file
mv "${FILENAME%.sloth}" .
