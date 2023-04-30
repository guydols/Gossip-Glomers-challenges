# Gossip Glomers challenge solution in Rust

A series of distributed systems challenges by Fly.io and Kyle Kingsbury, author of Jepsen.
I've had a lot of fun implementing and thinking about how to tackle these challenges.
If this is something you find interesting, I urge you to check out https://fly.io/dist-sys/

So far i've completed challenge 1, 2 and all the challenges under 3.

## How to use the solution

This binary is designed around the challenges and runs under the maelstrom.jar. 

Build the tidal binary and run under maelstrom with a workload:

	cargo build --release

	java -jar maelstrom.jar test -w echo --bin ./target/release/tidal --node-count 8 --time-limit 30

## Dependencies

This project depends on the tokio, async and serde, which are included as dependencies in the Cargo.toml file.

## License

This project is licensed under the GNU GPL - see the LICENSE file for details.

