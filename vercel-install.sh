#!/bin/bash
set -e 

# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 2. Update PATH for the rest of THIS script
. "$HOME/.cargo/env"

# 3. Add WASM target
rustup target add wasm32-unknown-unknown

# 4. Download Trunk binary (using curl instead of wget)
curl -L https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xz

# 5. Move trunk and ensure it is executable
mv trunk ./trunk_bin
chmod +x ./trunk_bin

# 6. Install Node dependencies
npm install