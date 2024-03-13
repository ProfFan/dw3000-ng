# Rust DW3000 Driver [![crates.io](https://img.shields.io/crates/v/dw3000-ng.svg)](https://crates.io/crates/dw3000-ng) [![Documentation](https://docs.rs/dw3000-ng/badge.svg)](https://docs.rs/dw3000-ng)
## Introduction

A modernized driver for the Decawave [DW3000] UWB transceiver, written in the [Rust] programming language. We used the crate dw1000 developped for the [DW1000] module and changed the registers access and spi functions, added fast command and implemented some high level functions.

[DW3000]: https://www.decawave.com/product/decawave-dw3000-ic/
[Rust]: https://www.rust-lang.org/
[DW1000]: https://crates.io/crates/dw1000


## Status

Both RTT methods (single and double sided) are working and giving good positioning values.
No implementation of PDoA or AoA.

Compared to the old `dw3000` crate we fixed the GPIOs and LEDs, also got rid of the old unmaintained ieee802154 crate and replaced it with smoltcp.

We mainly test on the ESP32 platform with `embassy` async framework.

## Usage

Include this crate in your Cargo project by adding the following to `Cargo.toml`:
```toml
[dependencies]
dw3000-ng = "0.5.1
```

## Documentation

Please refer to the **[API Reference]**.

Please also refer to the [DW3000 User Manual] 

[API Reference]: https://docs.rs/dw3000-ng
[DW3000 User Manual]: https://www.qorvo.com/products/d/da008154

## CHANGELOG

### 0.5.1

- Fix `DTUNE` register value

### 0.5.0

- Migrated to `embedded-hal` 1.0
- Removed explicit manipulations of the SPI CS pin

## License

BSD-3-Clause
