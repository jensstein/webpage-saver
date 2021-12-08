#!/usr/bin/env bash

set -xe

cd /src
cargo build --release --target aarch64-unknown-linux-gnu
