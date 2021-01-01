#!/usr/bin/env bash

TARGET="$1"
TIMEOUT=${TIMEOUT:-5}

timeout --preserve-status --foreground\
    "$TIMEOUT" qemu-system-aarch64 -M raspi3 -nographic -semihosting -kernel "$TARGET" -s

exit "$?"
