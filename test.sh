cargo test
cargo test -F async-std
cargo test -F tokio
cargo test -F discord
cargo test -F discord,async-std
cargo test -F discord,tokio