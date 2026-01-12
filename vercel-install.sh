#!/bin/bash
# Install Rust
curl https://sh.rustup.rs -sSf | sh -s -- -y
source $HOME/.cargo/env

# Add WASM target
rustup target add wasm32-unknown-unknown

# Download Trunk binary (faster than cargo install)
wget -qO- https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf-
mv trunk ./trunk_bin

# Install Node dependencies (Tailwind, etc.)
npm install