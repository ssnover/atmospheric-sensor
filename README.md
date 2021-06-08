# atmospheric-sensor
This is a small microcontroller application for reading data on CO2 levels and reporting it over a UART-serial bridge.

## Building
Building a cargo workspace with multiple conflicting configurations doesn't really work right now. In order to build the binaries you need to change your current working directory to the binary crate's and then call `cargo build` (or any other cargo command). This allows `rust-lld` to discover the correct linker scripts for each board.

