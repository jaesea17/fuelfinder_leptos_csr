#!/bin/bash
set -e 

# 1. Source the correct Rust env based on where it's installed
if [ -f "/rust/env" ]; then
  # Vercel's pre-installed Rust
  . "/rust/env"
elif [ -f "$HOME/.cargo/env" ]; then
  # Locally installed Rust
  . "$HOME/.cargo/env"
else
  # Install Rust if not present
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  . "$HOME/.cargo/env"
fi

# 2. Add WASM target
rustup target add wasm32-unknown-unknown

# 3. Download Trunk binary (using curl instead of wget)
curl -L https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xz

# 4. Move trunk and ensure it is executable
mv trunk ./trunk_bin
chmod +x ./trunk_bin

# 5. Install Node dependencies
npm install