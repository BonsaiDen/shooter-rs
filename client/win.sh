#!/bin/sh
export ALLEGRO_STATICLINK
export ALLEGRO_INCLUDE_DIR=$HOME/dev/allegro/include
export RUST_ALLEGRO_EXAMPLE_LINK_PATH=$HOME/dev/allegro/lib
echo "Note: Make sure to run 'cargo clean' before any initial windows build!"
cargo rustc --release --target "i686-pc-windows-gnu" -- -C link-args=-Wl,--subsystem,windows 

