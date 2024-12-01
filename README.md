# Rust DW3000 Driver [![crates.io](https://img.shields.io/crates/v/dw3000-ng.svg)](https://crates.io/crates/dw3000-ng) [![Documentation](https://docs.rs/dw3000-ng/badge.svg)](https://docs.rs/dw3000-ng)
## Introduction

A modernized driver for the Decawave [DW3000] UWB transceiver, written in the [Rust] programming language. We used the crate dw1000 developped for the [DW1000] module and changed the registers access and spi functions, added fast command and implemented some high level functions.

[DW3000]: https://www.decawave.com/product/decawave-dw3000-ic/
[Rust]: https://www.rust-lang.org/
[DW1000]: https://crates.io/crates/dw1000


## Status

Both RTT methods (single and double sided) are working and giving good positioning values.
PDoA and TDoA can be enabled optionally (Please read the docs as they require certain configurations!).

Compared to the old `dw3000` crate we fixed the GPIOs and LEDs, also got rid of the old unmaintained ieee802154 crate and replaced it with `smoltcp`.

We mainly test on the ESP32 platform with `embassy` async framework.

## Usage

Include this crate in your Cargo project by adding the following to `Cargo.toml`:
```toml
[dependencies]
dw3000-ng = "1.0"
```

## Documentation

Please refer to the **[API Reference]**.

Please also refer to the [DW3000 User Manual]

[API Reference]: https://docs.rs/dw3000-ng
[DW3000 User Manual]: https://www.qorvo.com/products/d/da008154

## Citation

If you are using this in your academic work, please cite it as follows:

```bibtex
@inproceedings{Jiang24hotmobile,
    author = {Jiang, Fan and Dhekne, Ashutosh},
    title = {Demo: uFiμ: An open-source integrated UWB-WiFi-IMU platform for localization research and beyond},
    year = {2024},
    isbn = {9798400704970},
    publisher = {Association for Computing Machinery},
    address = {New York, NY, USA},
    url = {https://doi.org/10.1145/3638550.3643628},
    doi = {10.1145/3638550.3643628},
    booktitle = {Proceedings of the 25th International Workshop on Mobile Computing Systems and Applications},
    pages = {156},
    location = {San Diego, CA, USA},
    series = {HOTMOBILE '24}
}
```

## CHANGELOG

### Current `main`

### 1.0.1

- Elided the `RegAccessor` lifetime
- Fixed the wrong trait import when `async` is enabled/disabled

### 1.0.0

- Added `init_tracing` example to inspect the SPI transactions happening during the initialization of the DW3000
- Modified `rx_wait` to not use `_unchecked` and return `Err` when the decoding of the 802.15.4 frame fails
- **BREAKING**: The library is now both `sync` and `async` compatible. The feature `async` can be used to enable the corresponding interfaces.
  - When `async`, the SPI traits are using `embedded_hal_async`, otherwise `embedded_hal`.
- **BREAKING**: The delay primitives in the `config` function are now `embedded_hal/embedded_hal_async::delay::DelayNs` instead of `FnMut(u32) -> Future<Output = ()>`

### 0.9.0

- API change: `config` now takes a `delay_us(u32)` function to allow non-blocking initialization.
- Now the radio init is feature-par with the official DW3000 driver. You should have much better TX/RX performance now.
- Fixed the register definition of `RX_FWTO`. RX timeout is now working.

### 0.8.4

- Fixed important bug in 0.8.3 where we are trying to read the device ID before the device is ready

### 0.8.3

- Fixed infinite loop in `init()` if the SPI device is not ready or connected

### 0.8.2

- Add parameter `recv_time` to allow delayed receiving by @trembel
- Fix `pll_cc` register by @JohannesProgrammiert

### 0.8.1

- Fixed the `STS` register setup when calling `config()` with STS enabled

### 0.8.0

- Renamed the `num-traits` feature to `rssi` to better indicate what it does
- Added PDoA and TDoA support

### 0.7.0

- Add field rx_quality to struct Message holding first path signal power by (@elrafoon)
- Fixed STS config values by (@elrafoon)

### 0.6.1

- Fixed read of the `RX_RAWST` register

### 0.6.0

- Added the carrier recovery integrator register

### 0.5.1

- Fix `DTUNE` register value

### 0.5.0

- Migrated to `embedded-hal` 1.0
- Removed explicit manipulations of the SPI CS pin

## License

BSD-3-Clause
