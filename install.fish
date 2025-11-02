#!/usr/bin/env fish

cargo build --release
cp -f target/release/adwbar ~/.local/bin/
