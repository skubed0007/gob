rm -rf ./bin
mkdir bin
cargo build --release
cp target/release/gob bin/gob