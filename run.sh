#!/usr/bin/env bash
# For docker
CMD=$1
BLOCK=$2
`./target/release/$CMD $BLOCK`
