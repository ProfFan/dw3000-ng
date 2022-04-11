# Rust DW3000 Driver [![crates.io](https://img.shields.io/crates/v/dw3000.svg)](https://crates.io/crates/dw3000) [![Documentation](https://docs.rs/dw3000/badge.svg)](https://docs.rs/dw3000)
## Introduction

Driver for the Decawave [DW3000] UWB transceiver, written in the [Rust] programming language. We used the crate dw1000 developped for the [DW1000] module and changed the registers access and spi functions, added fast command and implemented some high level functions.

[DW3000]: https://www.decawave.com/product/decawave-dw3000-ic/
[Rust]: https://www.rust-lang.org/
[DW1000]: https://crates.io/crates/dw1000


## Status

Both RTT methods (single and double sided) are working and giving good positioning values.
No implementation of PDoA or AoA.

We tested the crate using two different platforms; both platforms examples are available on dedicated repository (raspberry pi and STM32F103RB)
Examples available are basic communication and distance measurement between two modules (single and double sided RTT)

We built the driver on top of embedded-hal, which means it is portable and can be used on any platform that implements the embedded-hal API.


## Usage

Include this crate in your Cargo project by adding the following to `Cargo.toml`:
```toml
[dependencies]
dw3000 = "0.2.0"
```

We also provided workspaces in which you can find some example depending of the target (raspberry pi or stm32f103rb).
We built stm32f103rb examples using the app-template of the knurling project. 
Unfortunately, you cannot build your example directly from the main repository, you need to navigate to the examples folder to build and run applications. 


## Documentation

Please refer to the **[API Reference]**.
Please refer to our github for exemples **[github link]**.

Please also refer to the [DW3000 User Manual] 

[API Reference]: https://docs.rs/dw3000
[DW3000 User Manual]: https://www.decawave.com/wp-content/uploads/2021/05/DW3000-User-Manual-1.pdf#page=110&zoom=100,68,106
[github link]: https://github.com/SII-Public-Research/dw3000

## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License][] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE.md] for full details.

[Zero Clause BSD License]: https://opensource.org/licenses/0BSD
[LICENSE.md]: LICENSE.md


**Based on [Braun Embedded](https://braun-embedded.com/)** <br />
**Modified by SII** <br />
