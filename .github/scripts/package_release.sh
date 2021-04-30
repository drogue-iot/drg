#!/bin/bash

set -xe

SUFFIX="$1"
EXE="$2"
TYPE="$3"
VERSION="$4"

# copy, don't move, as we might need it later
cp "target/release/drg${EXE}" "drg${EXE}"
case "$TYPE" in
  "tar.gz")
    tar -czf drg-"$VERSION"-"$SUFFIX".tar.gz README.md LICENSE "drg${EXE}"
    ;;
  "zip")
    7z a -tzip drg-"$VERSION"-"$SUFFIX".zip README.md LICENSE "drg${EXE}"
    ;;
esac
