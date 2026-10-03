#!/bin/sh

TARGET=${1}
DESTNAME=$(echo "$TARGET" | sed -E '
    s/^x86_64-.*-windows.*/rodioplay.x86_64.exe/; t
    s/^x86_64-.*/rodioplay.x86_64/;         t
    s/^i686-.*-windows.*/rodioplay.i686.exe/; t
    s/^i686-.*/rodioplay.i686/;             t
    s/^aarch64-.*-windows.*/rodioplay.aarch64.exe/; t
')

MOVE_RESULT=1
if [ -z "$DESTNAME" ]; then
    MOVE_RESULT=0
fi

mkdir -p ./target/$TARGET
cargo build --release --target=${TARGET} || exit 1

SRCDIR="./target/$TARGET/release"
if echo "$TARGET" | grep -q "windows"; then
    SRCFILE="$SRCDIR/rodioplay.exe"
else
    SRCFILE="$SRCDIR/rodioplay"
fi

mkdir -p ./build
cp "${SRCFILE}" "./build/${DESTNAME}"

