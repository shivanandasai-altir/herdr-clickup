#!/bin/sh
# herdr starts plugin commands with its server's environment, so find the binary relative to this script.
root=$(cd "$(dirname "$0")/.." && pwd)
bin="$root/target/release/herdr-clickup"
if [ ! -x "$bin" ]; then
  echo "herdr-clickup is not built; run: cargo build --release --manifest-path $root/Cargo.toml" >&2
  exit 127
fi
exec "$bin" "$@"
