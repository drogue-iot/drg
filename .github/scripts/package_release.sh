#!/bin/bash

OS=`uname -s`
ARCH=`uname -m`

mv target/release/drg drg
tar -czf drg-"$OS"_"$ARCH".tar.gz README.md LICENSE drg