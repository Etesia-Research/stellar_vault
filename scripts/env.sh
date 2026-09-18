#!/bin/sh
# Source from the repository root when using the optional local tool installation.
export RUSTUP_HOME="$PWD/.tooling/rustup"
export CARGO_HOME="$PWD/.tooling/cargo"
export PATH="$PWD/.tooling/bin:$CARGO_HOME/bin:$PATH"
