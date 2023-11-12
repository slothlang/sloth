# Build Sloth
#cargo build
FILENAME="$1"
# Compile standard library
./target/release/sloth std/extern.sloth std/stdmath.sloth std/stdio.sloth $FILENAME

# Generate binary
clang -lm output.o std/stdsocket.c std/stdio.c std/stdlib.c std/stdmath.c -o "${FILENAME%.sloth}"

# Move file
mv "${FILENAME%.sloth}" out/
rm output.o
