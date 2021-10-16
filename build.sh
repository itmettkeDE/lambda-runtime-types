#!/usr/bin/env bash
set -e

BIN="${1}"
if [ -z "${BIN}" ]; then
    echo "Requires the binary to build as first paramter"
    exit 1
fi

cross build --features binary --bin "${BIN}" --release --target x86_64-unknown-linux-musl
strip "./target/x86_64-unknown-linux-musl/release/${BIN}"
cp "./target/x86_64-unknown-linux-musl/release/${BIN}" ./target/x86_64-unknown-linux-musl/release/bootstrap 
zip -r9 -j "./${BIN}.zip" ./target/x86_64-unknown-linux-musl/release/bootstrap
