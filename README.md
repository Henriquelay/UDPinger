# UDPinger

A simple applcation to ping a UDP server and measure some round trip time statistics.

## Configuring

This is a really simple application, all configuration is done in [src/main.rs] file, using consts. By deafault, it will occupy port 30001 and ping port 30000 of localhost.

## How to run

1. Have [Rust](https://www.rust-lang.org/tools/install) and Cargo installed.
2. `cargo run` while inside the project tree.

If you prefer to build it to a executable, you can use `cargo build` and then run the executable in `target/release/udpinger`.

To omit debug message (and achieve higher performance), you can append `--release` flag to both commands.

## License

This work is licensed under [the Unlicense](https://unlicense.org/) license. This means this is public domain. Feel free to do as you wish with it.
This does not cover the [UDP Pinger.pdf] file, or the [UDPPingServer.py], which were provided by the professor of the course, but are included here for convenience, testing and context.
All other files are licensed under the Unlicense.
