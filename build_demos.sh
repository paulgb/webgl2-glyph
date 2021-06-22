#!/bin/sh

set -e

for DEMO in $(ls -d demos/*/)
do
	cargo build --manifest-path "${DEMO}Cargo.toml"
done
