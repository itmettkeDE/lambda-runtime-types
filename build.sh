#!/usr/bin/env bash
set -e

BIN="${1}"
shift
if [ -z "${BIN}" ]; then
    echo "Requires the binary to build as first paramter"
    exit 1
fi

cross build --example "${BIN}" --release --target x86_64-unknown-linux-musl "${@}"
strip "./target/x86_64-unknown-linux-musl/release/examples/${BIN}"
cp "./target/x86_64-unknown-linux-musl/release/examples/${BIN}" ./target/x86_64-unknown-linux-musl/release/examples/bootstrap 
zip -r9 -j "./${BIN}.zip" ./target/x86_64-unknown-linux-musl/release/examples/bootstrap
