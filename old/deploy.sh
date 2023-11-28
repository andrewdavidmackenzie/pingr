#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly TARGET_HOST=andrew@pi
readonly TARGET_PATH=/home/andrew/bin/pingr
readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
readonly SOURCE_PATH=./target/${TARGET_ARCH}/release/pingr

PKG_CONFIG_SYSROOT_DIR=./sysroot cargo build --target=${TARGET_ARCH}

rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
ssh -t ${TARGET_HOST} ${TARGET_PATH}
