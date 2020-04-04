#!/usr/bin/env bash

# Set an error handler
trap on_exit EXIT

# printing the simple stack trace
on_exit() {
    while caller $((n++));
    do :;
    done;
}

PROJECT_NAME=waclient
TARGET=x86_64-unknown-linux-musl
BUILD_MODE=release
FILE=target/${TARGET}/${BUILD_MODE}/${PROJECT_NAME}
setup() {
    mkdir -p ./build
    [[ -z "$1" ]] && PROJECT_NAME="waclient" || PROJECT_NAME="$1"
    FILE=target/${TARGET}/${BUILD_MODE}/${PROJECT_NAME}
}

build() {
    echo "Start building ${PROJECT_NAME} in release mode.."
    RUSTC_WRAPPER= cross build --package ${PROJECT_NAME} --release --target=${TARGET}
    binary_stats
    # strip_binary we don't have to!
    # binary_stats
    copy_files
    echo "Done"
}

copy_files() {
    echo "Copying the file to build"
    cp ${FILE} ./build
}

strip_binary() {
    if [[ "$BUILD_MODE" = "debug" ]]; then
        return
    fi
    echo "Striping Binary file..."
    strip ${FILE}
}

binary_stats() {
    echo "Binary Stats:"
    ls -lh ${FILE}
}

# not used anymore
compress_binary() {
    echo "Compressing the binary file using upx..."
    upx -9k ${FILE}
}

setup $1
build $1
