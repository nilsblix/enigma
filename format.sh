set -e

cargo test --quiet
cargo build --quiet
cargo fmt
cd impls
cargo build --quiet
cargo fmt
cd ..

echo "\x1b[32mSuccessful!"
