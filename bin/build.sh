#!/bin/sh
# The build runs without the user's shell PATH; look where cargo usually lives.
PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
exec cargo build --release --locked
