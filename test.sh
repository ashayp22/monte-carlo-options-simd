export RUSTFLAGS="-C target-cpu=native -C opt-level=3"
cargo build
cargo test -- --nocapture