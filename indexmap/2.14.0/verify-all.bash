#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$script_dir/../.." && pwd)
cd "$script_dir"

export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo_root/target}"

cargo creusot clean --force
cargo creusot --simple-triggers=false prove -- --no-default-features
cargo creusot clean --force
cargo creusot --simple-triggers=false prove
cargo creusot clean --force
cargo creusot --simple-triggers=false prove -- --all-features
