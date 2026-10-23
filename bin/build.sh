#!/bin/sh
# Install is a download: the release for the plugin version holds one binary per platform. Build
# from source only when there is no asset for this machine, or when a local build is up to date.
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
bin="$root/target/release/herdr-clickup"

# A local build newer than every source file is the developer's own; keep it.
if [ -x "$bin" ] && [ -z "$(find "$root/src" "$root/Cargo.toml" "$root/Cargo.lock" -newer "$bin" -print -quit 2>/dev/null)" ]; then
    exit 0
fi

version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$root/herdr-plugin.toml" | head -n 1)
os=$(uname -s)
arch=$(uname -m)
case "${os}/${arch}" in
    Darwin/arm64) target=aarch64-apple-darwin ;;
    Darwin/x86_64) target=x86_64-apple-darwin ;;
    Linux/x86_64) target=x86_64-unknown-linux-gnu ;;
    Linux/aarch64 | Linux/arm64) target=aarch64-unknown-linux-gnu ;;
    *) target= ;;
esac

if [ -n "$target" ] && [ -n "$version" ]; then
    url="https://github.com/ssk090/herdr-clickup/releases/download/v${version}/herdr-clickup-${target}"
    mkdir -p "$root/target/release"
    tmp="${bin}.download"
    if curl -fsSL --retry 3 --retry-delay 2 -o "$tmp" "$url" 2>/dev/null; then
        chmod +x "$tmp"
        mv "$tmp" "$bin"
        echo "herdr-clickup ${version}: downloaded ${target}"
        exit 0
    fi
    rm -f "$tmp"
fi

PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
if ! command -v cargo >/dev/null 2>&1; then
    echo "herdr-clickup: no prebuilt binary for ${os}/${arch} at v${version:-unknown}, and cargo is not installed" >&2
    echo "Install Rust from https://rustup.rs, or use a supported macOS/Linux machine." >&2
    exit 1
fi
echo "herdr-clickup: no prebuilt binary for this platform; building from source"
exec cargo build --release --locked --manifest-path "$root/Cargo.toml"
