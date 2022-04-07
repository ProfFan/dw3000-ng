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

#![no_std]
#![no_main]
#![deny(missing_docs)]

pub mod configs;
pub mod fast_command;
pub mod hl;
pub mod ll;
pub mod time;

/// Redirection of nb::block
pub mod block{pub use nb::block;}

#[doc(no_inline)]
pub use ieee802154::mac;

pub use crate::{
	configs::Config,
	fast_command::FastCommand,
	hl::{
		AutoDoubleBufferReceiving,
		Error,
		Message,
		Ready,
		Sending,
		SingleBufferReceiving,
		Sleeping,
		Uninitialized,
		DW3000,
	},
	block::block,
};

use defmt_rtt as _; // global logger

// TODO(5) adjust HAL import
use stm32f1xx_hal as _; // memory layout

use panic_probe as _;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

// defmt-test 0.3.0 has the limitation that this `#[tests]` attribute can only be used
// once within a crate. the module can be in any file but there can only be at most
// one `#[tests]` module in this library crate
#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use defmt::assert;

    #[test]
    fn it_works() {
        assert!(true)
    }
}

