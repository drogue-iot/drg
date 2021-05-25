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
    mkdir -p drg-"$VERSION"
    cp README.md LICENSE "drg${EXE}" drg-"$VERSION"/
    tar -czf drg-"$VERSION"-"$SUFFIX".tar.gz drg-"$VERSION"
    ;;
  "zip")
    mkdir -p drg-"$VERSION"
    cp README.md LICENSE "drg${EXE}" drg-"$VERSION"/
    7z a -tzip drg-"$VERSION"-"$SUFFIX".zip drg-"$VERSION"
    ;;
esac
