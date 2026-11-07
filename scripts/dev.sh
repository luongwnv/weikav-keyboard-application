#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
# Prefer rustup when present; do not modify the user's global shell configuration.
if [ -x "$HOME/.cargo/bin/cargo" ]; then
  PATH="$HOME/.cargo/bin:$PATH"
  export PATH
fi
exec cargo run --locked -- "$@"
