#!/bin/bash

cargo fmt --all -- --check
cargo clippy --all-features -- -D warnings
cargo audit
