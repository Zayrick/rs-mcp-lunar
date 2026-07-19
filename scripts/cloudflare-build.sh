#!/bin/sh
set -eu

# Cloudflare Workers Builds currently provides Node.js but does not document
# Rust/Cargo in its build image. Bootstrap rustup only when it is unavailable,
# then make the Wasm target explicit for both fresh and cached build images.
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
install_root="${CARGO_INSTALL_ROOT:-$cargo_home}"
toolchain="${RUST_TOOLCHAIN:-1.89.0}"
export CARGO_HOME="$cargo_home"
export PATH="$install_root/bin:$cargo_home/bin:$PATH"

if ! command -v rustup >/dev/null 2>&1; then
    if ! command -v curl >/dev/null 2>&1; then
        echo "curl is required to install the Rust toolchain" >&2
        exit 1
    fi

    rustup_init="${TMPDIR:-/tmp}/rs-mcp-lunar-rustup-init.sh"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o "$rustup_init"
    sh "$rustup_init" -y --profile minimal --default-toolchain "$toolchain"
fi

rustup toolchain install "$toolchain" --profile minimal
rustup target add wasm32-unknown-unknown --toolchain "$toolchain"
export RUSTUP_TOOLCHAIN="$toolchain"

npm run build:worker
