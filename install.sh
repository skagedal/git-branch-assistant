#!/usr/bin/env bash

set -e

TOOL=git-branch-assistant
BIN=${HOME}/local/bin
BUILT_BINARY=`pwd`/build/install/${TOOL}/bin/${TOOL}

./gradlew install
ln -fs ${BUILT_BINARY} ${BIN}/${TOOL}

