#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <crate-directory>" >&2
  exit 2
fi

repo_root=$(cd "$(dirname "$0")" && pwd)
crate_dir=$(cd "$1" && pwd)

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$repo_root/target}"
cd "$crate_dir"
cargo creusot --simple-triggers=false prove
