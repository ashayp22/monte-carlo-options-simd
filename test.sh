set RUST_FLAGS=-C -target_feature=+avx,+fma
cargo build
cargo test -- --nocapture