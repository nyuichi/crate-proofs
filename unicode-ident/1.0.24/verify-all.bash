#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
cd "$script_dir"

export CARGO_NET_OFFLINE=true

cargo creusot prove '*' -- -p unicode-ident-verify --no-default-features
cargo test -p unicode-ident-verify --test verification_compare --no-default-features
