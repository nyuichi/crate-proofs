#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
cd "$script_dir"

export CARGO_NET_OFFLINE=true

cargo creusot prove '*' -- --no-default-features
cargo creusot prove '*' -- --no-default-features --features alloc
cargo creusot prove '*' -- --all-features
