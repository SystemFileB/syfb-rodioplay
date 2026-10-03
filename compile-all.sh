#!/bin/sh

TARGETS="
aarch64-pc-windows-gnullvm
i686-pc-windows-gnu
i686-unknown-linux-gnu
x86_64-pc-windows-gnu
x86_64-unknown-linux-gnu
"

for i in $TARGETS; do
    if [ "$i" = "i686-unknown-linux-gnu" ]; then
        PKG_CONFIG_ALLOW_CROSS=1 PKG_CONFIG_PATH=/usr/lib32/pkgconfig:/usr/share/pkgconfig ./compile-cross.sh $i
    else
        ./compile-cross.sh $i
    fi

    if [ $? -ne 0 ]; then
        exit $?
    fi
done
