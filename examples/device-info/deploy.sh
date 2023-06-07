#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly PACKAGE=device-info

readonly TARGET_HOST=rb
readonly TARGET_PATH=./bin
readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
readonly SOURCE_PATH=../../target/${TARGET_ARCH}/debug/${PACKAGE}

cargo build --target=${TARGET_ARCH} -F wslcross
scp ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}


