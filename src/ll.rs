//! Low-level interface to the DW3000
//!
//! This module implements a register-level interface to the DW3000. Users of
//! this library should typically not need to use this. Please consider using
//! the [high-level interface] instead.
//!
//! If you're using the low-level interface because the high-level interface
//! doesn't cover your use case, please consider [filing an issue].
//!
//! **NOTE**: Many field access methods accept types that have a larger number
//! of bits than the field actually consists of. If you use such a method to
//! pass a value that is too large to be written to the field, it will be
//! silently truncated.
//!
//! [high-level interface]: ../hl/index.html
//! [filing an issue]: https://github.com/braun-robotics/rust-dw3000/issues/new

use core::{fmt, marker::PhantomData};

use embedded_hal::{blocking::spi, digital::v2::OutputPin};

#[cfg(feature = "defmt")]
use defmt::Format;

/// Entry point to the DW3000 driver's low-level API
///
/// Please consider using [hl::DW3000] instead.
///
/// [hl::DW3000]: ../hl/struct.DW3000.html
#[derive(Copy, Clone)]
pub struct DW3000<SPI, CS> {
    spi: SPI,
    chip_select: CS,
}

impl<SPI, CS> DW3000<SPI, CS> {
    /// Create a new instance of `DW3000`
    ///
    /// Requires the SPI peripheral and the chip select pin that are connected
    /// to the DW3000.
    pub fn new(spi: SPI, chip_select: CS) -> Self {
        DW3000 { spi, chip_select }
    }

    /// commentaire
    pub fn fast_command(&mut self, fast: u8) -> Result<(), Error<SPI, CS>>
    where
        SPI: spi::Transfer<u8> + spi::Write<u8>,
        CS: OutputPin,
    {
        let mut buffer = [0];
        buffer[0] = (0x1 << 7) | ((fast << 1) & 0x3e) | 0x1;

        self.chip_select.set_low().map_err(Error::ChipSelect)?;
        <SPI as spi::Write<u8>>::write(&mut self.spi, &buffer).map_err(Error::Write)?;
        self.chip_select.set_high().map_err(Error::ChipSelect)?;

        Ok(())
    }

    /// Allow access to the SPI bus
    pub fn bus(&mut self) -> &mut SPI {
        &mut self.spi
    }

    /// Allow access to the chip select pin
    pub fn chip_select(&mut self) -> &mut CS {
        &mut self.chip_select
    }
}

/// Provides access to a register
///
/// You can get an instance for a given register using one of the methods on
/// [`DW3000`].
pub struct RegAccessor<'s, R, SPI, CS>(&'s mut DW3000<SPI, CS>, PhantomData<R>);

impl<'s, R, SPI, CS> RegAccessor<'s, R, SPI, CS>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
{
    /// Read from the register
    #[inline]
    pub fn read(&mut self) -> Result<R::Read, Error<SPI, CS>>
    where
        R: Register + Readable,
    {
        let mut r = R::read();
        let buffer = R::buffer(&mut r);

        init_header::<R>(false, buffer);
        self.0.chip_select.set_low().map_err(Error::ChipSelect)?;
        self.0.spi.transfer(buffer).map_err(Error::Transfer)?;
        self.0.chip_select.set_high().map_err(Error::ChipSelect)?;

        Ok(r)
    }

    /// Write to the register
    #[inline]
    pub fn write<F>(&mut self, f: F) -> Result<(), Error<SPI, CS>>
    where
        R: Register + Writable,
        F: FnOnce(&mut R::Write) -> &mut R::Write,
    {
        let mut w = R::write();
        f(&mut w);

        let buffer = R::buffer(&mut w);
        init_header::<R>(true, buffer);

        self.0.chip_select.set_low().map_err(Error::ChipSelect)?;
        <SPI as spi::Write<u8>>::write(&mut self.0.spi, buffer).map_err(Error::Write)?;
        self.0.chip_select.set_high().map_err(Error::ChipSelect)?;

        Ok(())
    }

    /// Modify the register
    #[inline]
    pub fn modify<F>(&mut self, f: F) -> Result<(), Error<SPI, CS>>
    where
        R: Register + Readable + Writable,
        F: for<'r> FnOnce(&mut R::Read, &'r mut R::Write) -> &'r mut R::Write,
    {
        let mut r = self.read()?;
        let mut w = R::write();

        <R as Writable>::buffer(&mut w).copy_from_slice(<R as Readable>::buffer(&mut r));

        f(&mut r, &mut w);

        let buffer = <R as Writable>::buffer(&mut w);
        init_header::<R>(true, buffer);

        self.0.chip_select.set_low().map_err(Error::ChipSelect)?;
        <SPI as spi::Write<u8>>::write(&mut self.0.spi, buffer).map_err(Error::Write)?;
        self.0.chip_select.set_high().map_err(Error::ChipSelect)?;

        Ok(())
    }
}

/// An SPI error that can occur when communicating with the DW3000
pub enum Error<SPI, CS>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
{
    /// SPI error occured during a transfer transaction
    Transfer(<SPI as spi::Transfer<u8>>::Error),

    /// SPI error occured during a write transaction
    Write(<SPI as spi::Write<u8>>::Error),

    /// Error occured while changing chip select signal
    ChipSelect(<CS as OutputPin>::Error),
}

// We can't derive this implementation, as the compiler will complain that the
// associated error type doesn't implement `Debug`.
impl<SPI, CS> fmt::Debug for Error<SPI, CS>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    <SPI as spi::Transfer<u8>>::Error: fmt::Debug,
    <SPI as spi::Write<u8>>::Error: fmt::Debug,
    CS: OutputPin,
    <CS as OutputPin>::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Transfer(error) => write!(f, "Transfer({:?})", error),
            Error::Write(error) => write!(f, "Write({:?})", error),
            Error::ChipSelect(error) => write!(f, "ChipSelect({:?})", error),
        }
    }
}

#[cfg(feature = "defmt")]
impl<SPI, CS> defmt::Format for Error<SPI, CS>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
{
    fn format(&self, f: defmt::Formatter) {
        match self {
            Error::Transfer(_) => defmt::write!(f, "Transfer()"),
            Error::Write(_) => defmt::write!(f, "Write()"),
            Error::ChipSelect(_) => defmt::write!(f, "ChipSelect()"),
        }
    }
}

/// Initializes the SPI message header
///
/// Initializes the SPI message header for accessing a given register, writing
/// the header directly into the provided buffer. Returns the length of the
/// header that was written.
///
/// TODO: Here we always use the full address, but we should also support the
/// short address mode and masked write mode.
#[inline(always)]
fn init_header<R: Register>(write: bool, buffer: &mut [u8]) -> usize {
    // bool write defines if we are in read or write mode (first bit)
    // sub_id is a bool that defines if we are in full or short command
    // we start with full address!
    buffer[0] = (((write as u8) << 7) & 0x80)
        | ((1u8 << 6) & 0x40) // We always use 2-octet addressing
        | ((R::ID << 1) & 0x3e) // 5-bit base address
        | ((R::SUB_ID >> 6) & 0x01); // MSB of the 7-bit sub-address

    buffer[1] = R::SUB_ID << 2; // last two bits M1 M0 are always 0

    2
}

/// Implemented for all registers
///
/// This is a mostly internal crate that should not be implemented or used
/// directly by users of this crate. It is exposed through the public API
/// though, so it can't be made private.
///
/// The DW3000 user manual, section 7.1, specifies what the values of the
/// constant should be for each register.
pub trait Register {
    /// The register index
    const ID: u8;

    /// The registers's sub-index
    const SUB_ID: u8;

    /// The lenght of the register
    const LEN: usize;
}

/// Marker trait for registers that can be read from
///
/// This is a mostly internal crate that should not be implemented or used
/// directly by users of this crate. It is exposed through the public API
/// though, so it can't be made private.
pub trait Readable {
    /// The type that is used to read from the register
    type Read;

    /// Return the read type for this register
    fn read() -> Self::Read;

    /// Return the read type's internal buffer
    fn buffer(r: &mut Self::Read) -> &mut [u8];
}

/// Marker trait for registers that can be written to
///
/// This is a mostly internal crate that should not be implemented or used
/// directly by users of this crate. It is exposed through the public API
/// though, so it can't be made private.
pub trait Writable {
    /// The type that is used to write to the register
    type Write;

    /// Return the write type for this register
    fn write() -> Self::Write;

    /// Return the write type's internal buffer
    fn buffer(w: &mut Self::Write) -> &mut [u8];
}

/// Generates register implementations
macro_rules! impl_register {
    (
        $(
            $id:expr,
            $sub_id:expr,
            $len:expr,
            $rw:tt,
            $name:ident($name_lower:ident) {
            #[$doc:meta]
            $(
                $field:ident,
                $first_bit:expr,
                $last_bit:expr,
                $ty:ty;
                #[$field_doc:meta]
            )*
            }
        )*
    ) => {
        $(
            #[$doc]
            #[allow(non_camel_case_types)]
            pub struct $name;

            impl Register for $name {
                const ID:     u8    = $id;
                const SUB_ID: u8   = $sub_id;
                const LEN:    usize = $len;
            }

            impl $name {
                // You know what would be neat? Using `if` in constant
                // expressions! But that's not possible, so we're left with the
                // following hack.
                // const SUB_INDEX_IS_NONZERO: usize =
                    // (Self::SUB_ID > 0) as usize;
                // const SUB_INDEX_NEEDS_SECOND_BYTE: usize =
                    // (Self::SUB_ID > 127) as usize;
                const HEADER_LEN: usize = 2;
                    // 1
                    // + Self::SUB_INDEX_IS_NONZERO
                    // + Self::SUB_INDEX_NEEDS_SECOND_BYTE;
            }

            #[$doc]
            pub mod $name_lower {
                use core::fmt;


                const HEADER_LEN: usize = super::$name::HEADER_LEN;


                /// Used to read from the register
                pub struct R(pub(crate) [u8; HEADER_LEN + $len]);

                impl R {
                    $(
                        #[$field_doc]
                        #[inline(always)]
                        pub fn $field(&self) -> $ty {
                            use core::mem::size_of;
                            use crate::ll::FromBytes;

                            // The index (in the register data) of the first
                            // byte that contains a part of this field.
                            const START: usize = $first_bit / 8;

                            // The index (in the register data) of the byte
                            // after the last byte that contains a part of this
                            // field.
                            const END: usize = $last_bit  / 8 + 1;

                            // The number of bytes in the register data that
                            // contain part of this field.
                            const LEN: usize = END - START;

                            // Get all bytes that contain our field. The field
                            // might fill out these bytes completely, or only
                            // some bits in them.
                            let mut bytes = [0; LEN];
                            bytes[..LEN].copy_from_slice(
                                &self.0[START+HEADER_LEN .. END+HEADER_LEN]
                            );

                            // Before we can convert the field into a number and
                            // return it, we need to shift it, to make sure
                            // there are no other bits to the right of it. Let's
                            // start by determining the offset of the field
                            // within a byte.
                            const OFFSET_IN_BYTE: usize = $first_bit % 8;

                            if OFFSET_IN_BYTE > 0 {
                                // Shift the first byte. We always have at least
                                // one byte here, so this always works.
                                bytes[0] >>= OFFSET_IN_BYTE;

                                // If there are more bytes, let's shift those
                                // too.
                                // We need to allow exceeding bitshifts in this
                                // loop, as we run into that if `OFFSET_IN_BYTE`
                                // equals `0`. Please note that we never
                                // actually encounter that at runtime, due to
                                // the if condition above.
                                let mut i = 1;
                                #[allow(arithmetic_overflow)]
                                while i < LEN {
                                    bytes[i - 1] |=
                                        bytes[i] << 8 - OFFSET_IN_BYTE;
                                    bytes[i] >>= OFFSET_IN_BYTE;
                                    i += 1;
                                }
                            }

                            // If the field didn't completely fill out its last
                            // byte, we might have bits from unrelated fields
                            // there. Let's erase those before doing the final
                            // conversion into the field's data type.
                            const SIZE_IN_BITS: usize =
                                $last_bit - $first_bit + 1;
                            const BITS_ABOVE_FIELD: usize =
                                8 - (SIZE_IN_BITS % 8);
                            const SIZE_IN_BYTES: usize =
                                (SIZE_IN_BITS - 1) / 8 + 1;
                            const LAST_INDEX: usize =
                                SIZE_IN_BYTES - 1;
                            if BITS_ABOVE_FIELD < 8 {
                                // Need to allow exceeding bitshifts to make the
                                // compiler happy. They're never actually
                                // encountered at runtime, due to the if
                                // condition.
                                #[allow(arithmetic_overflow)]
                                {
                                    bytes[LAST_INDEX] <<= BITS_ABOVE_FIELD;
                                    bytes[LAST_INDEX] >>= BITS_ABOVE_FIELD;
                                }
                            }

                            // Now all that's left is to convert the bytes into
                            // the field's type. Please note that methods for
                            // converting numbers to/from bytes are coming to
                            // stable Rust, so we might be able to remove our
                            // custom infrastructure here. Tracking issue:
                            // https://github.com/rust-lang/rust/issues/52963
                            let bytes = if bytes.len() > size_of::<$ty>() {
                                &bytes[..size_of::<$ty>()]
                            }
                            else {
                                &bytes
                            };
                            <$ty as FromBytes>::from_bytes(bytes)
                        }
                    )*
                }

                impl fmt::Debug for R {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "0x")?;
                        for i in (0 .. $len).rev() {
                            write!(f, "{:02x}", self.0[HEADER_LEN + i])?;
                        }

                        Ok(())
                    }
                }

                #[cfg(feature = "defmt")]
                impl defmt::Format for R {
                    fn format(&self, f: defmt::Formatter) {
                        defmt::write!(f, "0x");
                        for i in (0 .. $len).rev() {
                            defmt::write!(f, "{:02x}", self.0[HEADER_LEN + i]);
                        }
                    }
                }

                /// Used to write to the register
                pub struct W(pub(crate) [u8; HEADER_LEN + $len]);

                impl W {
                    $(
                        #[$field_doc]
                        #[inline(always)]
                        pub fn $field(&mut self, value: $ty) -> &mut Self {
                            use crate::ll::ToBytes;

                            // Convert value into bytes
                            let source = <$ty as ToBytes>::to_bytes(value);

                            // Now, let's figure out where the bytes are located
                            // within the register array.
                            const START:          usize = $first_bit / 8;
                            const END:            usize = $last_bit  / 8 + 1;
                            const OFFSET_IN_BYTE: usize = $first_bit % 8;

                            // Also figure out the length of the value in bits.
                            // That's going to come in handy.
                            const LEN: usize = $last_bit - $first_bit + 1;


                            // We need to track how many bits are left in the
                            // value overall, and in the value's current byte.
                            let mut bits_left         = LEN;
                            let mut bits_left_in_byte = 8;

                            // We also need to track how many bits have already
                            // been written to the current target byte.
                            let mut bits_written_to_byte = 0;

                            // Now we can take the bytes from the value, shift
                            // them, mask them, and write them into the target
                            // array.
                            let mut source_i  = 0;
                            let mut target_i  = START;
                            while target_i < END {
                                // Values don't always end at byte boundaries,
                                // so we need to mask the bytes when writing to
                                // the slice.
                                // Let's start out assuming we can write to the
                                // whole byte of the slice. This will be true
                                // for the middle bytes of our value.
                                let mut mask = 0xff;

                                // Let's keep track of the offset we're using to
                                // write to this byte. We're going to need it.
                                let mut offset_in_this_byte = 0;

                                // If this is the first byte we're writing to
                                // the slice, we need to remove the lower bits
                                // of the mask.
                                if target_i == START {
                                    mask <<= OFFSET_IN_BYTE;
                                    offset_in_this_byte = OFFSET_IN_BYTE;
                                }

                                // If this is the last byte we're writing to the
                                // slice, we need to remove the higher bits of
                                // the mask. Please note that we could be
                                // writing to _both_ the first and the last
                                // byte.
                                if target_i == END - 1 {
                                    let shift =
                                        8 - bits_left - offset_in_this_byte;
                                    mask <<= shift;
                                    mask >>= shift;
                                }

                                mask <<= bits_written_to_byte;

                                // Read the value from `source`
                                let value = source[source_i]
                                    >> 8 - bits_left_in_byte
                                    << offset_in_this_byte
                                    << bits_written_to_byte;

                                // Zero the target bits in the slice, then write
                                // the value.
                                self.0[HEADER_LEN + target_i] &= !mask;
                                self.0[HEADER_LEN + target_i] |= value & mask;

                                // The number of bits that were expected to be
                                // written to the target byte.
                                let bits_needed = mask.count_ones() as usize;

                                // The number of bits we actually wrote to the
                                // target byte.
                                let bits_used = bits_needed.min(
                                    bits_left_in_byte - offset_in_this_byte
                                );

                                bits_left -= bits_used;
                                bits_written_to_byte += bits_used;

                                // Did we use up all the bits in the source
                                // byte? If so, we can move on to the next one.
                                if bits_left_in_byte > bits_used {
                                    bits_left_in_byte -= bits_used;
                                }
                                else {
                                    bits_left_in_byte =
                                        8 - (bits_used - bits_left_in_byte);

                                    source_i += 1;
                                }

                                // Did we write all the bits in the target byte?
                                // If so, we can move on to the next one.
                                if bits_used == bits_needed {
                                    target_i += 1;
                                    bits_written_to_byte = 0;
                                }
                            }

                            self
                        }
                    )*
                }
            }

            impl_rw!($rw, $name, $name_lower, $len);
        )*


        impl<SPI, CS> DW3000<SPI, CS> {
            $(
                #[$doc]
                pub fn $name_lower(&mut self) -> RegAccessor<$name, SPI, CS> {
                    RegAccessor(self, PhantomData)
                }
            )*
        }
    }
}

// Helper macro, used internally by `impl_register!`
macro_rules! impl_rw {
    (RO, $name:ident, $name_lower:ident, $len:expr) => {
        impl_rw!(@R, $name, $name_lower, $len);
    };
    (RW, $name:ident, $name_lower:ident, $len:expr) => {
        impl_rw!(@R, $name, $name_lower, $len);
        impl_rw!(@W, $name, $name_lower, $len);
    };

    (@R, $name:ident, $name_lower:ident, $len:expr) => {
        impl Readable for $name {
            type Read = $name_lower::R;

            fn read() -> Self::Read {
                $name_lower::R([0; Self::HEADER_LEN + $len])
            }

            fn buffer(r: &mut Self::Read) -> &mut [u8] {
                &mut r.0
            }
        }
    };
    (@W, $name:ident, $name_lower:ident, $len:expr) => {
        impl Writable for $name {
            type Write = $name_lower::W;

            fn write() -> Self::Write {
                $name_lower::W([0; Self::HEADER_LEN + $len])
            }

            fn buffer(w: &mut Self::Write) -> &mut [u8] {
                &mut w.0
            }
        }
    };
}

// All register are implemented in this macro invocation. It follows the
// following syntax:
// <id>, <sub-id>, <size-bytes>, <RO/RW>, <name-upper>(name-lower) { /// <doc>
//     <field 1>
//     <field 2>
//     ...
// }
//

/************************************************************************ */
/**********               DWM3000 MODIFICATIONS               *********** */
/************************************************************************ */
// registers for DWM3000
// Each field follows the following syntax:
// <Id>, <Offset>, <Length>, <Access>, <NAME(name)>
//      <name>, <first-bit-index>, <last-bit-index>, <type>; /// <doc>

impl_register! {

    0x00, 0x00, 4, RO, DEV_ID(dev_id) { /// Device identifier
        rev,     0,  3, u8;  /// Revision
        ver,     4,  7, u8;  /// Version
        model,   8, 15, u8;  /// Model
        ridtag, 16, 31, u16; /// Register Identification Tag
    }
    0x00, 0x04, 8, RW, EUI(eui) { /// Extended Unique Identifier
        value, 0, 63, u64; /// Extended Unique Identifier
    }
    0x00, 0x0C, 4, RW, PANADR(panadr) { /// PAN Identifier and Short Address
        short_addr,  0, 15, u16; /// Short Address
        pan_id,     16, 31, u16; /// PAN Identifier
    }
    0x00, 0x10, 4, RW, SYS_CFG(sys_cfg) { /// System Configuration
        ffen,        0,  0, u8; /// Frame Filtering Enable
        dis_fcs_tx,  1,  1, u8; /// disable auto-FCS Transmission
        dis_fce,     2,  2, u8; /// Disable frame check error handling
        dis_drxb,    3,  3, u8; /// Disable Double RX Buffer
        phr_mode,    4,  4, u8; /// PHR Mode
        phr_6m8,     5,  5, u8; /// Sets the PHR rate to match the data rate
        spi_crcen,   6,  6, u8; /// Enable SPI CRC functionnality
        cia_ipatov,  7,  7, u8; /// Select CIA processing preamble CIR
        cia_sts,     8,  8, u8; /// Select CIA processing STS CIR
        rxwtoe,      9,  9, u8; /// Receive Wait Timeout Enable
        rxautr,     10, 10, u8; /// Receiver Auto Re-enable
        auto_ack,   11, 11, u8; /// Automatic Acknowledge Enable
        cp_spc,     12, 13, u8; /// STS Packet Configuration
        cp_sdc,     15, 15, u8; /// configures the SDC
        pdoa_mode,  16, 17, u8; /// configure PDoA
        fast_aat,   18, 18, u8; /// enable fast RX to TX turn around mode
    }
    0x00, 0x14, 2, RW, FF_CFG(ff_cfg) { /// comments
        ffab,        0,  0, u8; /// Frame Filtering Allow Beacon
        ffad,        1,  1, u8; /// Frame Filtering Allow Data
        ffaa,        2,  2, u8; /// Frame Filtering Allow Acknowledgement
        ffam,        3,  3, u8; /// Frame Filtering Allow MAC Command Frame
        ffar,        4,  4, u8; /// Frame Filtering Allow Reserved
        ffamulti,    5,  5, u8; /// Frame Filtering Allow Multipurpose
        ffaf,        6,  6, u8; /// Frame Filtering Allow Fragmented
        ffae,        7,  7, u8; /// Frame Filtering Allow extended frame
        ffbc,        8,  8, u8; /// Frame Filtering Behave As Coordinator
        ffib,        9,  9, u8; /// Frame Filtering Allow MAC
        le0_pend,    10,  10, u8; /// Data pending for device at led0 addr
        le1_pend,    11,  11, u8; /// Data pending for device at led1 addr
        le2_pend,    12,  12, u8; /// Data pending for device at led2 addr
        le3_pend,    13,  13, u8; /// Data pending for device at led3 addr
        ssadrap,     14,  14, u8; /// Short Source Address Data Request
        lsadrape,    15,  15, u8; /// Long Source Address Data Request
    }
    0x00, 0x18, 1, RO, SPI_RD_CRC(spi_rd_crc) { /// SPI CRC read status
        value, 0, 7, u8; /// SPI CRC read status
    }
    0x00, 0x1C, 4, RO, SYS_TIME(sys_time) { ///  System Time Counter register
        value, 0, 31, u32; /// System Time Counter register
    }
    0x00, 0x24, 6, RW, TX_FCTRL(tx_fctrl) { /// TX Frame Control
        txflen,      0,  9, u16;  /// TX Frame Length
        txbr,       10, 10, u8; /// Transmit Bit Rate
        tr,         11, 11, u8; /// Transmit Ranging enable
        txpsr,      12, 15, u8; /// Transmit Preamble Symbol Repetitions
        txb_offset, 16, 25, u16; /// Transmit buffer index offset
        fine_plen,  40, 47, u8; /// Fine PSR control
    }
    0x00, 0x2C, 4, RW, DX_TIME(dx_time) { /// Delayed Send or Receive Time
        value, 0, 31, u32; /// Delayed Send or Receive Time
    }
    0x00, 0x30, 4, RW, DREF_TIME(dref_time) { ///  Delayed send or receive reference time
        value, 0, 31, u32; /// Delayed send or receive reference time
    }
    0x00, 0x4, 3, RW, RX_FWTO(rx_fwto) { /// Receive frame wait timeout period
        value, 0, 23, u32; /// Receive frame wait timeout period
    }
    0x00, 0x38, 1, RW, SYS_CTRL(sys_ctrl) { /// System Control Register
        value, 0, 7, u8; /// System control
    }
    0x00, 0x3C, 6, RW, SYS_ENABLE(sys_enable) { /// System event enable mask register
        cplock_en,      1,  1, u8; /// Mask clock PLL lock event
        spicrce_en,     2,  2, u8; /// Mask SPI CRC Error event
        aat_en,         3,  3, u8; /// Mask automatic acknowledge trigger event
        txfrb_en,       4,  4, u8; /// Mask transmit frame begins event
        txprs_en,       5,  5, u8; /// Mask transmit preamble sent event
        txphs_en,       6,  6, u8; /// Mask transmit PHY Header Sent event
        txfrs_en,       7,  7, u8; /// Mask transmit frame sent event
        rxprd_en,       8,  8, u8; /// Mask receiver preamble detected event
        rxsfdd_en,      9,  9, u8; /// Mask receiver SFD detected event
        ciadone_en,    10,  10, u8; /// Mask CIA processing done event
        rxphd_en,      11,  11, u8; /// Mask receiver PHY header detect event
        rxphe_en,      12,  12, u8; /// Mask receiver PHY header error event
        rxfr_en,       13,  13, u8; /// Mask receiver data frame ready event
        rxfcg_en,      14,  14, u8; /// Mask receiver FCS good event
        rxfce_en,      15,  15, u8; /// Mask receiver FCS error event
        rxrfsl_en,     16,  16, u8; /// Mask receiver Reed Solomon Frame Sync Loss event
        rxfto_en,      17,  17, u8; /// Mask Receive Frame Wait Timeout event
        ciaerr_en,     18,  18, u8; /// Mask leading edge detection processing error event
        vwarn_en,      19,  19, u8; /// Mask Voltage warning event
        rxovrr_en,     20,  20, u8; /// Receiver overrun
        rxpto_en,      21,  21, u8; /// Mask Preamble detection timeout event
        spirdy_en,     23,  23, u8; /// Mask SPI ready event
        rcinit_en,     24,  24, u8; /// Mask IDLE RC event
        pll_hilo_en,   25,  25, u8; /// Mask PLL Losing Lock warning event
        rxsto_en,      26,  26, u8; /// Mask Receive SFD timeout event
        hpdwarn_en,    27,  27, u8; /// Mask Half Period Delay Warning event
        cperr_en,      28,  28, u8; /// Mask Scramble Timestamp Sequence (STS) error event
        arfe_en,       29,  29, u8; /// Mask Automatic Frame Filtering rejection event
        rxprej_en,     33,  33, u8; /// Mask Receiver Preamble Rejection event
        vt_det_en,     36,  36, u8; /// Mask Voltage/Temperature variation dtection interrupt event
        gpioirq_en,    37,  37, u8; /// Mask GPIO interrupt event
        aes_done_en,   38,  38, u8; /// Mask AES done interrupt event
        aes_err_en,    39,  39, u8; /// Mask AES error interrupt event
        cdm_err_en,    40,  40, u8; /// Mask CMD error interrupt event
        spi_ovf_en,    41,  41, u8; /// Mask SPI overflow interrupt event
        spi_unf_en,    42,  42, u8; /// Mask SPI underflow interrupt event
        spi_err_en,    43,  43, u8; /// Mask SPI error interrupt event
        cca_fail_en,   44,  44, u8; /// Mask CCA fail interrupt event
    }
    0x00, 0x44, 6, RW, SYS_STATUS(sys_status) { /// System Event Status Register
        irqs,       0,  0, u8; /// Interrupt Request Status
        cplock,     1,  1, u8; /// Clock PLL Lock
        spicrce,    2,  2, u8; /// External Sync Clock Reset
        aat,        3,  3, u8; /// Automatic Acknowledge Trigger
        txfrb,      4,  4, u8; /// TX Frame Begins
        txprs,      5,  5, u8; /// TX Preamble Sent
        txphs,      6,  6, u8; /// TX PHY Header Sent
        txfrs,      7,  7, u8; /// TX Frame Sent
        rxprd,      8,  8, u8; /// RX Preamble Detected
        rxsfdd,     9,  9, u8; /// RX SFD Detected
        ciadone,   10, 10, u8; /// LDE Processing Done
        rxphd,     11, 11, u8; /// RX PHY Header Detect
        rxphe,     12, 12, u8; /// RX PHY Header Error
        rxfr,      13, 13, u8; /// RX Data Frame Ready
        rxfcg,     14, 14, u8; /// RX FCS Good
        rxfce,     15, 15, u8; /// RX FCS Error
        rxfsl,     16, 16, u8; /// RX Reed-Solomon Frame Sync Loss
        rxfto,     17, 17, u8; /// RX Frame Wait Timeout
        ciaerr,    18, 18, u8; /// Leading Edge Detection Error
        vwarn,     19, 19, u8; /// Low voltage warning
        rxovrr,    20, 20, u8; /// RX Overrun
        rxpto,     21, 21, u8; /// Preamble detection timeout
        spirdy,    23, 23, u8; /// SPI ready for host access
        rcinit,    24, 24, u8; /// RC INIT
        pll_hilo,  25, 25, u8; /// lock PLL Losing Lock
        rxsto,     26, 26, u8; /// Receive SFD timeout
        hpdwarn,   27, 27, u8; /// Half Period Delay Warning
        cperr,     28, 28, u8; /// Scramble Timestamp Sequence (STS) error
        arfe,      29, 29, u8; /// Automatic Frame Filtering rejection
        rxprej,    29, 29, u8; /// Receiver Preamble Rejection
        vt_det,    33, 33, u8; /// Voltage or temperature variation detected
        gpioirq,   36, 36, u8; /// GPIO interrupt
        aes_done,  37, 37, u8; /// AES-DMA operation complete
        aes_err,   38, 38, u8; /// AES-DMA error
        cmd_err,   39, 39, u8; /// Command error
        spi_ovf,   40, 40, u8; /// SPI overflow error
        spi_unf,   41, 41, u8; /// SPI underflow error
        spierr,    42, 42, u8; /// SPI collision error
        cca_fail,  43, 43, u8; /// This event will be set as a result of failure of CMD_CCA_TX to transmit a packet
    }
    0x00, 0x4C, 4, RO, RX_FINFO(rx_finfo) { /// RX Frame Information
        rxflen,  0,  9, u16; /// Receive Frame Length
        rxnspl, 11, 12, u8; /// Receive Non-Standard Preamble Length
        rxbr,   13, 13, u8; /// Receive Bit Rate Report
        rng,    15, 15, u8; /// Receiver Ranging
        rxprf,  16, 17, u8; /// RX Pulse Repetition Rate Report
        rxpsr,  18, 19, u8; /// RX Preamble Repetition
        rxpacc, 20, 31, u16; /// Preamble Accumulation Count
    }
    0x00, 0x64, 16, RO, RX_TIME(rx_time) { /// Receive Time Stamp
        rx_stamp,  0,  39, u64; /// Fully adjusted time stamp
        rx_rawst, 64, 95, u64; /// Raw time stamp
    }
    0x00, 0x74, 5, RO, TX_TIME(tx_time) { /// Transmit Time Stamp
        tx_stamp,  0, 39, u64; /// Fully adjusted time stamp
    }
    0x01, 0x00, 4, RO, TX_RAWST(tx_rawst) { /// Transmit time stamp raw
        value, 0, 31, u32; /// Transmit time stamp raw
    }
    0x01, 0x04, 2, RW, TX_ANTD(tx_antd) { /// Transmitter antenna delay
        value, 0, 15, u16; /// Transmitter antenna delay
    }
    0x01, 0x08, 4, RW, ACK_RESP(ack_resp) { /// Acknowledgement delay time and response time
        w4r_tim,  0, 19, u32; /// Wait-for-Response turn-around Time
        ack_tim,  24, 31, u8; /// Auto-Acknowledgement turn-around TimeC
    }
    0x01, 0x0C, 4, RW, TX_POWER(tx_power) { /// TX Power Control
        value, 0, 31, u32; /// TX Power Control value
    }
    0x01, 0x14, 2, RW, CHAN_CTRL(chan_ctrl) { /// Channel Control Register
        rf_chan,   0, 0, u8; /// Selects the receive channel.
        sfd_type,  1, 2, u8; /// Enables the non-standard Decawave proprietary SFD sequence.
        tx_pcode,  3, 7, u8; /// This field selects the preamble code used in the transmitter.
        rx_pcode,  8, 12, u8; /// This field selects the preamble code used in the receiver.
    }
    0x01, 0x18, 4, RW, LE_PEND_01(le_pend_01) { /// Low Energy device address 0 and 1
        le_addr0,  0, 15, u16; /// Low Energy device 16-bit address
        le_addr1, 16, 31, u16; /// Low Energy device 16-bit address
    }
    0x01, 0x1C, 4, RW, LE_PEND_23(le_pend_23) { /// Low Energy device address 2 and 3
        le_addr2,  0, 15, u16; /// Low Energy device 16-bit address
        le_addr3, 16, 31, u16; /// Low Energy device 16-bit address
    }
    0x01, 0x20, 1, RW, SPI_COLLISION(spi_collision) { /// SPI collision status
        value,  0, 7, u8; /// SPI collision status
    }
    0x01, 0x24, 1, RW, RDB_STATUS(rdb_status) { /// RX double buffer status
        rxfcg0,     0, 0, u8; /// Receiver FCS Good
        rxfr0,      1, 1, u8; /// Receiver Data Frame Ready
        ciadone0,   2, 2, u8; /// CIA processing done on the CIR relating to a message in RX_BUFFER_0 when operating in double buffer mode
        cp_err0,    3, 3, u8; /// Scramble Timestamp Sequence (STS) error
        rxfcg1,     4, 4, u8; /// Receiver FCS Good
        rxfr1,      5, 5, u8; /// Receiver Data Frame Ready
        ciadone1,   6, 6, u8; /// CIA processing done on the CIR relating to a message in RX_BUFFER_1 when operating in double buffer mode
        cp_err1,    7, 7, u8; /// Scramble Timestamp Sequence (STS) error
    }
    0x01, 0x28, 1, RW, RDB_DIAG(rdb_diag) { /// RX double buffer diagnostic configuration
        rdb_dmode,    0, 2, u8; /// RX double buffer diagnostic mode
    }
    0x01, 0x30, 2, RW, AES_CFG(aes_cfg) { /// AES configuration
        mode,        0, 0, u8; /// Mode of operation of AES core
        key_size,    1, 2, u8; /// AES Key Size
        key_addr,    3, 5, u8; /// Address offset of AES KEY
        key_load,    6, 6, u8; /// Load the AES KEY from AES KEY source
        key_src,     7, 7, u8; /// AES key source
        tag_size,    8, 10, u8; /// Size of AES tag field
        core_sel,    11, 11, u8; /// AES Core select
        key_otp,     12, 12, u8; /// AES key Memory source
    }
    0x01, 0x34, 4, RW, AES_IV0(aes_iv0) { /// AES GCM core mode
        value,  0, 31, u32; /// AES GCM core mode
    }
    0x01, 0x38, 4, RW, AES_IV1(aes_iv1) { /// AES GCM core mode
        value,  0, 31, u32; /// AES GCM core mode
    }
    0x01, 0x3C, 4, RW, AES_IV2(aes_iv2) { /// AES GCM core mode
        value,  0, 31, u32; /// AES GCM core mode
    }
    0x01, 0x40, 2, RW, AES_IV3(aes_iv3) { /// AES GCM core mode
        value,  0, 15, u16; /// AES GCM core mode
    }
    0x01, 0x42, 2, RW, AES_IV4(aes_iv4) { /// AES GCM core mode
        value,  0, 15, u16; /// AES GCM core mode
    }
    0x01, 0x44, 8, RW, DMA_CFG(dma_cfg) { /// DMA configuration register
        src_port,   0, 2, u8; /// Source memory port for DMA transfer
        src_addr,   3, 12, u16; /// Address offset within source memory for DMA transfer
        dst_port,   13, 15, u8; /// Destination memory port for DMA transfer
        dst_addr,   16, 25, u16; /// Address offset within destination memory for DMA transfer
        cp_end_sel, 26, 26, u8; /// Select the endianess of the CP seed port
        hdr_size,   32, 38, u8; /// Size of header field in the packet to be transferred via the DMA
        pyld_size,  39, 48, u8; /// Size of payload field in the packet to be transferred via the DMA
    }
    0x01, 0x4C, 1, RW, AES_START(aes_start) { /// Start AES operation
        value,  0, 0, u8; /// Start AES operation
    }
    0x01, 0x50, 4, RW, AES_STS(aes_sts) { /// The AES Status
        aes_done,  0, 0, u8; /// AES operation complete. Write 1 to clear
        auth_err,  1, 1, u8; /// AES authentication error. Write 1 to clear.
        trans_err,  2, 2, u8; /// Indicates error with DMA transfer to memory. Write 1 to clear
        mem_conf,  3, 3, u8; /// Indicates access conflict between multiple masters (SPI host, CIA engine and AES-DMA engine) trying to access same memory
        ram_empty,  4, 4, u8; /// Indicates AES scratch RAM is empty
        ram_full,  5, 5, u8; /// Indicates AES scratch RAM is full
    }
    0x01, 0x54, 16, RW, AES_KEY(aes_key) { /// The 128-bit KEY for the AES GCM/CCM* core
        value,  0x0, 0x7F, u128; /// value
    }

    /*******************************************************************/
    /**************    STS CONFIG REGISTER   ***************************/
    /*******************************************************************/
    0x02, 0x00, 2, RW, STS_CFG(sts_cfg) { /// STS configuration
        cps_len,  0, 7, u8; /// STS length
    }
    0x02, 0x04, 1, RW, STS_CTRL(sts_ctrl) { /// STS control
        load_iv,  0, 0, u8; /// Load STS_IV bit into the AES-128 block for the generation of STS
        rst_last, 1, 1, u8; /// Start from last, when it is set to 1 the STS generation starts from the last count that was used by the AES-128 block for the generation of the previous STS.
    }
    0x02, 0x08, 2, RW, STS_STS(sts_sts) { /// STS status
        acc_qual,  0, 11, u16; /// STS accumulation quality
    }
    0x02, 0x0C, 16, RW, STS_KEY(sts_key) { /// STS 128-bit KEY
        value,  0x0, 0x7F, u128; /// value
    }
    0x02, 0x1C, 16, RW, STS_IV(sts_iv) { /// STS 128-bit IV
        value,  0x0, 0x7F, u128; /// value
    }

    /*******************************************************************/
    /*****************    RX_TUNE REGISTER   ***************************/
    /*******************************************************************/
    0x03, 0x18, 2, RW, DGC_CFG(dgc_cfg) { /// RX tuning configuration register
        rx_tune_en,  0,  0, u8; /// RX tuning enable bit
        thr_64,      9, 14, u8; /// RX tuning threshold configuration for 64 MHz PRF
    }
    0x03, 0x1C, 4, RW, DGC_CFG0(dgc_cfg0) { /// DGC_CFG0
        value,  0, 31, u32; /// Value
    }
    0x03, 0x20, 4, RW, DGC_CFG1(dgc_cfg1) { /// DGC_CFG1
        value,  0, 31, u32; /// Value
    }
    0x03, 0x38, 4, RW, DGC_LUT_0(dgc_lut_0) { /// DGC_LUT_0
        value,  0, 31, u32; /// Value
    }
    0x03, 0x3C, 4, RW, DGC_LUT_1(dgc_lut_1) { /// DGC_LUT_1
        value,  0, 31, u32; /// Value
    }
    0x03, 0x40, 4, RW, DGC_LUT_2(dgc_lut_2) { /// DGC_LUT_2
        value,  0, 31, u32; /// Value
    }
    0x03, 0x44, 4, RW, DGC_LUT_3(dgc_lut_3) { /// DGC_LUT_3
        value,  0, 31, u32; /// Value
    }
    0x03, 0x48, 4, RW, DGC_LUT_4(dgc_lut_4) { /// DGC_LUT_4
        value,  0, 31, u32; /// Value
    }
    0x03, 0x4C, 4, RW, DGC_LUT_5(dgc_lut_5) { /// DGC_LUT_5
        value,  0, 31, u32; /// Value
    }
    0x03, 0x50, 4, RW, DGC_LUT_6(dgc_lut_6) { /// DGC_LUT_6
        value,  0, 31, u32; /// Value
    }
    0x03, 0x60, 4, RW, DGC_DBG(dgc_dbg) { /// Reports DGC information
        dgc_decision,  28,  30, u8; /// DGC decision index.
    }

    /*******************************************************************/
    /*****************    EXT_SYNC REGISTER   **************************/
    /*******************************************************************/
    0x04, 0x00, 4, RW, EC_CTRL(ec_ctrl) { /// External clock synchronisation counter configuration
        osts_wait,  3,  10, u8; /// Wait counter used for external timebase reset
        ostr_mode,  11,  11, u8; /// External timebase reset mode enable bit
    }
    0x04, 0x0C, 4, RW, RX_CAL(rx_cal) { /// RX calibration block configuration
        cal_mode,   0,   1, u8; /// RX calibration mode
        cal_en,     4,   7, u8; /// RX calibration enable
        comp_dly,  16,  19, u8; /// RX calibration tuning value
    }
    0x04, 0x14, 4, RW, RX_CAL_RESI(rx_cal_resi) { /// RX calibration block result
        value,  0,  28, u32; /// reports the result once the RX calibration is complete
    }
    0x04, 0x1C, 4, RW, RX_CAL_RESQ(rx_cal_resq) { /// RX calibration block result
        value,  0,  28, u32; /// reports the result once the RX calibration is complete
    }
    0x04, 0x20, 1, RW, RX_CAL_STS(rx_cal_sts) { /// RX calibration block status
        value,  0,  0, u8; ///  reports the status once the RX calibration is complete
    }

    /*******************************************************************/
    /*****************    GPIO_CTRL REGISTER   *************************/
    /*******************************************************************/
    0x05, 0x00, 4, RW, GPIO_MODE(gpio_mode) { /// GPIO Mode Control Register
        msgp0,  0,  2, u8; ///  Mode Selection for GPIO0/RXOKLED
        msgp1,  3,  5, u8; ///  Mode Selection for GPIO1/SFDLED
        msgp2,  6,  8, u8; ///  Mode Selection for GPIO2/RXLED
        msgp3,  9,  11, u8; ///  Mode Selection for GPIO3/TXLED
        msgp4,  12,  14, u8; ///  Mode Selection for GPIO4/EXTPA
        msgp5,  15,  17, u8; ///  Mode Selection for GPIO5/EXTTXE
        msgp6,  18,  20, u8; ///  Mode Selection for GPIO6/EXTRXE
        msgp7,  21,  23, u8; ///  Mode Selection for GPIO7
        msgp8,  24,  26, u8; ///  Mode Selection for GPIO8
    }
    0x05, 0x04, 2, RW, GPIO_PULL_EN(gpio_pull_en) { /// GPIO Drive Strength and Pull Control
        mgpen0,  0,  0, u8; ///  Setting to 0 will lower the drive strength
        mgpen1,  1,  1, u8; ///  Setting to 0 will lower the drive strength
        mgpen2,  2,  2, u8; ///  Setting to 0 will lower the drive strength
        mgpen3,  3,  3, u8; ///  Setting to 0 will lower the drive strength
        mgpen4,  4,  4, u8; ///  Setting to 0 will lower the drive strength
        mgpen5,  5,  5, u8; ///  Setting to 0 will lower the drive strength
        mgpen6,  6,  6, u8; ///  Setting to 0 will lower the drive strength
        mgpen7,  7,  7, u8; ///  Setting to 0 will lower the drive strength
        mgpen8,  8,  8, u8; ///  Setting to 0 will lower the drive strength
    }
    0x05, 0x08, 2, RW, GPIO_DIR(gpio_dir) { /// GPIO Direction Control Register
        gpd0,  0,  0, u8; ///   value of 0 means the pin is an output
        gpd1,  1,  1, u8; ///   value of 0 means the pin is an output
        gpd2,  2,  2, u8; ///   value of 0 means the pin is an output
        gpd3,  3,  3, u8; ///   value of 0 means the pin is an output
        gpd4,  4,  4, u8; ///   value of 0 means the pin is an output
        gpd5,  5,  5, u8; ///   value of 0 means the pin is an output
        gpd6,  6,  6, u8; ///   value of 0 means the pin is an output
        gpd7,  7,  7, u8; ///   value of 0 means the pin is an output
        gpd8,  8,  8, u8; ///   value of 0 means the pin is an output
    }
    0x05, 0x0C, 2, RW, GPIO_OUT(gpio_out) { /// GPIO Data Output Register
        gop0,  0,  0, u8; ///   show the current output setting
        gop1,  1,  1, u8; ///   show the current output setting
        gop2,  2,  2, u8; ///   show the current output setting
        gop3,  3,  3, u8; ///   show the current output setting
        gop4,  4,  4, u8; ///   show the current output setting
        gop5,  5,  5, u8; ///   show the current output setting
        gop6,  6,  6, u8; ///   show the current output setting
        gop7,  7,  7, u8; ///   show the current output setting
        gop8,  8,  8, u8; ///   show the current output setting
    }
    0x05, 0x10, 2, RW, GPIO_IRQE(gpio_irqe) { /// GPIO Interrupt Enable
        girqe0,  0,  0, u8; ///   selected as interrupt source
        girqe1,  1,  1, u8; ///   selected as interrupt source
        girqe2,  2,  2, u8; ///   selected as interrupt source
        girqe3,  3,  3, u8; ///   selected as interrupt source
        girqe4,  4,  4, u8; ///   selected as interrupt source
        girqe5,  5,  5, u8; ///   selected as interrupt source
        girqe6,  6,  6, u8; ///   selected as interrupt source
        girqe7,  7,  7, u8; ///   selected as interrupt source
        girqe8,  8,  8, u8; ///   selected as interrupt source
    }
    0x05, 0x14, 2, RW, GPIO_ISTS(gpio_ists) { /// GPIO Interrupt Status
        gists0,  0,  0, u8; ///   Value 1 means GPIO gave rise to the GPIOIRQ SYS_STATUS event
        gists1,  1,  1, u8; ///   Value 1 means GPIO gave rise to the GPIOIRQ SYS_STATUS event
        gists2,  2,  2, u8; ///   Value 1 means GPIO gave rise to the GPIOIRQ SYS_STATUS event
        gists3,  3,  3, u8; ///   Value 1 means GPIO gave rise to the GPIOIRQ SYS_STATUS event
        gists4,  4,  4, u8; ///   Value 1 means GPIO gave rise to the GPIOIRQ SYS_STATUS event
        gists5,  5,  5, u8; ///   Value 1 means GPIO gave rise to the GPIOIRQ SYS_STATUS event
        gists6,  6,  6, u8; ///   Value 1 means GPIO gave rise to the GPIOIRQ SYS_STATUS event
        gists7,  7,  7, u8; ///   Value 1 means GPIO gave rise to the GPIOIRQ SYS_STATUS event
        gists8,  8,  8, u8; ///   Value 1 means GPIO gave rise to the GPIOIRQ SYS_STATUS event
    }
    0x05, 0x18, 2, RW, GPIO_ISEN(gpio_isen) { /// GPIO Interrupt Sense Selection
        gisen0,  0,  0, u8; ///   GPIO IRQ Sense selection GPIO input
        gisen1,  1,  1, u8; ///   GPIO IRQ Sense selection GPIO input
        gisen2,  2,  2, u8; ///   GPIO IRQ Sense selection GPIO input
        gisen3,  3,  3, u8; ///   GPIO IRQ Sense selection GPIO input
        gisen4,  4,  4, u8; ///   GPIO IRQ Sense selection GPIO input
        gisen5,  5,  5, u8; ///   GPIO IRQ Sense selection GPIO input
        gisen6,  6,  6, u8; ///   GPIO IRQ Sense selection GPIO input
        gisen7,  7,  7, u8; ///   GPIO IRQ Sense selection GPIO input
        gisen8,  8,  8, u8; ///   GPIO IRQ Sense selection GPIO input
    }
    0x05, 0x1C, 2, RW, GPIO_IMODE(gpio_imode) { /// GPIO Interrupt Mode (Level / Edge)
        gimod0,  0,  0, u8; ///   GPIO IRQ Mode selection for GPIO input
        gimod1,  1,  1, u8; ///   GPIO IRQ Mode selection for GPIO input
        gimod2,  2,  2, u8; ///   GPIO IRQ Mode selection for GPIO input
        gimod3,  3,  3, u8; ///   GPIO IRQ Mode selection for GPIO input
        gimod4,  4,  4, u8; ///   GPIO IRQ Mode selection for GPIO input
        gimod5,  5,  5, u8; ///   GPIO IRQ Mode selection for GPIO input
        gimod6,  6,  6, u8; ///   GPIO IRQ Mode selection for GPIO input
        gimod7,  7,  7, u8; ///   GPIO IRQ Mode selection for GPIO input
        gimod8,  8,  8, u8; ///   GPIO IRQ Mode selection for GPIO input
    }
    0x05, 0x20, 2, RW, GPIO_IBES(gpio_ibes) { /// GPIO Interrupt “Both Edge” Select
        gibes0,  0,  0, u8; ///   GPIO IRQ “Both Edge” selection for GPIO input
        gibes1,  1,  1, u8; ///   GPIO IRQ “Both Edge” selection for GPIO input
        gibes2,  2,  2, u8; ///   GPIO IRQ “Both Edge” selection for GPIO input
        gibes3,  3,  3, u8; ///   GPIO IRQ “Both Edge” selection for GPIO input
        gibes4,  4,  4, u8; ///   GPIO IRQ “Both Edge” selection for GPIO input
        gibes5,  5,  5, u8; ///   GPIO IRQ “Both Edge” selection for GPIO input
        gibes6,  6,  6, u8; ///   GPIO IRQ “Both Edge” selection for GPIO input
        gibes7,  7,  7, u8; ///   GPIO IRQ “Both Edge” selection for GPIO input
        gibes8,  8,  8, u8; ///   GPIO IRQ “Both Edge” selection for GPIO input
    }
    0x05, 0x24, 4, RW, GPIO_ICLR(gpio_iclr) { /// GPIO Interrupt Latch Clear
        giclr0,  0,  0, u8; ///   GPIO IRQ latch clear for GPIO input
        giclr1,  1,  1, u8; ///   GPIO IRQ latch clear for GPIO input
        giclr2,  2,  2, u8; ///   GPIO IRQ latch clear for GPIO input
        giclr3,  3,  3, u8; ///   GPIO IRQ latch clear for GPIO input
        giclr4,  4,  4, u8; ///   GPIO IRQ latch clear for GPIO input
        giclr5,  5,  5, u8; ///   GPIO IRQ latch clear for GPIO input
        giclr6,  6,  6, u8; ///   GPIO IRQ latch clear for GPIO input
        giclr7,  7,  7, u8; ///   GPIO IRQ latch clear for GPIO input
        giclr8,  8,  8, u8; ///   GPIO IRQ latch clear for GPIO input
    }
    0x05, 0x28, 4, RW, GPIO_IDBE(gpio_idbe) { /// GPIO Interrupt De-bounce Enable
        gidbe0,  0,  0, u8; ///   GPIO IRQ de-bounce enable for GPIO
        gidbe1,  1,  1, u8; ///   GPIO IRQ de-bounce enable for GPIO
        gidbe2,  2,  2, u8; ///   GPIO IRQ de-bounce enable for GPIO
        gidbe3,  3,  3, u8; ///   GPIO IRQ de-bounce enable for GPIO
        gidbe4,  4,  4, u8; ///   GPIO IRQ de-bounce enable for GPIO
        gidbe5,  5,  5, u8; ///   GPIO IRQ de-bounce enable for GPIO
        gidbe6,  6,  6, u8; ///   GPIO IRQ de-bounce enable for GPIO
        gidbe7,  7,  7, u8; ///   GPIO IRQ de-bounce enable for GPIO
        gidbe8,  8,  8, u8; ///   GPIO IRQ de-bounce enable for GPIO
    }
    0x05, 0x2C, 2, RO, GPIO_RAW(gpio_raw) { /// GPIO Raw State
        grawp0,  0,  0, u8; ///   GPIO port raw state
        grawp1,  1,  1, u8; ///   GPIO port raw state
        grawp2,  2,  2, u8; ///   GPIO port raw state
        grawp3,  3,  3, u8; ///   GPIO port raw state
        grawp4,  4,  4, u8; ///   GPIO port raw state
        grawp5,  5,  5, u8; ///   GPIO port raw state
        grawp6,  6,  6, u8; ///   GPIO port raw state
        grawp7,  7,  7, u8; ///   GPIO port raw state
        grawp8,  8,  8, u8; ///   GPIO port raw state
    }

    /*******************************************************************/
    /*****************    DRX_CONF REGISTER    *************************/
    /*******************************************************************/
    0x06, 0x00, 2, RW, DTUNE0(dtune0) { /// PAC configuration
        pac,    0,  1, u8; ///   Preamble Acquisition Chunk size
        dt0b4,  4,  4, u8; ///   Tuning bit 4 of digital tuning reg0
    }
    0x06, 0x02, 2, RW, RX_SFD_TOC(rx_sfd_toc) { /// SFD timeout
        value,  0,  15, u16; /// don't set to 0
    }
    0x06, 0x04, 2, RW, PRE_TOC(pre_toc) { /// Preamble detection timeout
        value,  0,  15, u16; /// digital receiver configuration
    }
    0x06, 0x0C, 4, RW, DTUNE3(dtune3) { /// Receiver tuning register
        value,  0,  31, u32; /// value
    }
    0x06, 0x14, 4, RO, DTUNE5(dtune5) { /// Digital Tuning Reserved register
        value,  0,  31, u32; /// value
    }
    0x06, 0x29, 3, RO, DRX_CAR_INT(drx_car_int) { /// Carrier recovery integrator register
        //  formule de math !! A FINIR // TODO
    }

    /*******************************************************************/
    /*****************     RF_CONF REGISTER    *************************/
    /*******************************************************************/
    0x07, 0x00, 4, RW, RF_ENABLE(rf_enable) { /// RF control enable
        value,  0,  31, u32; /// value
    }
    0x07, 0x04, 4, RW, RF_CTRL_MASK(rf_ctrl_mask) { /// RF enable mask
        value,  0,  31, u32; /// value
    }
    0x07, 0x14, 4, RW, RF_SWITCH(rf_switch) { /// RF switch configuration
        antswnotoggle,  0,  0, u8; /// When set to 1, the automatic toggling of the antenna switch is disabled when the device is operating in PDoA modes
        antswpdoaport,  1,  1, u8; /// Specifies the starting port for reception when the device is operating in PDoA modes
        antswen,        8,  8, u8; /// Setting this to 1 will enable manual control of the antenna switch
        antswctrl,     12, 14, u8; /// Manual control of antenna switch when ANTSWEN is set
        trxswen,       16, 16, u8; /// Setting this to 1 will enable manual control of the TX RX switch
        trxswctrl,     24, 29, u8; /// TX/RX switch control when TRXSWEN bit is set
    }
    0x07, 0x1A, 1, RW, RF_TX_CTRL_1(rf_tx_ctrl_1) { /// RF transmitter configuration
        value,  0,  7, u8; /// value
    }
    0x07, 0x1C, 4, RW, RF_TX_CTRL_2(rf_tx_ctrl_2) { /// RF transmitter configuration
        value,  0,  31, u32; /// Pulse Generator Delay value
    }
    0x07, 0x28, 1, RW, TX_TEST(tx_test) { /// Transmitter test configuration
        tx_entest,  0,  3, u8; /// Transmitter test enable
    }
    0x07, 0x34, 1, RW, SAR_TEST(rsar_test) { /// Transmitter Calibration – SAR temperaturesensor read enable
        sar_rden,  2,  2, u8; /// Writing 1 enables the SAR temperature sensor reading
    }
    0x07, 0x40, 8, RW, LDO_TUNE(ldo_tune) { /// Internal LDO voltage tuning parameter
        value,  0x00,  0x3C, u128; ///  used to control the output voltage levels of the on chip LDOs
    }
    0x07, 0x48, 4, RW, LDO_CTRL(ldo_ctrl) { /// LDO control
        value,  0,  31, u32; ///  LDO control
    }
    0x07, 0x51, 1, RW, LDO_RLOAD(ldo_rload) { /// LDO tuning register
        value,  0,  7, u8; ///  LDO tuning register
    }

    /*******************************************************************/
    /*****************     RF_CAL REGISTER    **************************/
    /*******************************************************************/
    0x08, 0x00, 1, RW, SAR_CTRL(sar_ctrl) { /// Transmitter Calibration – SAR control
        sar_start, 0, 0, u8; /// Writing 1 sets SAR enable and writing 0 clears the enable.
    }
    0x08, 0x04, 1, RW, SAR_STATUS(sar_status) { /// Transmitter Calibration – SAR  status
        sar_done, 0, 0, u8; /// Set to 1 when the data is ready to be read.
    }
    0x08, 0x08, 3, RO, SAR_READING(sar_reading) { /// Transmitter Calibration –Latest SAR readings
        sar_lvbat, 0,  7, u8; /// Latest SAR reading for Voltage level.
        sar_ltemp, 8, 15, u8; /// Latest SAR reading for Temperature level.
    }
    0x08, 0x0C, 2, RO, SAR_WAKE_RD(sar_wake_rd) { /// Transmitter Calibration – SAR readings at last wake-up
        sar_wvbat, 0,  7, u8; /// SAR reading of Voltage level taken at last wake up event.
        sar_wtemp, 8, 15, u8; /// To read the temp, use SAR_READING instead.
    }
    0x08, 0x10, 2, RW, PGC_CTRL(pgc_ctrl) { /// Transmitter Calibration – Pulse Generator control
        pg_start,     0, 0, u8; /// Start the pulse generator calibration.
        pgc_auto_cal, 1, 1, u8; /// Start the pulse generator auto-calibration.
        pgc_tmeas,    2, 5, u8; /// Number of clock cycles over which to run the pulse generator calibration counter.
    }
    0x08, 0x14, 2, RO, PGC_STATUS(pgc_status) { /// Transmitter Calibration – Pulse Generator status
        pg_delay_cnt,  0, 11, u16; /// Pulse generator count value
        autocal_done, 12, 12, u8; /// Auto-calibration of the PG_DELAY  has completed.
    }
    0x08, 0x18, 2, RW, PG_TEST(pg_test) { /// Transmitter Calibration – Pulse Generator test
        value, 0, 15, u16; /// Pulse Generator test
    }
    0x08, 0x1C, 2, RO, PG_CAL_TARGET(pg_cal_target) { /// Transmitter Calibration – Pulse Generator count target value
        value, 0, 11, u16; /// Pulse generator target value of PG_COUNT at which point PG auto cal will complete.
    }

    /*******************************************************************/
    /*****************     FS_CTRL REGISTER    *************************/
    /*******************************************************************/
    0x09, 0x00, 2, RW, PLL_CFG(pll_cfg) { /// PLL configuration
        value, 0, 15, u16; /// PLL configuration
    }
    0x09, 0x04, 3, RW, PLL_CC(pll_cc) { /// PLL coarse code – starting code for calibration procedure
        ch9_code, 0,  7, u8; /// PLL calibration coarse code for channel 5.
        ch5_code, 8, 21, u8; /// PLL calibration coarse code for channel 9.
    }
    0x09, 0x08, 2, RW, PLL_CAL(pll_cal) { /// PLL calibration configuration
        use_old,    1, 1, u8; /// Use the coarse code value as set in PLL_CC register as starting point for PLL calibration.
        pll_cfg_ld, 4, 7, u8; /// PLL calibration configuration value.
        cal_en,     8, 8, u8; /// PLL  calibration  enable  bit.
    }
    0x09, 0x14, 1, RW, XTAL(xtal) { /// Frequency synthesiser – Crystal trim
        value, 0, 7, u8; /// Crystal Trim.
    }

    /*******************************************************************/
    /*********************     AON REGISTER    *************************/
    /*******************************************************************/
    0x0A, 0x00, 3, RW, AON_DIG_CFG(aon_dig_cfg) { /// AON wake up configuration register
        onw_aon_dld, 0,  0, u8; /// On Wake-up download the AON array.
        onw_run_sar, 1,  1, u8; /// On Wake-up Run the (temperature and voltage) Analog-to-Digital Convertors.
        onw_go2idle, 8,  8, u8; /// On Wake-up go to IDLE_PLL state.
        onw_go2rx,   9,  9, u8; /// On Wake-up go to RX.
        onw_pgfcal, 11, 11, u8; /// On Wake-up perform RX calibration
    }
    0x0A, 0x04, 1, RW, AON_CTRL(aon_ctrl) { /// AON control register
        restore,      0, 0, u8; /// Copy the user configurations from the AON memory to the host interface register set.
        save,         1, 1, u8; /// Copy the user configurations from the host interface register  set  into  the  AON  memory.
        cfg_upload,   2, 2, u8; /// Upload the AON block configurations to the AON.
        dca_read,     3, 3, u8; /// Direct AON memory access read.
        dca_write,    4, 4, u8; /// Direct AON memory write access
        dca_write_hi, 5, 5, u8; /// Direct AON memory write access. Needs to be set when using address > 0xFF
        dca_enab,     7, 7, u8; /// Direct AON memory access enable bit.
    }
    0x0A, 0x08, 1, RW, AON_RDATA(aon_rdata) { /// AON direct access read data result
        value, 0, 7, u8; /// AON direct access read data result
    }
    0x0A, 0x0C, 2, RW, AON_ADDR(aon_addr) { /// AON direct access address
        value, 0, 15, u16; /// AON direct access address
    }
    0x0A, 0x10, 1, RW, AON_WDATA(aon_wdata) { /// AON direct access write data
        value, 0, 7, u8; /// AON direct access write data
    }
    0x0A, 0x14, 1, RW, AON_CFG(aon_cfg) { /// AON configuration register
        sleep_en,   0, 0, u8; /// Sleep enable configuration bit.
        wake_cnt,   1, 1, u8; /// Wake when sleep counter elapses.
        brout_en,   2, 2, u8; /// Enable the BROWNOUT detector during SLEEP or DEEPSLEEP.
        wake_csn,   3, 3, u8; /// Wake using SPI access.
        wake_wup,   4, 4, u8; /// Wake using WAKEUP pin.
        pres_sleep, 5, 5, u8; /// Preserve Sleep.
    }

    /*******************************************************************/
    /******************     OTP_IF REGISTER    *************************/
    /*******************************************************************/
    0x0B, 0x00, 4, RW, OTP_WDATA(otp_wdata) { /// OTP data to program to a particular address
        value, 0, 31, u32; /// OTP data to program to a particular address
    }
    0x0B, 0x04, 4, RW, OTP_ADDR(otp_addr) { /// OTP address to which to program the data
        otp_addr, 0, 10, u16; /// Address within OTP memory that will be accessed read or written.
    }
    0x0B, 0x08, 2, RW, OTP_CFG(otp_cfg) { /// OTP configuration register
        otp_man,       0,  0, u8; /// Enable manual control over OTP interface.
        otp_read,      1,  1, u8; /// OTP read enable.
        otp_write,     2,  2, u8; /// OTP write enable.
        otp_write_mr,  3,  3, u8; /// OTP write mode.
        dgc_kick,      6,  6, u8; /// Loading of the RX_TUNE_CAL parameter
        ldo_kick,      7,  7, u8; /// Loading of the LDOTUNE_CAL parameter
        bias_kick,     8,  8, u8; /// Loading of the BIASTUNE_CAL parameter
        ops_kick,     10, 10, u8; /// Loading of the operating parameter set selected by the OPS_SEL configuration
        ops_sel,      11, 12, u8; /// Operating parameter set selection.
        dgc_sel,      13, 13, u8; /// RX_TUNE parameter set selection.
    }
    0x0B, 0x0C, 1, RW, OTP_STAT(otp_stat) { /// OTP memory programming status register
        otp_prog_done, 0,  0, u8; /// OTP Programming Done
        otp_vpp_ok,    1,  1, u8; /// OTP Programming Voltage OK.
    }
    0x0B, 0x10, 4, RO, OTP_RDATA(otp_rdata) { /// OTP data read from given address
        value, 0, 31, u32; /// OTP data read from given address
    }
    0x0B, 0x14, 4, RW, OTP_SRDATA(otp_srdata) { /// OTP Special Register (SR) read data
        value, 0, 31, u32; /// OTP Special Register (SR) read data
    }

    /*******************************************************************/
    /*********************     CIA REGISTER    *************************/
    /*******************************************************************/
    0x0C, 0x00, 8, RO, IP_TS(ip_ts) { /// Preamble sequence receive time stamp and status
        ip_toa,    0,  39, u64; /// Preamble sequence Time of Arrival estimate.
        ip_poa,   40,  53, u16; /// Phase of arrival as computed from the preamble CIR.
        ip_toast, 56,  63, u8; /// Preamble sequence Time of Arrival status indicator.
    }
    0x0C, 0x08, 8, RO, STS_TS(sts_ts) { /// STS receive time stamp and status
        sts_toa,    0,  39, u64; /// STS Time of Arrival estimate.
        sts_poa,   40,  53, u16; /// Phase of arrival as computed from the STS CIR.
        sts_toast, 55,  63, u16; /// STS sequence Time of Arrival status indicator.
    }
    0x0C, 0x10, 8, RO, STS1_TS(sts1_ts) { /// 2nd STS receive time stamp and status
        sts1_toa,    0,  39, u64; /// STS second Time of Arrival estimate.
        sts1_poa,   40,  53, u16; /// Phase of arrival as computed from the STS based CIR estimate.
        sts1_toast, 55,  63, u16; /// STS second Time of Arrival status indicator.
    }
    0x0C, 0x18, 6, RO, TDOA(tdoa) { /// The TDoA between the two CIRs
        value, 0, 47, u64; /// The TDoA between the two CIRs
    }
    0x0C, 0x1E, 2, RO, PDOA(pdoa) { /// The PDoA between the two CIRs
        pdoa,      0, 13, u16; /// Phase difference result.
        fp_th_md, 14, 14, u8; /// First path threshold test mode.
    }
    0x0C, 0x20, 4, RO, CIA_DIAG_0(cia_diag_0) { /// CIA Diagnostic 0
        coe_ppm, 0, 12, u16; /// Clock offset estimate.
    }
    0x0C, 0x24, 4, RO, CIA_DIAG_1(cia_diag_1) { /// Reserved diagnostic data
    }
    0x0C, 0x28, 4, RO, IP_DIAG_0(ip_diag_0) { /// Preamble Diagnostic 0 – peak
        ip_peaka,  0, 20, u32; /// Amplitude of the sample accumulated using the preamble sequence.
        ip_peaki, 21, 30, u16; /// Index of the sample accumulated using the preamble sequence.
    }
    0x0C, 0x2C, 4, RO, IP_DIAG_1(ip_diag_1) { /// Preamble Diagnostic 1 – power indication
        ip_carea, 0, 16, u32; /// Channel area accumulated using the preamble sequence.
    }
    0x0C, 0x30, 4, RO, IP_DIAG_2(ip_diag_2) { /// Preamble Diagnostic 2 – magnitude @ FP + 1
        ip_fp1m, 0, 21, u32; /// Magnitude of the sample at the first index immediately after the estimated first path position accumulated using the preamble sequence.
    }
    0x0C, 0x34, 4, RO, IP_DIAG_3(ip_diag_3) { /// Preamble Diagnostic 3 – magnitude @ FP + 2
        ip_fp2m, 0, 21, u32; /// Magnitude of the sample at the second index immediately after the estimated first path position accumulated using the preamble sequence.
    }
    0x0C, 0x38, 4, RO, IP_DIAG_4(ip_diag_4) { /// Preamble Diagnostic 4 – magnitude @ FP + 3
        ip_fp3m, 0, 21, u32; /// Magnitude of the sample at the third index immediately after the estimated first path position accumulated using the preamble sequence.
    }
    0x0C, 0x3C, 12, RO, IP_DIAG_RES1(ip_diag_res1) { /// Reserved diagnostic data
    }
    0x0C, 0x48, 4, RO, IP_DIAG_8(ip_diag_8) { /// Preamble Diagnostic 8 – first path
        ip_fp, 0, 15, u16; /// Estimated first path location accumulated using the preamble sequence.
    }
    0x0C, 0x4C, 12, RO, IP_DIAG_RES2(ip_diag_res2) { /// Reserved diagnostic data
    }
    0x0C, 0x58, 4, RO, IP_DIAG_12(ip_diag_12) { /// Preamble Diagnostic 12 – symbols accumulated
        ip_nacc, 0, 11, u16; /// Number of preamble sequence symbols that were accumulated to form the preamble CIR.
    }
    0x0C, 0x5C, 4, RO, STS_DIAG_0(sts_diag_0) { /// STS 0 Diagnostic 0 – STS CIA peak amplitude
        cp0_peaka,  0, 20, u32; /// Amplitude of the sample accumulated using the STS
        cp0_peaki, 21, 29, u16; /// Index of the sample accumulated using the STS
    }
    0x0C, 0x60, 4, RO, STS_DIAG_1(sts_diag_1) { /// STS 0 Diagnostic 1 – STS power indication
        cp0_carea, 0, 15, u16; /// Channel area accumulated using the the STS
    }
    0x0C, 0x64, 4, RO, STS_DIAG_2(sts_diag_2) { /// STS 0 Diagnostic 2 – STS magnitude @ FP + 1
        cp0_fp1m, 0, 21, u32; /// Magnitude of the sample at the first index immediately after the estimated first path position accumulated using the STS
    }
    0x0C, 0x68, 4, RO, STS_DIAG_3(sts_diag_3) { /// STS 0 Diagnostic 3 – STS magnitude @ FP + 2
        cp0_fp2m, 0, 21, u32; /// Magnitude of the sample at the second index immediately after the estimated first path position accumulated using the STS
    }
    0x0D, 0x00, 4, RO, STS_DIAG_4(sts_diag_4) { /// STS 0 Diagnostic 4 – STS magnitude @ FP + 3
        cp0_fp3m, 0, 21, u32; /// Magnitude of the sample at the third index immediately after the estimated first path position accumulated using the STS
    }
    0x0D, 0x04, 12, RO, STS0_DIAG_RES1(sts0_diag_res1) { /// Reserved diagnostic data
    }
    0x0D, 0x10, 4, RO, STS_DIAG_8(sts_diag_8) { /// STS 0 Diagnostic 8 – STS first path
        cp0_fp, 0, 14, u16; /// Estimated first path location accumulated using the STS
    }
    0x0D, 0x14, 12, RO, STS0_DIAG_RES2(sts0_diag_res2) { /// Reserved diagnostic data
    }
    0x0D, 0x20, 4, RO, STS_DIAG_12(sts_diag_12) { /// STS 0 diagnostic 12 – accumulated STS length
        cp0_nacc, 0, 10, u16; /// Number of preamble sequence symbols that were accumulated to form the preamble CIR.
    }
    0x0D, 0x24, 20, RO, STS0_DIAG_RES3(sts0_diag_res3) { /// Reserved diagnostic data
    }
    0x0D, 0x38, 4, RO, STS1_DIAG_0(sts1_diag_0) { /// STS 1 Diagnostic 0 – STS CIA peak amplitude
        cp1_peaka,  0, 20, u32; /// Amplitude of the sample accumulated using the STS
        cp1_peaki, 21, 29, u16; /// Index of the sample accumulated using the STS
    }
    0x0D, 0x3C, 4, RO, STS1_DIAG_1(sts1_diag_1) { /// STS 1 Diagnostic 1 – STS power indication
        cp1_carea, 0, 15, u16; /// Channel area accumulated using the the STS
    }
    0x0D, 0x40, 4, RO, STS1_DIAG_2(sts1_diag_2) { /// STS 1 Diagnostic 2 – STS magnitude @ FP + 1
        cp1_fp1m, 0, 21, u32; /// Magnitude of the sample at the first index immediately after the estimated first path position accumulated using the STS
    }
    0x0D, 0x44, 4, RO, STS1_DIAG_3(sts1_diag_3) { /// STS 1 Diagnostic 3 – STS magnitude @ FP + 2
        cp1_fp2m, 0, 21, u32; /// Magnitude of the sample at the second index immediately after the estimated first path position accumulated using the STS
    }
    0x0D, 0x48, 4, RO, STS1_DIAG_4(sts1_diag_4) { /// STS 1 Diagnostic 4 – STS magnitude @ FP + 3
        cp1_fp3m, 0, 21, u32; /// Magnitude of the sample at the third index immediately after the estimated first path position accumulated using the STS
    }
    0x0D, 0x4C, 12, RO, STS1_DIAG_RES1(sts1_diag_res1) { /// Reserved diagnostic data
    }
    0x0D, 0x58, 4, RO, STS1_DIAG_8(sts1_diag_8) { /// STS 1 Diagnostic 8 – STS first path
        cp1_fp, 0, 14, u16; /// Estimated first path location accumulated using the STS
    }
    0x0D, 0x5C, 12, RO, STS1_DIAG_RES2(sts1_diag_res2) { /// Reserved diagnostic data
    }
    0x0D, 0x68, 4, RO, STS1_DIAG_12(sts1_diag_12) { /// STS 1 Diagnostic 12 – STS accumulated STS length
        cp1_nacc, 0, 10, u16; /// Number of preamble sequence symbols that were accumulated to form the preamble CIR.
    }
    0x0E, 0x00, 4, RW, CIA_CONF(cia_conf) { /// CIA general configuration
        rxantd,   0, 15, u16; /// Configures the receive antenna delay.
        mindiag, 20, 20, u8; ///  Minimum Diagnostics.
    }
    0x0E, 0x04, 4, RW, FP_CONF(fp_conf) { /// First path temp adjustment and thresholds
        fp_agreed_th, 8, 10, u8; /// The threshold to use when performing the FP_AGREE test.
        cal_temp,    11, 18, u8; /// Temperature at which the device was calibrated.
        tc_rxdly_en, 20, 20, u8; /// Temperature compensation for RX antenna delay.
    }
    0x0E, 0x0C, 4, RW, IP_CONF(ip_conf) { /// Preamble Config – CIA preamble configuration
        ip_ntm,   0, 4,  u8; /// Preamble Noise Threshold Multiplier.
        ip_pmult, 5, 6,  u8; /// Preamble Peak Multiplier.
        ip_rtm,  16, 20, u8; /// Preamble replica threshold multiplier
    }
    0x0E, 0x12, 4, RW, STS_CONF_0(sts_conf_0) { /// STS Config 0 – CIA STS configuration
        sts_ntm,   0,  4, u8; /// STS Noise Threshold Multiplier.
        sts_pmult, 5,  6, u8; /// STS Peak Multiplier.
        sts_rtm,  16, 22, u8; /// STS replica threshold multiplier
    }
    0x0E, 0x16, 4, RW, STS_CONF_1(sts_conf_1) { /// STS Config 1 – CIA STS configuration
        res_b0,        0,  7, u8; /// Tuning value
        fp_agreed_en, 28, 28, u8; /// Checks to see if the two ToA estimates are within allowed tolerances.
        sts_cq_en,    29, 29, u8; /// Checks how consistent the impulse response stays during the accumulation of the STS.
        sts_ss_en,    30, 30, u8; /// Compare the sampling statistics of the STS reception to those of the earlier reception of the preamble sequence.
        sts_pgr_en,   31, 31, u8; /// Test the growth rate of the STS based CIR to the earlier growth rate of the preamble based CIR.
    }
    0x0E, 0x1A, 2, RW, CIA_ADJUST(cia_adjust) { /// User adjustment to the PDoA
        value, 0, 13, u8; /// Adjustment value to account for non-balanced antenna circuits.
    }

    /*******************************************************************/
    /*****************     DIG_DIAG REGISTER    ************************/
    /*******************************************************************/
    0x0F, 0x00, 1, RW, EVC_CTRL(evc_ctrl) { /// Event counter control
        evc_en,  0, 0, u8; /// Event Counters Enable.
        evc_clr, 1, 1, u8; /// Event Counters Clear.
    }
    0x0F, 0x04, 2, RO, EVC_PHE(evc_phe) { /// PHR error counter
        value, 0, 11, u16; /// PHR Error Event Counter.
    }
    0x0F, 0x06, 2, RO, EVC_RSE(evc_rse) { /// RSD error counter
        value, 0, 11, u16; /// Reed Solomon decoder (Sync Loss) Error Event Counter.
    }
    0x0F, 0x08, 2, RO, EVC_FCG(evc_fcg) { /// Frame check sequence good counter
        value, 0, 11, u16; /// Frame Check Sequence Good Event Counter.
    }
    0x0F, 0x0A, 2, RO, EVC_FCE(evc_fce) { /// Frame Check Sequence error counter
        value, 0, 11, u16; /// Frame Check Sequence Error Event Counter.
    }
    0x0F, 0x0C, 1, RO, EVC_FFR(evc_ffr) { /// Frame filter rejection counter
        value, 0,  7, u8; /// Frame Filter Rejection Event Counter.
    }
    0x0F, 0x0E, 1, RO, EVC_OVR (evc_ovr) { /// RX overrun error counter
        value, 0,  7, u8; /// RX Overrun Error Event Counter.
    }
    0x0F, 0x10, 2, RO, EVC_STO(evc_sto) { /// SFD timeout counter
        value, 0, 11, u16; /// SFD timeout errors Event Counter.
    }
    0x0F, 0x12, 2, RO, EVC_PTO(evc_pto) { /// Preamble timeout counter
        value, 0, 11, u16; /// Preamble  Detection  Timeout  Event  Counter.
    }
    0x0F, 0x14, 1, RO, EVC_FWTO(evc_fwto) { /// RX frame wait timeout counter
        value, 0, 7, u8; /// RX  Frame  Wait  Timeout  Event  Counter.
    }
    0x0F, 0x16, 2, RO, EVC_TXFS(evc_txfs) { /// TX frame sent counter
        value, 0, 11, u16; /// TX Frame Sent Event Counter.
    }
    0x0F, 0x18, 1, RO, EVC_HPW(evc_hpw) { /// Half period warning counter
        value, 0, 7, u8; /// Half Period Warning Event Counter.
    }
    0x0F, 0x1A, 1, RO, EVC_SWCE(evc_swce) { /// SPI write CRC error counter
        value, 0, 7, u8; /// SPI write CRC error counter.
    }
    0x0F, 0x1C, 8, RO, EVC_RES1(evc_res1) { /// Digital diagnostics reserved area 1
        value, 0, 63, u64; /// Digital diagnostics reserved area 1
    }
    0x0F, 0x24, 4, RW, DIAG_TMC(diag_tmc) { /// Test mode control register
        tx_pstm,    4,  4, u8; /// Transmit Power Spectrum Test Mode.
        hirq_pol,  21, 21, u8; /// Host interrupt polarity.
        cia_wden,  24, 24, u8; /// Enable the CIA watchdog.
        cia_run,   26, 26, u8; /// Run the CIA manually.
    }
    0x0F, 0x28, 1, RO, EVC_CPQE(evc_cpqe) { /// STS quality error counter
        value, 0, 7, u8; /// STS quality error counter
    }
    0x0F, 0x2A, 1, RO, EVC_VWARN(evc_vwarn) { /// Low voltage warning error counter
        value, 0, 7, u8; /// Low voltage warning error counter
    }
    0x0F, 0x2C, 1, RO, SPI_MODE(spi_mode) { /// SPI mode
        value, 0, 1, u8; /// SPI mode
    }
    0x0F, 0x30, 4, RO, SYS_STATE(sys_state) { /// System states *
        tx_state,    0,  3, u8; /// Current Transmit State Machine value
        rx_state,    8, 11, u8; /// Current Receive State Machine value
        pmsc_state, 16, 23, u8; /// Current PMSC State Machine value
    }
    0x0F, 0x3C, 1, RO, FCMD_STAT(fcmd_stat) { /// Fast command status
        value, 0, 4, u8; /// Fast command status.
    }
    0x0F, 0x48, 4, RO, CTR_DBG(ctr_dbg) { /// Current value of  the low 32-bits of the STS IV
        value, 0, 31, u32; /// Current value of  the low 32-bits of the STS IV
    }
    0x0F, 0x4C, 1, RO, SPICRCINIT(spicrcinit) { /// SPI CRC LFSR initialisation code
        value, 0, 7, u8; /// SPI CRC LFSR initialisation code for the SPI CRC function.
    }

    /*******************************************************************/
    /********************     PMSC REGISTER    *************************/
    /*******************************************************************/
    0x11, 0x00, 2, RW, SOFT_RST(soft_rst) { /// Soft reset of the device blocks
        arm_rst,  0, 0, u8; /// Soft ARM reset
        prgn_rst, 1, 1, u8; /// Soft PRGN reset
        cia_rst,  2, 2, u8; /// Soft CIA reset
        bist_rst, 3, 3, u8; /// Soft BIST reset
        rx_rst,   4, 4, u8; /// Soft RX reset
        tx_rst,   5, 5, u8; /// Soft TX reset
        hif_rst,  6, 6, u8; /// Soft HIF reset
        pmsc_rst, 7, 7, u8; /// Soft PMSC reset
        gpio_rst, 8, 8, u8; /// Soft GPIO reset
    }
    0x11, 0x04, 4, RW, CLK_CTRL(clk_ctrl) { /// PMSC clock control register
        sys_clk,       0,  1, u8; /// System Clock Selection field.
        rx_clk,        2,  3, u8; /// Receiver Clock Selection
        tx_clk,        4,  5, u8; /// Transmitter Clock Selection.
        acc_clk_en,    6,  6, u8; /// Force Accumulator Clock Enable
        cia_clk_en,    8,  8, u8; /// Force CIA Clock Enable
        sar_clk_en,   10, 10, u8; /// Analog-to-Digital Convertor Clock Enable.
        acc_mclk_en,  15, 15, u8; /// Accumulator Memory Clock Enable.
        gpio_clk_en,  16, 16, u8; /// GPIO clock Enable
        gpio_dclk_en, 18, 18, u8; /// GPIO De-bounce Clock Enable.
        gpio_drst_n,  19, 19, u8; /// GPIO de-bounce reset (NOT), active low.
        lp_clk_en,    23, 23, u8; /// Kilohertz clock Enable.
    }
    0x11, 0x08, 4, RW, SEQ_CTRL(seq_ctrl) { /// PMSC sequencing control register
        ainit2idle,    8,  8, u8; /// Automatic  IDLE_RC  to  IDLE_PLL.
        atx2slp,      11, 11, u8; /// After TX automatically Sleep.
        arx2slp,      12, 12, u8; /// After RX automatically Sleep.
        pll_sync,     15, 15, u8; /// This enables a 1 GHz clock used for some external SYNC modes.
        ciarune,      17, 17, u8; /// CIA run enable.
        force2init,   23, 23, u8; /// Force to IDLE_RC state.
        lp_clk_div,   26, 31, u8; /// Kilohertz clock divisor.
    }
    0x11, 0x12, 4, RW, TXFSEQ(txfseq) { /// PMSC fine grain TX sequencing control
        value, 0, 31, u32; /// PMSC fine grain TX sequencing control
    }
    0x11, 0x16, 4, RW, LED_CTRL(led_ctrl) { /// PMSC fine grain TX sequencing control
        blink_tim,   0,  7, u8; /// Blink time count value.
        blink_en,    8,  8, u8; /// Blink Enable.
        force_trig, 16, 19, u8; /// Manually triggers an LED blink.
    }
    0x11, 0x1A, 4, RW, RX_SNIFF(rx_sniff) { /// Receiver SNIFF mode configuration
        sniff_on,   0,  3, u8; /// SNIFF Mode ON time.
        sniff_off,  8, 15, u8; /// SNIFF Mode OFF time specified in μs.
    }
    0x11, 0x1F, 2, RW, BIAS_CTRL(bias_ctrl) { /// Analog blocks’ calibration values
        value, 0, 13, u16; /// Analog blocks’ calibration values
    }

    /*******************************************************************/
    /*****************     ACC_MEM REGISTER    *************************/
    /*******************************************************************/
    0x15, 0x00, 12288, RO, ACC_MEM(acc_mem) { /// Read access to accumulator data memory
    } // If the code doesn't run properly, reduce the length from 12288 to 8096

    /*******************************************************************/
    /*****************     SCRATCH_RAM REGISTER    *********************/
    /*******************************************************************/
    0x16, 0x00, 127, RW, SCRATCH_RAM(scratch_ram) { /// Scratch RAM memory buffer
    }

    /*******************************************************************/
    /*****************     AES_RAM REGISTER    *************************/
    /*******************************************************************/
    0x17, 0x00, 128, RW, AES_KEY_RAM(aes_key_ram) { /// storage for up to 8 x 128 bit AES KEYs
        aes_key1,   0x0,  0x7F, u128; /// 1st AES key
        aes_key2,  0x80,  0xFF, u128; /// 2nd AES key
        aes_key3, 0x100, 0x17F, u128; /// 3rd AES key
        aes_key4, 0x180, 0x1FF, u128; /// 4th AES key
        aes_key5, 0x200, 0x27F, u128; /// 5th AES key
        aes_key6, 0x280, 0x2FF, u128; /// 6th AES key
        aes_key7, 0x300, 0x37F, u128; /// 7th AES key
        aes_key8, 0x380, 0x3FF, u128; /// 8th AES key
    }

    /*******************************************************************/
    /*****************     SET_1, SET2 REGISTERS    ********************/
    /*******************************************************************/
    0x18, 0x00, 464, RO, DB_DIAG(db_diag) { /// Double buffer diagnostic register set
    }
    0x18, 0x00, 232, RO, DB_DIAG_SET1(db_diag_set1) { /// Double buffer diagnostic register set 1
    }
    0x18, 0xE8, 232, RO, DB_DIAG_SET2(db_diag_set2) { /// Double buffer diagnostic register set 2
    }

    /*******************************************************************/
    /*****************     INDIRECT_PTR_A REGISTER    ******************/
    /*******************************************************************/
    0x1D, 0x00, 1, RW, INDIRECT_PTR_A(indirect_ptr_a) { /// Indirect pointer A
        value, 0, 7, u8; /// Indirect pointer A
    }

    /*******************************************************************/
    /*****************     INDIRECT_PTR_B REGISTER    ******************/
    /*******************************************************************/
    0x1E, 0x00, 1, RW, INDIRECT_PTR_B(indirect_ptr_b) { /// Indirect pointer B
        value, 0, 7, u8; /// Indirect pointer B
    }

    /*******************************************************************/
    /*****************     IN_PTR_CFG REGISTER    **********************/
    /*******************************************************************/
    0x1F, 0x00, 1, RO, FINT_STAT(fint_stat) { /// Fast System Event Status Register
        txok,       0,  0,  u8; /// TXFRB or TXPRS or TXPHS or TXFRS.
        cca_fail,   1,  1,  u8; /// AAT or CCA_FAIL.
        rxtserr,    2,  2,  u8; /// CIAERR
        rxok,       3,  3,  u8; /// RXFR and CIADONE or RXFCG.
        rxerr,      4,  4,  u8; /// RXFCE or RXFSL or  RXPHE or  ARFE or  RXSTO or RXOVRR.
        rxto,       5,  5,  u8; /// RXFTO  or  RXPTO.
        sys_event,  6,  6,  u8; /// VT_DET or GPIOIRQ or RCINIT or SPIRDY.
        sys_panic,  7,  7,  u8; /// AES_ERR or CMD_ERR or SPI_UNF or SPI_OVF or SPIERR or PLL_HILO or VWARN.
    }
    0x1F, 0x04, 1, RW, PTR_ADDR_A(ptr_addr_a) { /// Base address of the register to be accessed through indirect pointer A
        ptra_base,  0,  4,  u8; /// Base address of the register to be accessed through indirect pointer A
    }
    0x1F, 0x08, 2, RW, PTR_OFFSET_A(ptr_offset_a) { /// Offset address of the register to be accessed through indirect pointer A
        ptra_ofs,   0, 14,  u16; /// Offset address of the register to be accessed through indirect pointer A
    }
    0x1F, 0x0C, 1, RW, PTR_ADDR_B(ptr_addr_b) { /// Base address of the register to be accessed through indirect pointer B
        ptrb_base,  0,  4,  u8; /// Base address of the register to be accessed through indirect pointer B
    }
    0x1F, 0x10, 2, RW, PTR_OFFSET_B(ptr_offset_b) { /// Offset address of the register to be accessed through indirect pointer B
        ptrb_ofs,   0, 14,  u16; /// Offset address of the register to be accessed through indirect pointer B
    }
}

/// Transmit Data Buffer
///
/// Currently only the first 127 bytes of the buffer are supported, which is
/// enough to support standard Standard IEEE 802.15.4 UWB frames.
#[allow(non_camel_case_types)]
pub struct TX_BUFFER;

impl Register for TX_BUFFER {
    const ID: u8 = 0x14;
    const LEN: usize = 127;
    const SUB_ID: u8 = 0x00;
}

impl Writable for TX_BUFFER {
    type Write = tx_buffer::W;

    fn write() -> Self::Write {
        tx_buffer::W([0; 127 + 2])
    }

    fn buffer(w: &mut Self::Write) -> &mut [u8] {
        &mut w.0
    }
}

impl<SPI, CS> DW3000<SPI, CS> {
    /// Transmit Data Buffer
    pub fn tx_buffer(&mut self) -> RegAccessor<TX_BUFFER, SPI, CS> {
        RegAccessor(self, PhantomData)
    }
}

/// Transmit Data Buffer
pub mod tx_buffer {

    const HEADER_LEN: usize = 2;
    const LEN: usize = 127;

    /// Used to write to the register
    pub struct W(pub(crate) [u8; LEN + HEADER_LEN]);

    impl W {
        /// Provides write access to the buffer contents
        pub fn data(&mut self) -> &mut [u8] {
            &mut self.0[HEADER_LEN..]
        }
    }
}

/// Receive Data Buffer 0
///
/// Currently only the first 127 bytes of the buffer are supported, which is
/// enough to support standard Standard IEEE 802.15.4 UWB frames.
#[allow(non_camel_case_types)]
pub struct RX_BUFFER_0;

impl Register for RX_BUFFER_0 {
    const ID: u8 = 0x12;
    const LEN: usize = 127;
    const SUB_ID: u8 = 0x00;
}

impl Readable for RX_BUFFER_0 {
    type Read = rx_buffer_0::R;

    fn read() -> Self::Read {
        rx_buffer_0::R([0; 127 + 2])
    }

    fn buffer(w: &mut Self::Read) -> &mut [u8] {
        &mut w.0
    }
}

impl<SPI, CS> DW3000<SPI, CS> {
    /// Receive Data Buffer
    pub fn rx_buffer_0(&mut self) -> RegAccessor<RX_BUFFER_0, SPI, CS> {
        RegAccessor(self, PhantomData)
    }
}

/// Receive Data Buffer
pub mod rx_buffer_0 {
    use core::fmt;

    const HEADER_LEN: usize = 2;
    const LEN: usize = 127;

    /// Used to read from the register
    pub struct R(pub(crate) [u8; HEADER_LEN + LEN]);

    impl R {
        /// Provides read access to the buffer contents
        pub fn data(&self) -> &[u8] {
            &self.0[HEADER_LEN..HEADER_LEN + LEN]
        }
    }

    impl fmt::Debug for R {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "0x")?;
            for i in (0..LEN).rev() {
                write!(f, "{:02x}", self.0[HEADER_LEN + i])?;
            }

            Ok(())
        }
    }
}

/// Receive Data Buffer 1
///
/// Currently only the first 127 bytes of the buffer are supported, which is
/// enough to support standard Standard IEEE 802.15.4 UWB frames.
#[allow(non_camel_case_types)]
pub struct RX_BUFFER_1;

impl Register for RX_BUFFER_1 {
    const ID: u8 = 0x13;
    const LEN: usize = 127;
    const SUB_ID: u8 = 0x00;
}

impl Readable for RX_BUFFER_1 {
    type Read = rx_buffer_1::R;

    fn read() -> Self::Read {
        rx_buffer_1::R([0; 127 + 2])
    }

    fn buffer(w: &mut Self::Read) -> &mut [u8] {
        &mut w.0
    }
}

impl<SPI, CS> DW3000<SPI, CS> {
    /// Receive Data Buffer1
    pub fn rx_buffer_1(&mut self) -> RegAccessor<RX_BUFFER_1, SPI, CS> {
        RegAccessor(self, PhantomData)
    }
}

/// Receive Data Buffer
pub mod rx_buffer_1 {
    use core::fmt;

    const HEADER_LEN: usize = 2;
    const LEN: usize = 127;

    /// Used to read from the register
    pub struct R(pub(crate) [u8; HEADER_LEN + LEN]);

    impl R {
        /// Provides read access to the buffer contents
        pub fn data(&self) -> &[u8] {
            &self.0[HEADER_LEN..HEADER_LEN + LEN]
        }
    }

    impl fmt::Debug for R {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "0x")?;
            for i in (0..LEN).rev() {
                write!(f, "{:02x}", self.0[HEADER_LEN + i])?;
            }

            Ok(())
        }
    }
}

/// Internal trait used by `impl_registers!`
trait FromBytes {
    fn from_bytes(bytes: &[u8]) -> Self;
}

/// Internal trait used by `impl_registers!`
trait ToBytes {
    type Bytes;

    fn to_bytes(self) -> Self::Bytes;
}

/// Internal macro used to implement `FromBytes`/`ToBytes`
macro_rules! impl_bytes {
    ($($ty:ty,)*) => {
        $(
            impl FromBytes for $ty {
                fn from_bytes(bytes: &[u8]) -> Self {
                    let mut val = 0;

                    for (i, &b) in bytes.iter().enumerate() {
                        val |= (b as $ty) << (i * 8);
                    }

                    val
                }
            }

            impl ToBytes for $ty {
                type Bytes = [u8; ::core::mem::size_of::<$ty>()];

                fn to_bytes(self) -> Self::Bytes {
                    let mut bytes = [0; ::core::mem::size_of::<$ty>()];

                    for (i, b) in bytes.iter_mut().enumerate() {
                        let shift = 8 * i;
                        let mask  = 0xff << shift;

                        *b = ((self & mask) >> shift) as u8;
                    }

                    bytes
                }
            }
        )*
    }
}

impl_bytes! {
    u8,
    u16,
    u32,
    u64,
    u128,
}
