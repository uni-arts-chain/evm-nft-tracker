#!/bin/bash
set -e

if [ -z "$1" ]; then
 	exec ethereum-nft-tracker $1
else 
	exec "$@"
fi


