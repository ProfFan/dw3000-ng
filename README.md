# Rust DW3000 Driver [![crates.io](https://img.shields.io/crates/v/dw3000-ng.svg)](https://crates.io/crates/dw3000-ng) [![Documentation](https://docs.rs/dw3000-ng/badge.svg)](https://docs.rs/dw3000-ng)
## Introduction

A modernized driver for the Decawave [DW3000] UWB transceiver, written in the [Rust] programming language. We used the crate dw1000 developped for the [DW1000] module and changed the registers access and spi functions, added fast command and implemented some high level functions.

[DW3000]: https://www.decawave.com/product/decawave-dw3000-ic/
[Rust]: https://www.rust-lang.org/
[DW1000]: https://crates.io/crates/dw1000


## Status

Both RTT methods (single and double sided) are working and giving good positioning values.
No implementation of PDoA or AoA.

Compared to the old dw3000 crate we fixed the GPIOs and LEDs, also got rid of the old unmaintained ieee802154 crate and replaced it with smoltcp.

Examples are still from the old dw3000 crate and need to be updated. I mainly test on the ESP32 platform.

## Usage

Include this crate in your Cargo project by adding the following to `Cargo.toml`:
```toml
[dependencies]
dw3000-ng = "0.3.1"
```

## Documentation

Please refer to the **[API Reference]**.
Please refer to our github for exemples **[github link]**.

Please also refer to the [DW3000 User Manual] 

[API Reference]: https://docs.rs/dw3000-ng
[DW3000 User Manual]: https://www.decawave.com/wp-content/uploads/2021/05/DW3000-User-Manual-1.pdf#page=110&zoom=100,68,106

## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License][] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.

[Zero Clause BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: LICENSE.md


**Based on [Braun Embedded](https://braun-embedded.com/)** <br />
**Modified by SII** <br />
