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

pub use awake::*;
pub use error::*;
pub use ready::*;
pub use receiving::*;
pub use sending::*;
pub use sleeping::*;
pub use state_impls::*;
pub use uninitialized::*;

use crate::ll;

mod awake;
mod error;
mod ready;
mod receiving;
mod sending;
mod sleeping;
mod state_impls;
mod uninitialized;

/// Entry point to the DW3000 driver API
#[derive(Copy, Clone)]
pub struct DW3000<SPI, CS, State> {
    ll: ll::DW3000<SPI, CS>,
    seq: Wrapping<u8>,
    state: State,
}

// Can't be derived without putting requirements on `SPI` and `CS`.
impl<SPI, CS, State> fmt::Debug for DW3000<SPI, CS, State>
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
