#!/usr/bin/env bash

TARGET="$1"
REST_ARGS="${@:2}"
TIMEOUT=${TIMEOUT:-5}

timeout --preserve-status --foreground\
    "$TIMEOUT" qemu-system-aarch64 -M raspi3 -nographic -semihosting -kernel "$TARGET" -s "$REST_ARGS"

exit "$?"
