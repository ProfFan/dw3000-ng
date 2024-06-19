//! High-level interface to the DW3000
//!
//! The entry point to this API is the [DW3000] struct. Please refer to the
//! documentation there for more details.
//!
//! This module implements a high-level interface to the DW3000. This is the
//! recommended way to access the DW3000 using this crate, unless you need the
//! greater flexibility provided by the [register-level interface].
//!
//! [register-level interface]: ../ll/index.html

use core::{fmt, num::Wrapping};

#[allow(unused_imports)]
pub use awake::*;
pub use error::*;
pub use ready::*;
#[allow(unused_imports)]
pub use receiving::*;
#[allow(unused_imports)]
pub use sending::*;
#[allow(unused_imports)]
pub use sleeping::*;
pub use state_impls::*;
#[allow(unused_imports)]
pub use uninitialized::*;
#[allow(unused_imports)]
pub use carrier_freq_offset::*;

use crate::ll;

mod awake;
mod error;
mod ready;
mod receiving;
mod sending;
mod sleeping;
mod state_impls;
mod uninitialized;
mod carrier_freq_offset;

/// Entry point to the DW3000 driver API
#[derive(Copy, Clone)]
pub struct DW3000<SPI, State> {
    ll: ll::DW3000<SPI>,
    seq: Wrapping<u8>,
    state: State,
}

// Can't be derived without putting requirements on `SPI` and `CS`.
impl<SPI, State> fmt::Debug for DW3000<SPI, State>
where
    State: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DW3000 {{ state: ")?;
        self.state.fmt(f)?;
        write!(f, ", .. }}")?;

        Ok(())
    }
}
