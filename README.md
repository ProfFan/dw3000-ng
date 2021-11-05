<<<<<<< HEAD
# Rust DW3000 Driver [![crates.io](https://img.shields.io/crates/v/dw3000.svg)](https://crates.io/crates/dw3000) [![Documentation](https://docs.rs/dw3000/badge.svg)](https://docs.rs/dw3000)
## Introduction

Driver for the Decawave [DW3000] UWB transceiver, written in the [Rust] programming language. We used the crate dw1000 developped for the [DW1000] module and changed the registers access and spi functions, added fast command and implemented some high level functions.

[DW3000]: https://www.decawave.com/product/decawave-dw3000-ic/
[Rust]: https://www.rust-lang.org/
[DW1000]: https://crates.io/crates/dw1000


## Status

We tried a first positionning exemple using RTT methode. Lot of work still need to be added like the use of PDoA or AoA.

These examples uses a NUCLEO STM32F103RB

This driver is built on top of embedded-hal, which means it is portable and can be used on any platform that implements the embedded-hal API.


## Usage

Include this crate in your Cargo project by adding the following to `Cargo.toml`:
```toml
[dependencies]
dw3000 = "0.1.1"
```


## Documentation

Please refer to the **[API Reference]**.

Please also refer to the [DW3000 User Manual] 

[API Reference]: https://docs.rs/dw3000
[DW3000 User Manual]: https://www.decawave.com/wp-content/uploads/2021/05/DW3000-User-Manual-1.pdf#page=110&zoom=100,68,106


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License][] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.

[Zero Clause BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: LICENSE.md


**Based on [Braun Embedded](https://braun-embedded.com/)** <br />
**Modified by Clément PENE and Romain SABORET** <br />
=======
ta gueule
>>>>>>> 7ed5f4d6efb79f0875dacbf422317fc6307621f7
