//! Driver crate for the DW3000 UWB transceiver
//!
//! The recommended way to use this driver is the [high-level interface]. If you
//! require a higher degree of flexibility, you can use the
//! [register-level interface] instead.
//!
//! We used the crate [`dw1000`] developped for the DW1000 module and changed
//! the registers access and spi functions, added fast command and implemented
//! some high level functions.
//!
//! We tried a first positionning exemple using RTT methode.
//! Lot of work still need to be added like the use of PDoA or AoA.
//!
//! These examples uses a NUCLEO STM32F103RB
//!
//! This driver is built on top of [`embedded-hal`], which means it is portable
//! and can be used on any platform that implements the `embedded-hal` API.
//!
//! [high-level interface]: hl/index.html
//! [register-level interface]: ll/index.html
//! [`dw1000`]: https://crates.io/crates/dw1000
//! [`embedded-hal`]: https://crates.io/crates/embedded-hal
#![cfg_attr(not(any(test, feature = "std")), no_main)]
#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![deny(missing_docs)]

pub mod configs;
pub mod fast_command;
pub mod hl;
pub mod ll;
pub mod time;

/// Redirection of nb::block
pub mod block {
    pub use nb::block;
}

pub use crate::{
    block::block,
    configs::Config,
    fast_command::FastCommand,
    hl::{
        AutoDoubleBufferReceiving, Error, Message, Ready, Sending, SingleBufferReceiving, Sleeping,
        Uninitialized, DW3000,
    },
};
