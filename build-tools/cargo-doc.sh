#!/bin/bash
set -e

cargo clean
cargo doc --no-deps --all-features --open