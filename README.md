# Redis clone

A Redis-compatible server in Rust, built section by section in Kiln.

    cargo run --release

Then talk to it with any Redis client, e.g. `redis-cli -p 6379 PING` (in WSL:
`sudo apt install redis-tools`).

`Cargo.toml` includes tokio for async networking, but nothing forces you to use it: threads
and `std::net` work just as well for every section.
