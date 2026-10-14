#!/bin/sh
set -e
cd "$(dirname "$0")"
rustc -O tbc.rs -o tbc
mkdir -p "$HOME/.local/bin"
install -m 755 tbc "$HOME/.local/bin/tbc"
"$HOME/.local/bin/tbc"
