#!/bin/sh
export ALLEGRO_STATICLINK
export ALLEGRO_INCLUDE_DIR=$HOME/dev/allegro/include
export RUST_ALLEGRO_EXAMPLE_LINK_PATH=$HOME/dev/allegro/lib
cargo build --release --target "i686-pc-windows-gnu" 

