#!/usr/bin/env bash

set -eu

# The ~/.claude directory is a persisted named volume (see docker-compose.yml),
# which Docker mounts root-owned. Make it writable by this user so Claude Code
# can store conversation history and credentials there across rebuilds.
sudo chown -R "$(id -u):$(id -g)" "$HOME/.claude" 2>/dev/null || true

# Install poetry and the Python package (with optional extras, e.g. pycairo).
python3 -m pip install --user poetry
python3 -m poetry install --all-extras

# Install the WebAssembly build tools for rust/prefig-wasm:
# the wasm32 compilation target and wasm-pack.
rustup target add wasm32-unknown-unknown
curl -sSfL https://rustwasm.github.io/wasm-pack/installer/init.sh | sh

# Install JS dependencies for the website workspace.
cd website
npm ci
