//! Low-level interface to the DW1000
//!
//! This module implements a register-level interface to the DW1000. Users of
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
//! [filing an issue]: https://github.com/braun-robotics/rust-dw1000/issues/new


use core::{
    fmt,
    marker::PhantomData,
};

use embedded_hal::{
    blocking::spi,
    digital::v2::OutputPin,
};

use rtt_target::rprintln;
<<<<<<< HEAD

=======
>>>>>>> 83b8685d1063d9feaac753c839ca684e8096ee01

/// Entry point to the DW1000 driver's low-level API
///
/// Please consider using [hl::DW1000] instead.
///
/// [hl::DW1000]: ../hl/struct.DW1000.html
pub struct DW1000<SPI, CS> {
    spi        : SPI,
    chip_select: CS,
}

impl<SPI, CS> DW1000<SPI, CS> {
    /// Create a new instance of `DW1000`
    ///
    /// Requires the SPI peripheral and the chip select pin that are connected
    /// to the DW1000.
    pub fn new(spi: SPI, chip_select: CS) -> Self {
        DW1000 {
            spi,
            chip_select,
        }
    }
}


/// Provides access to a register
///
/// You can get an instance for a given register using one of the methods on
/// [`DW1000`].
pub struct RegAccessor<'s, R, SPI, CS>(&'s mut DW1000<SPI, CS>, PhantomData<R>);

impl<'s, R, SPI, CS> RegAccessor<'s, R, SPI, CS>
    where
        SPI: spi::Transfer<u8> + spi::Write<u8>,
        CS:  OutputPin,
{
    /// Read from the register
    pub fn read(&mut self)
        -> Result<R::Read, Error<SPI, CS>>
        where
            R: Register + Readable,
    {
        let mut r      = R::read();
        let mut buffer = R::buffer(&mut r);

        init_header::<R>(false, &mut buffer);
<<<<<<< HEAD

        rprintln!("{:?}", buffer);

=======
>>>>>>> 83b8685d1063d9feaac753c839ca684e8096ee01
        self.0.chip_select.set_low()
            .map_err(|err| Error::ChipSelect(err))?;
        self.0.spi.transfer(buffer)
            .map_err(|err| Error::Transfer(err))?;
        self.0.chip_select.set_high()
            .map_err(|err| Error::ChipSelect(err))?;

        Ok(r)
    }

    /// Write to the register
    pub fn write<F>(&mut self, f: F)
        -> Result<(), Error<SPI, CS>>
        where
            R: Register + Writable,
            F: FnOnce(&mut R::Write) -> &mut R::Write,
    {
        let mut w = R::write();
        f(&mut w);

        let buffer = R::buffer(&mut w);
        init_header::<R>(true, buffer);

        self.0.chip_select.set_low()
            .map_err(|err| Error::ChipSelect(err))?;
        <SPI as spi::Write<u8>>::write(&mut self.0.spi, buffer)
            .map_err(|err| Error::Write(err))?;
        self.0.chip_select.set_high()
            .map_err(|err| Error::ChipSelect(err))?;

        Ok(())
    }

    /// Modify the register
    pub fn modify<F>(&mut self, f: F)
        -> Result<(), Error<SPI, CS>>
        where
            R: Register + Readable + Writable,
            F: for<'r>
                FnOnce(&mut R::Read, &'r mut R::Write) -> &'r mut R::Write,
    {
        let mut r = self.read()?;
        let mut w = R::write();

        <R as Writable>::buffer(&mut w)
            .copy_from_slice(<R as Readable>::buffer(&mut r));

        f(&mut r, &mut w);

        let buffer = <R as Writable>::buffer(&mut w);
        init_header::<R>(true, buffer);

        self.0.chip_select.set_low()
            .map_err(|err| Error::ChipSelect(err))?;
        <SPI as spi::Write<u8>>::write(&mut self.0.spi, buffer)
            .map_err(|err| Error::Write(err))?;
        self.0.chip_select.set_high()
            .map_err(|err| Error::ChipSelect(err))?;

        Ok(())
    }
}


/// An SPI error that can occur when communicating with the DW1000
pub enum Error<SPI, CS>
    where
        SPI: spi::Transfer<u8> + spi::Write<u8>,
        CS:  OutputPin,
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
            Error::Transfer(error)   => write!(f, "Transfer({:?})", error),
            Error::Write(error)      => write!(f, "Write({:?})", error),
            Error::ChipSelect(error) => write!(f, "ChipSelect({:?})", error),
        }
    }
}


/// Initializes the SPI message header
///
/// Initializes the SPI message header for accessing a given register, writing
/// the header directly into the provided buffer. Returns the length of the
/// header that was written.
fn init_header<R: Register>(write: bool, buffer: &mut [u8]) -> usize {
    let sub_id = R::SUB_ID > 0;

    // bool write definit si on est en lecture ou e ecriture (premier bit)
    // sub_id est un bool qui definit si on est en full ou short command
    // on commmence par du full address !
    buffer[0] =
        (((write as u8)  << 7) & 0x80) |
        (((sub_id as u8) << 6) & 0x40) |
        ((R::ID          << 1)  & 0x3e) |
        (((R::SUB_ID as u8)) >> 6);

    if !sub_id {
        return 1;
    }

    buffer[1] = ((R::SUB_ID as u8)  << 2); 

    2
}


/// Implemented for all registers
///
/// This is a mostly internal crate that should not be implemented or used
/// directly by users of this crate. It is exposed through the public API
/// though, so it can't be made private.
///
/// The DW1000 user manual, section 7.1, specifies what the values of the
/// constant should be for each register.
pub trait Register {
    /// The register index
    const ID: u8;

    /// The registers's sub-index
    const SUB_ID: u16;

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
                const SUB_ID: u16   = $sub_id;
                const LEN:    usize = $len;
            }

            impl $name {
                // You know what would be neat? Using `if` in constant
                // expressions! But that's not possible, so we're left with the
                // following hack.
                const SUB_INDEX_IS_NONZERO: usize =
                    (Self::SUB_ID > 0) as usize;
                const SUB_INDEX_NEEDS_SECOND_BYTE: usize =
                    (Self::SUB_ID > 127) as usize;
                const HEADER_LEN: usize =
                    1
                    + Self::SUB_INDEX_IS_NONZERO
                    + Self::SUB_INDEX_NEEDS_SECOND_BYTE;
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

                            // The numer of bytes in the register data that
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
                                #[allow(exceeding_bitshifts)]
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
                                #[allow(exceeding_bitshifts)]
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


                /// Used to write to the register
                pub struct W(pub(crate) [u8; HEADER_LEN + $len]);

                impl W {
                    $(
                        #[$field_doc]
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


        impl<SPI, CS> DW1000<SPI, CS> {
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


/*************************************************************************/
/**********               DWM3000 MODIFICATIONS               ************/
/*************************************************************************/
// registers for DWM3000 
// Each field follows the following syntax:
// <Id>, <Offset>, <Length>, <Access>, <NAME(name)>
//      <name>, <first-bit-index>, <last-bit-index>, <type>; /// <doc>

impl_register! {
/*
    0x00, 0, 126, RO, GEN_CFG_AES(gen_cfg_aes) { /// Device identifier

    }

    0x00, 0, 126, RO, GEN_CFG_AES2(gen_cfg_aes2) { /// Device identifier

    }
*/
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
        value, 0, 7, u8; /// Comment
    }
    0x00, 0x1C, 4, RO, SYS_TIME(sys_time) { ///  System Time Counter register
        value, 0, 31, u32; /// Comment
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
    0x00, 0x30, 4, RW, DREF_TIME(dref_time) { /// commentaires
        value, 0, 31, u32; /// Delayed send or receive reference time
    }
    0x00, 0x4, 3, RW, RX_FWTO(rx_fwto) { /// commentaires
        value, 0, 31, u32; /// Receive frame wait timeout period
    }
    0x00, 0x38, 1, RW, SYS_CTRL(sys_ctrl) { /// System Control Register
        value, 0, 7, u8; /// System control
    }
    0x00, 0x3C, 6, RW, SYS_ENABLE(sys_enable) { /// A TESTER
        cplock_en,      1,  1, u8; /// C
        spicrce_en,     2,  2, u8; /// C
        aat_en,         3,  3, u8; /// C
        txfrb_en,       4,  4, u8; /// C
        txprs_en,       5,  5, u8; /// C
        txphs_en,       6,  6, u8; /// C
        txfrs_en,       7,  7, u8; /// C
        rxprd_en,       8,  8, u8; /// C
        rxsfdd_en,      9,  9, u8; /// C
        ciadone_en,    10,  10, u8; /// C
        rxphd_en,      11,  11, u8; /// C
        rxphe_en,      12,  12, u8; /// C
        rxfr_en,       13,  13, u8; /// C
        rxfcg_en,      14,  14, u8; /// C
        rxfce_en,      15,  15, u8; /// C
        rxrfsl_en,     16,  16, u8; /// C
        rxfto_en,      17,  17, u8; /// C
        ciaerr_en,     18,  18, u8; /// C
        vwarn_en,      19,  19, u8; /// C
        rxovrr_en,     20,  20, u8; /// C
        rxpto_en,      21,  21, u8; /// C
        spirdy_en,     23,  23, u8; /// C
        rcinit_en,     24,  24, u8; /// C
        pll_hilo_en,   25,  25, u8; /// C
        rxsto_en,      26,  26, u8; /// C
        hpdwarn_en,    27,  27, u8; /// C
        cperr_en,      28,  28, u8; /// C
        arfe_en,       29,  29, u8; /// C
        rxprej_en,     33,  33, u8; /// C
        vt_det_en,     36,  36, u8; /// C
        gpioirq_en,    37,  37, u8; /// C
        aes_done_en,   38,  38, u8; /// C
        aes_err_en,    39,  39, u8; /// C
        cdm_err_en,    40,  40, u8; /// C
        spi_ovf_en,    41,  41, u8; /// C
        spi_unf_en,    42,  42, u8; /// C
        spi_err_en,    43,  43, u8; /// C
        cca_fail_en,   44,  44, u8; /// C
    }
    0x00, 0x44, 6, RW, SYS_STATUS(sys_status) { /// System Event Status Register
        // A TESTER
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
        vwarn,     19, 19, u8; /// C
        rxovrr,    20, 20, u8; /// RX Overrun
        rxpto,     21, 21, u8; /// C
        spirdy,    23, 23, u8; /// C
        rcinit,    24, 24, u8; /// C
        pll_hilo,  25, 25, u8; /// C
        rxsto,     26, 26, u8; /// C
        hpdwarn,   27, 27, u8; /// C
        cperr,     28, 28, u8; /// C
        arfe,      29, 29, u8; /// C
        rxprej,    29, 29, u8; /// C
        vt_det,    33, 33, u8; /// C
        gpioirq,   36, 36, u8; /// C
        aes_done,  37, 37, u8; /// C
        aes_err,   38, 38, u8; /// C
        cmd_err,   39, 39, u8; /// C
        spi_ovf,   40, 40, u8; /// C
        spi_unf,   41, 41, u8; /// C
        spierr,    42, 42, u8; /// C
        cca_fail,  43, 43, u8; /// C
    }
    0x00, 0x4C, 4, RO, RX_FINFO(rx_finfo) { /// RX Frame Information
        // A TESTER
        rxflen,  0,  9, u16; /// Receive Frame Length
        rxnspl, 11, 12, u8; /// Receive Non-Standard Preamble Length
        rxbr,   13, 13, u8; /// Receive Bit Rate Report
        rng,    15, 15, u8; /// Receiver Ranging
        rxprf,  16, 17, u8; /// RX Pulse Repetition Rate Report
        rxpsr,  18, 19, u8; /// RX Preamble Repetition
        rxpacc, 20, 31, u16; /// Preamble Accumulation Count        
    }
    0x00, 0x64, 16, RO, RX_TIME(rx_time) { /// Receive Time Stamp
        // A TESTER
        rx_stamp,  0,  39, u64; /// Fully adjusted time stamp
        rx_rawst, 64, 95, u64; /// Raw time stamp
    }
    0x00, 0x74, 5, RO, TX_TIME(tx_time) { /// Transmit Time Stamp
        // A TESTER
        tx_stamp,  0, 39, u64; /// Fully adjusted time stamp
    }
    0x01, 0x00, 4, RO, TX_RAWST(tx_rawst) { /// Transmit time stamp raw
        // A TESTER
        value, 0, 31, u32; /// C
    }
    0x01, 0x04, 2, RW, TX_ANTD(tx_antd) { /// Transmitter antenna delay
        // A TESTER
        value, 0, 15, u16; /// C
    }
    0x01, 0x08, 4, RW, ACK_RESP(ack_resp) { /// Acknowledgement delay time and response time
        // A TESTER
        w4r_tim,  0, 19, u32; /// C
        ack_tim,  24, 31, u8; /// C
    }
    0x01, 0x0C, 4, RW, TX_POWER(tx_power) { /// TX Power Control
        // A TESTER
        // The TX_POWER register has multiple sets of fields defined, depending
        // on the smart TX power control setting. I don't know how to model
        // this, so I've opted to provide just a single `value` field for
        // maximum flexibility.
        value, 0, 31, u32; /// TX Power Control value
    }
    0x01, 0x14, 2, RW, CHAN_CTRL(chan_ctrl) { /// Channel Control Register
        // A TESTER
        rf_chan,   0, 0, u8; /// Selects the receive channel.
        sfd_type,  1, 2, u8; /// Enables the non-standard Decawave proprietary SFD sequence.
        tx_pcode,  3, 7, u8; /// This field selects the preamble code used in the transmitter.
        rx_pcode,  8, 12, u8; /// This field selects the preamble code used in the receiver.
    }
    0x01, 0x18, 4, RW, LE_PEND_01(le_pend_01) { /// C
        // A TESTER
        le_addr0,  0, 15, u16; /// CC
        le_addr1, 16, 31, u16; /// CC
    }
    0x01, 0x1C, 4, RW, LE_PEND_23(le_pend_23) { /// C
        // A TESTER
        le_addr2,  0, 15, u16; /// CC
        le_addr3, 16, 31, u16; /// CC
    }
    0x01, 0x20, 1, RW, SPI_COLLISION(spi_collision) { /// C
        // A TESTER
        value,  0, 7, u8; /// Fully adjusted time stamp
    }
    0x01, 0x24, 1, RW, RDB_STATUS(rdb_status) { /// RX double buffer status
        // A TESTER
        rxfcg0,     0, 0, u8; /// CC
        rxfr0,      1, 1, u8; /// CC
        ciadone0,   2, 2, u8; /// CC
        cp_err0,    3, 3, u8; /// CC
        rxfcg1,     4, 4, u8; /// CC
        rxfr1,      5, 5, u8; /// CC
        ciadone1,   6, 6, u8; /// CC
        cp_err1,    7, 7, u8; /// CC
    }
    0x01, 0x28, 1, RW, RDB_DIAG(rdb_diag) { /// C
        // A TESTER
        rdb_dmode,    0, 2, u8; /// CC
    }
    0x01, 0x30, 2, RW, AES_CFG(aes_cfg) { /// C
        // A TESTER
        mode,        0, 0, u8; /// CC
        key_size,    1, 2, u8; /// CC
        key_addr,    3, 5, u8; /// CC
        key_load,    6, 6, u8; /// CC
        key_src,     7, 7, u8; /// CC
        tag_size,    8, 10, u8; /// CC
        core_sel,    11, 11, u8; /// CC
        key_otp,     12, 12, u8; /// CC
    }
    0x01, 0x34, 4, RW, AES_IV0(aes_iv0) { /// C
        // A TESTER
        value,  0, 31, u32; /// CC
    }
    0x01, 0x38, 4, RW, AES_IV1(aes_iv1) { /// C
        // A TESTER
        value,  0, 31, u32; /// CC
    }
    0x01, 0x3C, 4, RW, AES_IV2(aes_iv2) { /// C
        // A TESTER
        value,  0, 31, u32; /// CC
    }
    0x01, 0x40, 2, RW, AES_IV3(aes_iv3) { /// C
        // A TESTER
        value,  0, 15, u16; /// CC
    }
    0x01, 0x42, 2, RW, AES_IV4(aes_iv4) { /// C
        // A TESTER
        value,  0, 15, u16; /// CC
    }
    0x01, 0x44, 8, RW, DMA_CFG(dma_cfg) { /// C
        // A TESTER
        src_port,   0, 2, u8; /// CC
        src_addr,   3, 12, u16; /// CC
        dst_port,   13, 15, u8; /// CC
        dst_addr,   16, 25, u16; /// CC
        cp_end_sel, 26, 26, u8; /// CC
        hdr_size,   32, 38, u8; /// CC
        pyld_size,  39, 48, u8; /// CC
    }
    0x01, 0x4C, 1, RW, AES_START(aes_start) { /// C
        // A TESTER
        value,  0, 0, u8; /// CC
    }
    0x01, 0x50, 4, RW, AES_STS(aes_sts) { /// C
        // A TESTER
        aes_done,  0, 0, u8; /// CC
        auth_err,  1, 1, u8; /// CC
        trans_err,  2, 2, u8; /// CC
        mem_conf,  3, 3, u8; /// CC
        ram_empty,  4, 4, u8; /// CC
        ram_full,  5, 5, u8; /// CC
    }
    0x01, 0x54, 16, RW, AES_KEY(aes_key) { /// C
        // A FINIR
        //value,  0, 127, u128; /// CC
    }

    // STS_CFG


    0x19, 0x00, 5, RO, SYS_STATE(sys_state) { /// System State information
        tx_state,    0,  3, u8; /// Current Transmit State Machine value
        rx_state,    8, 12, u8; /// Current Receive State Machine value
        pmsc_state, 16, 23, u8; /// Current PMSC State Machine value
    }
    
    0x21, 0x00, 1, RW, SFD_LENGTH(sfd_length) { /// This is the length of the SFD sequence used when the data rate is 850kbps and higher.
        value, 0, 7, u8; /// This is the length of the SFD sequence used when the data rate is 850kbps and higher.
    }
    0x23, 0x04, 2, RW, AGC_TUNE1(agc_tune1) { /// AGC Tuning register 1
        value, 0, 15, u16; /// AGC Tuning register 1 value
    }
    0x23, 0x0C, 4, RW, AGC_TUNE2(agc_tune2) { /// AGC Tuning register 2
        value, 0, 31, u32; /// AGC Tuning register 2 value
    }
    0x24, 0x00, 4, RW, EC_CTRL(ec_ctrl) { /// External Clock Sync Counter Config
        ostsm,   0,  0, u8; /// External Transmit Synchronization Mode Enable
        osrsm,   1,  1, u8; /// External Receive Synchronization Mode Enable
        pllldt,  2,  2, u8; /// Clock PLL Lock Detect Tune
        wait,    3, 10, u8; /// Wait Counter
        ostrm,  11, 11, u8; /// External Timebase Reset Mode Enable
    }
    0x26, 0x00, 4, RW, GPIO_MODE(gpio_mode) { /// GPIO Mode Control Register
        msgp0,  6,  7, u8; /// Mode Selection for GPIO0/RXOKLED
        msgp1,  8,  9, u8; /// Mode Selection for GPIO1/SFDLED
        msgp2, 10, 11, u8; /// Mode Selection for GPIO2/RXLED
        msgp3, 12, 13, u8; /// Mode Selection for GPIO3/TXLED
        msgp4, 14, 15, u8; /// Mode Selection for GPIO4/EXTPA
        msgp5, 16, 17, u8; /// Mode Selection for GPIO5/EXTTXE
        msgp6, 18, 19, u8; /// Mode Selection for GPIO6/EXTRXE
        msgp7, 20, 21, u8; /// Mode Selection for SYNC/GPIO7
        msgp8, 22, 23, u8; /// Mode Selection for IRQ/GPIO8
    }
    0x26, 0x08, 4, RW, GPIO_DIR(gpio_dir) { /// GPIO Direction Control Register
        gdp0,  0,  0, u8; /// Direction Selection for GPIO0
        gdp1,  1,  1, u8; /// Direction Selection for GPIO1
        gdp2,  2,  2, u8; /// Direction Selection for GPIO2
        gdp3,  3,  3, u8; /// Direction Selection for GPIO3
        gdm0,  4,  4, u8; /// Mask for setting the direction of GPIO0
        gdm1,  5,  5, u8; /// Mask for setting the direction of GPIO1
        gdm2,  6,  6, u8; /// Mask for setting the direction of GPIO2
        gdm3,  7,  7, u8; /// Mask for setting the direction of GPIO3
        gdp4,  8,  8, u8; /// Direction Selection for GPIO4
        gdp5,  9,  9, u8; /// Direction Selection for GPIO5
        gdp6, 10, 10, u8; /// Direction Selection for GPIO6
        gdp7, 11, 11, u8; /// Direction Selection for GPIO7
        gdm4, 12, 12, u8; /// Mask for setting the direction of GPIO4
        gdm5, 13, 13, u8; /// Mask for setting the direction of GPIO5
        gdm6, 14, 14, u8; /// Mask for setting the direction of GPIO6
        gdm7, 15, 15, u8; /// Mask for setting the direction of GPIO7
        gdp8, 16, 16, u8; /// Direction Selection for GPIO8
        gdm8, 20, 20, u8; /// Mask for setting the direction of GPIO8
    }
    0x26, 0x0C, 4, RW, GPIO_DOUT(gpio_dout) { /// GPIO Data Output register
        gop0,  0,  0, u8; /// Output state setting for GPIO0
        gop1,  1,  1, u8; /// Output state setting for GPIO1
        gop2,  2,  2, u8; /// Output state setting for GPIO2
        gop3,  3,  3, u8; /// Output state setting for GPIO3
        gom0,  4,  4, u8; /// Mask for setting the output state of GPIO0
        gom1,  5,  5, u8; /// Mask for setting the output state of GPIO1
        gom2,  6,  6, u8; /// Mask for setting the output state of GPIO2
        gom3,  7,  7, u8; /// Mask for setting the output state of GPIO3
        gop4,  8,  8, u8; /// Output state setting for GPIO4
        gop5,  9,  9, u8; /// Output state setting for GPIO5
        gop6, 10, 10, u8; /// Output state setting for GPIO6
        gop7, 11, 11, u8; /// Output state setting for GPIO7
        gom4, 12, 12, u8; /// Mask for setting the output state of GPIO4
        gom5, 13, 13, u8; /// Mask for setting the output state of GPIO5
        gom6, 14, 14, u8; /// Mask for setting the output state of GPIO6
        gom7, 15, 15, u8; /// Mask for setting the output state of GPIO7
        gop8, 16, 16, u8; /// Output state setting for GPIO8
        gom8, 20, 20, u8; /// Mask for setting the output state of GPIO8
    }
    0x26, 0x10, 4, RW, GPIO_IRQE(gpio_irqe) { /// GPIO Interrupt Enable
        girqe0,  0,  0, u8; /// GPIO IRQ Enable for GPIO0 input
        girqe1,  1,  1, u8; /// GPIO IRQ Enable for GPIO1 input
        girqe2,  2,  2, u8; /// GPIO IRQ Enable for GPIO2 input
        girqe3,  3,  3, u8; /// GPIO IRQ Enable for GPIO3 input
        girqe4,  4,  4, u8; /// GPIO IRQ Enable for GPIO4 input
        girqe5,  5,  5, u8; /// GPIO IRQ Enable for GPIO5 input
        girqe6,  6,  6, u8; /// GPIO IRQ Enable for GPIO6 input
        girqe7,  7,  7, u8; /// GPIO IRQ Enable for GPIO7 input
        girqe8,  8,  8, u8; /// GPIO IRQ Enable for GPIO8 input
    }
    0x26, 0x14, 4, RW, GPIO_ISEN(gpio_isen) { /// GPIO Interrupt Sense Selection
        gisen0,  0,  0, u8; /// GPIO IRQ sense for GPIO0 input
        gisen1,  1,  1, u8; /// GPIO IRQ sense for GPIO1 input
        gisen2,  2,  2, u8; /// GPIO IRQ sense for GPIO2 input
        gisen3,  3,  3, u8; /// GPIO IRQ sense for GPIO3 input
        gisen4,  4,  4, u8; /// GPIO IRQ sense for GPIO4 input
        gisen5,  5,  5, u8; /// GPIO IRQ sense for GPIO5 input
        gisen6,  6,  6, u8; /// GPIO IRQ sense for GPIO6 input
        gisen7,  7,  7, u8; /// GPIO IRQ sense for GPIO7 input
        gisen8,  8,  8, u8; /// GPIO IRQ sense for GPIO8 input
    }
    0x26, 0x18, 4, RW, GPIO_IMODE(gpio_imode) { /// GPIO Interrupt Mode (Level / Edge)
        gimod0,  0,  0, u8; /// GPIO IRQ mode selection for GPIO0 input
        gimod1,  1,  1, u8; /// GPIO IRQ mode selection for GPIO1 input
        gimod2,  2,  2, u8; /// GPIO IRQ mode selection for GPIO2 input
        gimod3,  3,  3, u8; /// GPIO IRQ mode selection for GPIO3 input
        gimod4,  4,  4, u8; /// GPIO IRQ mode selection for GPIO4 input
        gimod5,  5,  5, u8; /// GPIO IRQ mode selection for GPIO5 input
        gimod6,  6,  6, u8; /// GPIO IRQ mode selection for GPIO6 input
        gimod7,  7,  7, u8; /// GPIO IRQ mode selection for GPIO7 input
        gimod8,  8,  8, u8; /// GPIO IRQ mode selection for GPIO8 input
    }
    0x26, 0x1C, 4, RW, GPIO_IBES(gpio_ibes) { /// GPIO Interrupt “Both Edge” Select
        gibes0,  0,  0, u8; /// GPIO IRQ "Both Edges" selection for GPIO0 input
        gibes1,  1,  1, u8; /// GPIO IRQ "Both Edges" selection for GPIO1 input
        gibes2,  2,  2, u8; /// GPIO IRQ "Both Edges" selection for GPIO2 input
        gibes3,  3,  3, u8; /// GPIO IRQ "Both Edges" selection for GPIO3 input
        gibes4,  4,  4, u8; /// GPIO IRQ "Both Edges" selection for GPIO4 input
        gibes5,  5,  5, u8; /// GPIO IRQ "Both Edges" selection for GPIO5 input
        gibes6,  6,  6, u8; /// GPIO IRQ "Both Edges" selection for GPIO6 input
        gibes7,  7,  7, u8; /// GPIO IRQ "Both Edges" selection for GPIO7 input
        gibes8,  8,  8, u8; /// GPIO IRQ "Both Edges" selection for GPIO8 input
    }
    0x26, 0x20, 4, RW, GPIO_ICLR(gpio_iclr) { /// GPIO Interrupt Latch Clear
        giclr0,  0,  0, u8; /// GPIO IRQ latch clear for GPIO0 input
        giclr1,  1,  1, u8; /// GPIO IRQ latch clear for GPIO1 input
        giclr2,  2,  2, u8; /// GPIO IRQ latch clear for GPIO2 input
        giclr3,  3,  3, u8; /// GPIO IRQ latch clear for GPIO3 input
        giclr4,  4,  4, u8; /// GPIO IRQ latch clear for GPIO4 input
        giclr5,  5,  5, u8; /// GPIO IRQ latch clear for GPIO5 input
        giclr6,  6,  6, u8; /// GPIO IRQ latch clear for GPIO6 input
        giclr7,  7,  7, u8; /// GPIO IRQ latch clear for GPIO7 input
        giclr8,  8,  8, u8; /// GPIO IRQ latch clear for GPIO8 input
    }
    0x26, 0x24, 4, RW, GPIO_IDBE(gpio_idbe) { /// GPIO Interrupt De-bounce Enable
        gidbe0,  0,  0, u8; /// GPIO IRQ de-bounce enable for GPIO0
        gidbe1,  1,  1, u8; /// GPIO IRQ de-bounce enable for GPIO1
        gidbe2,  2,  2, u8; /// GPIO IRQ de-bounce enable for GPIO2
        gidbe3,  3,  3, u8; /// GPIO IRQ de-bounce enable for GPIO3
        gidbe4,  4,  4, u8; /// GPIO IRQ de-bounce enable for GPIO4
        gidbe5,  5,  5, u8; /// GPIO IRQ de-bounce enable for GPIO5
        gidbe6,  6,  6, u8; /// GPIO IRQ de-bounce enable for GPIO6
        gidbe7,  7,  7, u8; /// GPIO IRQ de-bounce enable for GPIO7
        gidbe8,  8,  8, u8; /// GPIO IRQ de-bounce enable for GPIO8
    }
    0x26, 0x28, 4, RW, GPIO_RAW(gpio_raw) { /// GPIO raw state
        grawp0,  0,  0, u8; /// GPIO0 port raw state
        grawp1,  1,  1, u8; /// GPIO1 port raw state
        grawp2,  2,  2, u8; /// GPIO2 port raw state
        grawp3,  3,  3, u8; /// GPIO3 port raw state
        grawp4,  4,  4, u8; /// GPIO4 port raw state
        grawp5,  5,  5, u8; /// GPIO5 port raw state
        grawp6,  6,  6, u8; /// GPIO6 port raw state
        grawp7,  7,  7, u8; /// GPIO7 port raw state
        grawp8,  8,  8, u8; /// GPIO8 port raw state
    }
    0x27, 0x02, 2, RW, DRX_TUNE0B(drx_tune0b) { /// Digital Tuning Register 0b
        value, 0, 15, u16; /// DRX_TUNE0B tuning value
    }
    0x27, 0x04, 2, RW, DRX_TUNE1A(drx_tune1a) { /// Digital Tuning Register 1a
        value, 0, 15, u16; /// DRX_TUNE1A tuning value
    }
    0x27, 0x06, 2, RW, DRX_TUNE1B(drx_tune1b) { /// Digital Tuning Register 1b
        value, 0, 15, u16; /// DRX_TUNE1B tuning value
    }
    0x27, 0x08, 4, RW, DRX_TUNE2(drx_tune2) { /// Digital Tuning Register 2
        value, 0, 31, u32; /// DRX_TUNE2 tuning value
    }
    0x27, 0x20, 2, RW, DRX_SFDTOC(drx_sfdtoc) { /// SFD timeout
        count, 0, 15, u16; /// SFD detection timeout count
    }
    0x27, 0x24, 2, RW, DRX_PRETOC(drx_pretoc) { /// Preamble detection timeou
        count, 0, 15, u16; /// Preamble detection timeout count
    }
    0x27, 0x26, 2, RW, DRX_TUNE4H(drx_tune4h) { /// Digital Tuning Register 4h
        value, 0, 15, u16; /// DRX_TUNE4H tuning value
    }
    0x27, 0x28, 2, RO, DRX_CAR_INT(dxr_car_int) { /// Carrier Recovery Integrator Register
        value, 0, 15, u16; /// value
    }
    0x27, 0x2C, 2, RO, RXPACC_NOSAT(rxpacc_nosat) { /// Digital debug register. Unsaturated accumulated preamble symbols.
        value, 0, 15, u16; /// value
    }
    0x28, 0x0B, 1, RW, RF_RXCTRLH(rf_rxctrlh) { /// Analog RX Control Register
        value, 0, 7, u8; /// Analog RX Control Register
    }
    0x28, 0x0C, 3, RW, RF_TXCTRL(rf_txctrl) { /// Analog TX Control Register
        txmtune, 5,  8, u8; /// Transmit mixer tuning register
        txmq,    9, 11, u8; /// Transmit mixer Q-factor tuning register
        value, 0, 23, u32; /// The entire register
    }
    0x28, 0x30, 5, RW, LDOTUNE(ldotune) { /// LDO voltage tuning parameter
        value, 0, 39, u64; /// Internal LDO voltage tuning parameter
    }
    0x2A, 0x0B, 1, RW, TC_PGDELAY(tc_pgdelay) { /// Pulse Generator Delay
        value, 0, 7, u8; /// Transmitter Calibration - Pulse Generator Delay
    }
    0x2B, 0x07, 4, RW, FS_PLLCFG(fs_pllcfg) { /// Frequency synth - PLL configuration
        value, 0, 31, u32; /// Frequency synth - PLL configuration
    }
    0x2B, 0x0B, 1, RW, FS_PLLTUNE(fs_plltune) { /// Frequency synth - PLL Tuning
        value, 0, 7, u8; /// Frequency synthesiser - PLL Tuning
    }
    0x2D, 0x04, 2, RW, OTP_ADDR(otp_addr) { /// OTP Address
        value, 0, 10, u16; /// OTP Address
    }
    0x2D, 0x06, 2, RW, OTP_CTRL(otp_ctrl) { /// OTP Control
        otprden,  0,  0, u8; /// Forces OTP into manual read mode
        otpread,  1,  1, u8; /// Commands a read operation
        otpmrwr,  3,  3, u8; /// OTP mode register write
        otpprog,  6,  6, u8; /// Write OTP_WDAT to OTP_ADDR
        otpmr,    7, 10, u8; /// OTP mode register
        ldeload, 15, 15, u8; /// Force load of LDE microcode
    }
    0x2D, 0x0A, 4, RO, OTP_RDAT(otp_rdat) { /// OTP Read Data
        value, 0, 31, u32; /// OTP Read Data
    }
    0x2E, 0x0806, 1, RW, LDE_CFG1(lde_cfg1) { /// LDE Configuration Register 1
        ntm,   0, 4, u8; /// Noise Threshold Multiplier
        pmult, 5, 7, u8; /// Peak Multiplier
    }
    0x2E, 0x1804, 2, RW, LDE_RXANTD(lde_rxantd) { /// RX Antenna Delay
        value, 0, 15, u16; /// RX Antenna Delay
    }
    0x2E, 0x1806, 2, RW, LDE_CFG2(lde_cfg2) { /// LDE Configuration Register 2
        value, 0, 15, u16; /// The LDE_CFG2 configuration value
    }
    0x2F, 0x00, 4, RW, EVC_CTRL(evc_ctrl) { /// Event Counter Control
        evc_en,  0, 0, u8; /// Event Counters Enable
        evc_clr, 1, 1, u8; /// Event Counters Clear
    }
    0x2F, 0x18, 2, RO, EVC_HPW(evc_hpw) { /// Half Period Warning Counter
        value, 0, 11, u16; /// Half Period Warning Event Counter
    }
    0x2F, 0x1A, 2, RO, EVC_TPW(evc_tpw) { /// TX Power-Up Warning Counter
        value, 0, 11, u16; /// TX Power-Up Warning Event Counter
    }
    0x36, 0x00, 4, RW, PMSC_CTRL0(pmsc_ctrl0) { /// PMSC Control Register 0
        sysclks,    0,  1, u8; /// System Clock Selection
        rxclks,     2,  3, u8; /// Receiver Clock Selection
        txclks,     4,  5, u8; /// Transmitter Clock Selection
        face,       6,  6, u8; /// Force Accumulator Clock Enable
        adcce,     10, 10, u8; /// ADC Clock Enable
        amce,      15, 15, u8; /// Accumulator Memory Clock Enable
        gpce,      16, 16, u8; /// GPIO Clock Enable
        gprn,      17, 17, u8; /// GPIO Reset (Not), active low
        gpdce,     18, 18, u8; /// GPIO De-bounce Clock Enable
        gpdrn,     19, 19, u8; /// GPIO De-bounce Reset (Not), active low
        khzclken,  23, 23, u8; /// Kilohertz Clock Enable
        softreset, 28, 31, u8; /// Soft Reset
    }
    0x36, 0x04, 4, RW, PMSC_CTRL1(pmsc_ctrl1) { /// PMSC Control Register 1
        arx2init,   1,  1, u8; /// Automatic transition from receive to init
        pktseq,     3, 10, u8; /// Control PMSC control of analog RF subsystem
        atxslp,    11, 11, u8; /// After TX automatically sleep
        arxslp,    12, 12, u8; /// After RX automatically sleep
        snoze,     13, 13, u8; /// Snooze Enable
        snozr,     14, 14, u8; /// Snooze Repeat
        pllsyn,    15, 15, u8; /// Enable clock used for external sync modes
        lderune,   17, 17, u8; /// LDE Run Enable
        khzclkdiv, 26, 31, u8; /// Kilohertz Clock Divisor
    }
    0x36, 0x28, 4, RW, PMSC_LEDC(pmsc_ledc) { /// PMSC LED Control Register
        blink_tim, 0, 7, u8; /// Blink time count value
        blnken, 8, 8, u8; /// Blink Enable
        blnknow, 16, 19, u8; /// Manually triggers an LED blink. There is one trigger bit per LED IO
    }*/
    0x0F, 0x00, 79, RO, DIG_DIAG(dig_dial) { /// Digital diagnostics interface 
    }
    0x0F, 0x00, 1, RW, EVC_CTRL(evc_ctrl) { /// Event counter control 
        evc_en,  0, 0, u8; /// Event Counters Enable.  
        evc_clr, 1, 1, u8; /// Event Counters Clear.   
    }
    0x0F, 0x04, 2, RO, EVC_PHE(evc_phe) { /// PHR error counter
        evc_phe, 0, 11, u16; /// PHR Error Event Counter.  
    }
    0x0F, 0x06, 2, RO, EVC_RSE(evc_rse) { /// RSD error counter 
        evc_rse, 0, 11, u16; /// Reed Solomon decoder (Sync Loss) Error Event Counter.   
    }
    0x0F, 0x08, 2, RO, EVC_FCG(evc_fcg) { /// Frame check sequence good counter
        evc_fcg, 0, 11, u16; /// Frame Check Sequence Good Event Counter.  
    }
    0x0F, 0x08, 2, RO, EVC_FCE(evc_fce) { /// Frame Check Sequence error counter 
        evc_fce, 0, 11, u16; /// Frame Check Sequence Error Event Counter. 
    }
    0x0F, 0x0C, 1, RO, EVC_FFR(evc_ffr) { /// Frame filter rejection counter 
        evc_ffr, 0,  7, u8; /// Frame Filter Rejection Event Counter.  
    }
    0x0F, 0x0E, 1, RO, EVC_OVR (evc_ovr) { /// RX overrun error counter   
        evc_ovr, 0,  7, u8; /// RX Overrun Error Event Counter. 
    }
    0x0F, 0x10, 2, RO, EVC_STO(evc_sto) { /// SFD timeout counter 
        evc_sto, 0, 11, u16; /// SFD timeout errors Event Counter.  
    }
    0x0F, 0x12, 2, RO, EVC_PTO(evc_pto) { /// Preamble timeout counter 
        evc_pto, 0, 11, u16; /// Preamble  Detection  Timeout  Event  Counter.    
    }
    0x0F, 0x14, 1, RO, EVC_FWTO(evc_fwto) { /// RX frame wait timeout counter 
        evc_fwto, 0, 7, u8; /// RX  Frame  Wait  Timeout  Event  Counter.   
    }
    0x0F, 0x16, 2, RO, EVC_TXFS(evc_txfs) { /// TX frame sent counter 
        evc_txfs, 0, 11, u16; /// TX Frame Sent Event Counter. 
    }
    0x0F, 0x18, 1, RO, EVC_HPW(evc_hpw) { /// Half period warning counter 
        evc_hpw, 0, 7, u8; ///
    }/*
    0x0F, 0x1A, 79, RO, DIG_DIAG(dig_dial) { /// SPI write CRC error counter 
    }
    0x0F, 0x1C, 79, RO, DIG_DIAG(dig_dial) { /// Digital diagnostics reserved area 1  
    }
    0x0F, 0x24, 79, RO, DIG_DIAG(dig_dial) { /// Test mode control register 
    }
    0x0F, 0x28, 79, RO, DIG_DIAG(dig_dial) { /// STS quality error counter 
    }
    0x0F, 0x2A, 79, RO, DIG_DIAG(dig_dial) { /// Low voltage warning error counter 
    }
    0x0F, 0x2C, 79, RO, DIG_DIAG(dig_dial) { /// SPI mode 
    }
    0x0F, 0x30, 79, RO, DIG_DIAG(dig_dial) { /// System state 
    }
    0x0F, 0x3C, 79, RO, DIG_DIAG(dig_dial) { /// Fast command status 
    }
    0x0F, 0x48, 79, RO, DIG_DIAG(dig_dial) { /// Current value of  the low 32-bits of the STS IV 
    }
    0x0F, 0x4C, 79, RO, DIG_DIAG(dig_dial) { /// SPI CRC LFSR initialisation code
    }*/
    0x11, 0x00, 24, RO, PMSC_CTRL(pmsc_ctrl) { /// Power management, timing and seq control
    }
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
        txfineseq, 0, 31, u32; /// PMSC fine grain TX sequencing control
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
        bias_ctrl, 0, 13, u16; /// Analog blocks’ calibration values
    }
    0x15, 0x00, 12288, RO, ACC_MEM(acc_mem) { /// Read access to accumulator data memory
    } // If the code doesn't run properly, reduce the length from 12288 to 8096
    0x16, 0x00, 127, RW, SCRATCH_RAM(scratch_ram) { /// Scratch RAM memory buffer
    }
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
    0x18, 0x00, 464, RO, DB_DIAG(db_diag) { /// Double buffer diagnostic register set
    }
    0x18, 0x00, 232, RO, DB_DIAG_SET1(db_diag_set1) { /// Double buffer diagnostic register set 1
    }
    0x18, 0xE8, 232, RO, DB_DIAG_SET2(db_diag_set2) { /// Double buffer diagnostic register set 2
    }
    0x1D, 0x00, 1, RW, INDIRECT_PTR_A(indirect_ptr_a) { /// Indirect pointer A 
    }
    0x1E, 0x00, 1, RW, INDIRECT_PTR_B(indirect_ptr_b) { /// Indirect pointer B 
    }
    0x1F, 0x00, 19, RO, IN_PTR_CFG(in_ptr_cfg) { /// Indirect pointer configuration and fast interrupt status register
    }
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

/*
// Each field follows the following syntax:
// <name>, <first-bit-index>, <last-bit-index>, <type>; /// <doc>
impl_register! {
    0x00, 0x00, 4, RO, DEV_ID(dev_id) { /// Device identifier
        rev,     0,  3, u8;  /// Revision
        ver,     4,  7, u8;  /// Version
        model,   8, 15, u8;  /// Model
        ridtag, 16, 31, u16; /// Register Identification Tag
    }
    0x01, 0x00, 8, RW, EUI(eui) { /// Extended Unique Identifier
        value, 0, 63, u64; /// Extended Unique Identifier
    }
    0x03, 0x00, 4, RW, PANADR(panadr) { /// PAN Identifier and Short Address
        short_addr,  0, 15, u16; /// Short Address
        pan_id,     16, 31, u16; /// PAN Identifier
    }
    0x04, 0x00, 4, RW, SYS_CFG(sys_cfg) { /// System Configuration
        ffen,        0,  0, u8; /// Frame Filtering Enable
        ffbc,        1,  1, u8; /// Frame Filtering Behave As Coordinator
        ffab,        2,  2, u8; /// Frame Filtering Allow Beacon
        ffad,        3,  3, u8; /// Frame Filtering Allow Data
        ffaa,        4,  4, u8; /// Frame Filtering Allow Acknowledgement
        ffam,        5,  5, u8; /// Frame Filtering Allow MAC Command Frame
        ffar,        6,  6, u8; /// Frame Filtering Allow Reserved
        ffa4,        7,  7, u8; /// Frame Filtering Allow Frame Type 4
        ffa5,        8,  8, u8; /// Frame Filtering Allow Frame Type 5
        hirq_pol,    9,  9, u8; /// Host Interrupt Polarity
        spi_edge,   10, 10, u8; /// SPI Data Launch Edge
        dis_fce,    11, 11, u8; /// Disable Frame Check Error Handling
        dis_drxb,   12, 12, u8; /// Disable Double RX Buffer
        dis_phe,    13, 13, u8; /// Disable Receiver Abort on PHR Error
        dis_rsde,   14, 14, u8; /// Disable Receiver Abort on RSD Error
        fcs_init2f, 15, 15, u8; /// FCS Seed Selection
        phr_mode,   16, 17, u8; /// PHR Mode
        dis_stxp,   18, 18, u8; /// Disable Smart TX Power Control
        rxm110k,    22, 22, u8; /// Receiver Mode 110kpbs Data Rate
        rxwtoe,     28, 28, u8; /// Receiver Wait Timeout Enable
        rxautr,     29, 29, u8; /// Receiver Auto-Re-Enable
        autoack,    30, 30, u8; /// Automatic Acknowledgement Enable
        aackpend,   31, 31, u8; /// Automatic Acknowledgement Pending
    }
    0x06, 0x00, 5, RO, SYS_TIME(sys_time) { /// System Time Counter
        value, 0, 39, u64; /// System Time Counter
    }
    0x08, 0x00, 5, RW, TX_FCTRL(tx_fctrl) { /// TX Frame Control
        tflen,     0,  6, u8;  /// TX Frame Length
        tfle,      7,  9, u8;  /// TX Frame Length Extension
        txbr,     13, 14, u8;  /// TX Bit Rate
        tr,       15, 15, u8;  /// TX Ranging Enable
        txprf,    16, 17, u8;  /// TX Pulse Repetition Frequency
        txpsr,    18, 19, u8;  /// TX Preamble Symbol Repetitions
        pe,       20, 21, u8;  /// Preamble Extension
        txboffs,  22, 31, u16; /// TX Buffer Index Offset
        ifsdelay, 32, 39, u8;  /// Inter-Frame Spacing
    }
    0x0A, 0x00, 5, RW, DX_TIME(dx_time) { /// Delayed Send or Receive Time
        value, 0, 39, u64; /// Delayed Send or Receive Time
    }
    0x0D, 0x00, 4, RW, SYS_CTRL(sys_ctrl) { /// System Control Register
        sfcst,      0,  0, u8; /// Suppress Auto-FCS Transmission
        txstrt,     1,  1, u8; /// Transmit Start
        txdlys,     2,  2, u8; /// Transmitter Delayed Sending
        cansfcs,    3,  3, u8; /// Cancel Auto-FCS Suppression
        trxoff,     6,  6, u8; /// Transceiver Off
        wait4resp,  7,  7, u8; /// Wait for Response
        rxenab,     8,  8, u8; /// Enable Receiver
        rxdlye,     9,  9, u8; /// Receiver Delayed Enable
        hrbpt,     24, 24, u8; /// Host Side RX Buffer Pointer Toggle
    }
    0x0E, 0x00, 4, RW, SYS_MASK(sys_mask) { /// System Event Mask Register
        mpclock,    1,  1, u8; /// Mask clock PLL lock
        mesyncr,    2,  2, u8; /// Mask external sync clock reset
        maat,       3,  3, u8; /// Mask automatic acknowledge trigger
        mtxfrbm,    4,  4, u8; /// Mask transmit frame begins
        mtxprs,     5,  5, u8; /// Mask transmit preamble sent
        mtxphs,     6,  6, u8; /// Mask transmit PHY Header Sent
        mtxfrs,     7,  7, u8; /// Mask transmit frame sent
        mrxprd,     8,  8, u8; /// Mask receiver preamble detected
        mrxsfdd,    9,  9, u8; /// Mask receiver SFD detected
        mldedone,  10, 10, u8; /// Mask LDE processing done
        mrxphd,    11, 11, u8; /// Mask receiver PHY header detect
        mrxphe,    12, 12, u8; /// Mask receiver PHY header error
        mrxdfr,    13, 13, u8; /// Mask receiver data frame ready
        mrxfcg,    14, 14, u8; /// Mask receiver FCS good
        mrxfce,    15, 15, u8; /// Mask receiver FCS error
        mrxrfsl,   16, 16, u8; /// Mask receiver Reed Solomon Frame Sync loss
        mrxrfto,   17, 17, u8; /// Mask Receive Frame Wait Timeout
        mldeerr,   18, 18, u8; /// Mask leading edge detection processing error
        mrxovrr,   20, 20, u8; /// Mask Receiver Overrun
        mrxpto,    21, 21, u8; /// Mask Preamble detection timeout
        mgpioirq,  22, 22, u8; /// Mask GPIO interrupt
        mslp2init, 23, 23, u8; /// Mask SLEEP to INIT event
        mrfpllll,  24, 24, u8; /// Mask RF PLL Losing Lock warning
        mcpllll,   25, 25, u8; /// Mask Clock PLL Losing Lock warning
        mrxsfdto,  26, 26, u8; /// Mask Receive SFD timeout
        mhpdwarn,  27, 27, u8; /// Mask Half Period Delay Warning
        mtxberr,   28, 28, u8; /// Mask Transmit Buffer Error
        maffrej,   29, 29, u8; /// Mask Automatic Frame Filtering rejection
    }
    0x0F, 0x00, 5, RW, SYS_STATUS(sys_status) { /// System Event Status Register
        irqs,       0,  0, u8; /// Interrupt Request Status
        cplock,     1,  1, u8; /// Clock PLL Lock
        esyncr,     2,  2, u8; /// External Sync Clock Reset
        aat,        3,  3, u8; /// Automatic Acknowledge Trigger
        txfrb,      4,  4, u8; /// TX Frame Begins
        txprs,      5,  5, u8; /// TX Preamble Sent
        txphs,      6,  6, u8; /// TX PHY Header Sent
        txfrs,      7,  7, u8; /// TX Frame Sent
        rxprd,      8,  8, u8; /// RX Preamble Detected
        rxsfdd,     9,  9, u8; /// RX SFD Detected
        ldedone,   10, 10, u8; /// LDE Processing Done
        rxphd,     11, 11, u8; /// RX PHY Header Detect
        rxphe,     12, 12, u8; /// RX PHY Header Error
        rxdfr,     13, 13, u8; /// RX Data Frame Ready
        rxfcg,     14, 14, u8; /// RX FCS Good
        rxfce,     15, 15, u8; /// RX FCS Error
        rxrfsl,    16, 16, u8; /// RX Reed-Solomon Frame Sync Loss
        rxrfto,    17, 17, u8; /// RX Frame Wait Timeout
        ldeerr,    18, 18, u8; /// Leading Edge Detection Error
        rxovrr,    20, 20, u8; /// RX Overrun
        rxpto,     21, 21, u8; /// Preamble Detection Timeout
        gpioirq,   22, 22, u8; /// GPIO Interrupt
        slp2init,  23, 23, u8; /// SLEEP to INIT
        rfpll_ll,  24, 24, u8; /// RF PLL Losing Lock
        clkpll_ll, 25, 25, u8; /// Clock PLL Losing Lock
        rxsfdto,   26, 26, u8; /// Receive SFD Timeout
        hpdwarn,   27, 27, u8; /// Half Period Delay Warning
        txberr,    28, 28, u8; /// TX Buffer Error
        affrej,    29, 29, u8; /// Auto Frame Filtering Rejection
        hsrbp,     30, 30, u8; /// Host Side RX Buffer Pointer
        icrbp,     31, 31, u8; /// IC Side RX Buffer Pointer
        rxrscs,    32, 32, u8; /// RX Reed-Solomon Correction Status
        rxprej,    33, 33, u8; /// RX Preamble Rejection
        txpute,    34, 34, u8; /// TX Power Up Time Error
    }
    0x10, 0x00, 4, RO, RX_FINFO(rx_finfo) { /// RX Frame Information
        rxflen,  0,  6, u8; /// Receive Frame Length
        rxfle,   7,  9, u8; /// Receive Frame Length Extension
        rxnspl, 11, 12, u8; /// Receive Non-Standard Preamble Length
        rxbr,   13, 14, u8; /// Receive Bit Rate Report
        rng,    15, 15, u8; /// Receiver Ranging
        rxprfr, 16, 17, u8; /// RX Pulse Repetition Rate Report
        rxpsr,  18, 19, u8; /// RX Preamble Repetition
    }
    0x15, 0x00, 14, RO, RX_TIME(rx_time) { /// Receive Time Stamp
        rx_stamp,  0,  39, u64; /// Fully adjusted time stamp
        fp_index, 40,  55, u16; /// First Path Index
        fp_ampl1, 56,  71, u16; /// First Path Amplitude Point 1
        rx_rawst, 72, 111, u64; /// Raw time stamp
    }
    0x17, 0x00, 10, RO, TX_TIME(tx_time) { /// Transmit Time Stamp
        tx_stamp,  0, 39, u64; /// Fully adjusted time stamp
        tx_rawst, 40, 79, u64; /// Raw time stamp
    }
    0x18, 0x00, 2, RW, TX_ANTD(tx_antd) { /// TX Antenna Delay
        value, 0, 15, u16; /// TX Antenna Delay
    }
    0x19, 0x00, 5, RO, SYS_STATE(sys_state) { /// System State information
        tx_state,    0,  3, u8; /// Current Transmit State Machine value
        rx_state,    8, 12, u8; /// Current Receive State Machine value
        pmsc_state, 16, 23, u8; /// Current PMSC State Machine value
    }
    0x1E, 0x00, 4, RW, TX_POWER(tx_power) { /// TX Power Control
        // The TX_POWER register has multiple sets of fields defined, depending
        // on the smart TX power control setting. I don't know how to model
        // this, so I've opted to provide just a single `value` field for
        // maximum flexibility.
        value, 0, 31, u32; /// TX Power Control value
    }
    0x1F, 0x00, 4, RW, CHAN_CTRL(chan_ctrl) { /// Channel Control Register
        tx_chan, 0, 3, u8; /// Selects the transmit channel.
        rx_chan, 4, 7, u8; /// Selects the receive channel.
        dwsfd, 17, 17, u8; /// Enables the non-standard Decawave proprietary SFD sequence.
        rxprf, 18, 19, u8; /// Selects the PRF used in the receiver.
        tnssfd, 20, 20, u8; /// This bit enables the use of a user specified (non-standard) SFDin the transmitter.
        rnssfd, 21, 21, u8; /// This bit enables the use of a user specified (non-standard) SFDin the receiver.
        tx_pcode, 22, 26, u8; /// This field selects the preamble code used in the transmitter.
        rx_pcode, 27, 31, u8; /// This field selects the preamble code used in the receiver.
    }
    0x21, 0x00, 1, RW, SFD_LENGTH(sfd_length) { /// This is the length of the SFD sequence used when the data rate is 850kbps and higher.
        value, 0, 7, u8; /// This is the length of the SFD sequence used when the data rate is 850kbps and higher.
    }
    0x23, 0x04, 2, RW, AGC_TUNE1(agc_tune1) { /// AGC Tuning register 1
        value, 0, 15, u16; /// AGC Tuning register 1 value
    }
    0x23, 0x0C, 4, RW, AGC_TUNE2(agc_tune2) { /// AGC Tuning register 2
        value, 0, 31, u32; /// AGC Tuning register 2 value
    }
    0x24, 0x00, 4, RW, EC_CTRL(ec_ctrl) { /// External Clock Sync Counter Config
        ostsm,   0,  0, u8; /// External Transmit Synchronization Mode Enable
        osrsm,   1,  1, u8; /// External Receive Synchronization Mode Enable
        pllldt,  2,  2, u8; /// Clock PLL Lock Detect Tune
        wait,    3, 10, u8; /// Wait Counter
        ostrm,  11, 11, u8; /// External Timebase Reset Mode Enable
    }
    0x26, 0x00, 4, RW, GPIO_MODE(gpio_mode) { /// GPIO Mode Control Register
        msgp0,  6,  7, u8; /// Mode Selection for GPIO0/RXOKLED
        msgp1,  8,  9, u8; /// Mode Selection for GPIO1/SFDLED
        msgp2, 10, 11, u8; /// Mode Selection for GPIO2/RXLED
        msgp3, 12, 13, u8; /// Mode Selection for GPIO3/TXLED
        msgp4, 14, 15, u8; /// Mode Selection for GPIO4/EXTPA
        msgp5, 16, 17, u8; /// Mode Selection for GPIO5/EXTTXE
        msgp6, 18, 19, u8; /// Mode Selection for GPIO6/EXTRXE
        msgp7, 20, 21, u8; /// Mode Selection for SYNC/GPIO7
        msgp8, 22, 23, u8; /// Mode Selection for IRQ/GPIO8
    }
    0x26, 0x08, 4, RW, GPIO_DIR(gpio_dir) { /// GPIO Direction Control Register
        gdp0,  0,  0, u8; /// Direction Selection for GPIO0
        gdp1,  1,  1, u8; /// Direction Selection for GPIO1
        gdp2,  2,  2, u8; /// Direction Selection for GPIO2
        gdp3,  3,  3, u8; /// Direction Selection for GPIO3
        gdm0,  4,  4, u8; /// Mask for setting the direction of GPIO0
        gdm1,  5,  5, u8; /// Mask for setting the direction of GPIO1
        gdm2,  6,  6, u8; /// Mask for setting the direction of GPIO2
        gdm3,  7,  7, u8; /// Mask for setting the direction of GPIO3
        gdp4,  8,  8, u8; /// Direction Selection for GPIO4
        gdp5,  9,  9, u8; /// Direction Selection for GPIO5
        gdp6, 10, 10, u8; /// Direction Selection for GPIO6
        gdp7, 11, 11, u8; /// Direction Selection for GPIO7
        gdm4, 12, 12, u8; /// Mask for setting the direction of GPIO4
        gdm5, 13, 13, u8; /// Mask for setting the direction of GPIO5
        gdm6, 14, 14, u8; /// Mask for setting the direction of GPIO6
        gdm7, 15, 15, u8; /// Mask for setting the direction of GPIO7
        gdp8, 16, 16, u8; /// Direction Selection for GPIO8
        gdm8, 20, 20, u8; /// Mask for setting the direction of GPIO8
    }
    0x26, 0x0C, 4, RW, GPIO_DOUT(gpio_dout) { /// GPIO Data Output register
        gop0,  0,  0, u8; /// Output state setting for GPIO0
        gop1,  1,  1, u8; /// Output state setting for GPIO1
        gop2,  2,  2, u8; /// Output state setting for GPIO2
        gop3,  3,  3, u8; /// Output state setting for GPIO3
        gom0,  4,  4, u8; /// Mask for setting the output state of GPIO0
        gom1,  5,  5, u8; /// Mask for setting the output state of GPIO1
        gom2,  6,  6, u8; /// Mask for setting the output state of GPIO2
        gom3,  7,  7, u8; /// Mask for setting the output state of GPIO3
        gop4,  8,  8, u8; /// Output state setting for GPIO4
        gop5,  9,  9, u8; /// Output state setting for GPIO5
        gop6, 10, 10, u8; /// Output state setting for GPIO6
        gop7, 11, 11, u8; /// Output state setting for GPIO7
        gom4, 12, 12, u8; /// Mask for setting the output state of GPIO4
        gom5, 13, 13, u8; /// Mask for setting the output state of GPIO5
        gom6, 14, 14, u8; /// Mask for setting the output state of GPIO6
        gom7, 15, 15, u8; /// Mask for setting the output state of GPIO7
        gop8, 16, 16, u8; /// Output state setting for GPIO8
        gom8, 20, 20, u8; /// Mask for setting the output state of GPIO8
    }
    0x26, 0x10, 4, RW, GPIO_IRQE(gpio_irqe) { /// GPIO Interrupt Enable
        girqe0,  0,  0, u8; /// GPIO IRQ Enable for GPIO0 input
        girqe1,  1,  1, u8; /// GPIO IRQ Enable for GPIO1 input
        girqe2,  2,  2, u8; /// GPIO IRQ Enable for GPIO2 input
        girqe3,  3,  3, u8; /// GPIO IRQ Enable for GPIO3 input
        girqe4,  4,  4, u8; /// GPIO IRQ Enable for GPIO4 input
        girqe5,  5,  5, u8; /// GPIO IRQ Enable for GPIO5 input
        girqe6,  6,  6, u8; /// GPIO IRQ Enable for GPIO6 input
        girqe7,  7,  7, u8; /// GPIO IRQ Enable for GPIO7 input
        girqe8,  8,  8, u8; /// GPIO IRQ Enable for GPIO8 input
    }
    0x26, 0x14, 4, RW, GPIO_ISEN(gpio_isen) { /// GPIO Interrupt Sense Selection
        gisen0,  0,  0, u8; /// GPIO IRQ sense for GPIO0 input
        gisen1,  1,  1, u8; /// GPIO IRQ sense for GPIO1 input
        gisen2,  2,  2, u8; /// GPIO IRQ sense for GPIO2 input
        gisen3,  3,  3, u8; /// GPIO IRQ sense for GPIO3 input
        gisen4,  4,  4, u8; /// GPIO IRQ sense for GPIO4 input
        gisen5,  5,  5, u8; /// GPIO IRQ sense for GPIO5 input
        gisen6,  6,  6, u8; /// GPIO IRQ sense for GPIO6 input
        gisen7,  7,  7, u8; /// GPIO IRQ sense for GPIO7 input
        gisen8,  8,  8, u8; /// GPIO IRQ sense for GPIO8 input
    }
    0x26, 0x18, 4, RW, GPIO_IMODE(gpio_imode) { /// GPIO Interrupt Mode (Level / Edge)
        gimod0,  0,  0, u8; /// GPIO IRQ mode selection for GPIO0 input
        gimod1,  1,  1, u8; /// GPIO IRQ mode selection for GPIO1 input
        gimod2,  2,  2, u8; /// GPIO IRQ mode selection for GPIO2 input
        gimod3,  3,  3, u8; /// GPIO IRQ mode selection for GPIO3 input
        gimod4,  4,  4, u8; /// GPIO IRQ mode selection for GPIO4 input
        gimod5,  5,  5, u8; /// GPIO IRQ mode selection for GPIO5 input
        gimod6,  6,  6, u8; /// GPIO IRQ mode selection for GPIO6 input
        gimod7,  7,  7, u8; /// GPIO IRQ mode selection for GPIO7 input
        gimod8,  8,  8, u8; /// GPIO IRQ mode selection for GPIO8 input
    }
    0x26, 0x1C, 4, RW, GPIO_IBES(gpio_ibes) { /// GPIO Interrupt “Both Edge” Select
        gibes0,  0,  0, u8; /// GPIO IRQ "Both Edges" selection for GPIO0 input
        gibes1,  1,  1, u8; /// GPIO IRQ "Both Edges" selection for GPIO1 input
        gibes2,  2,  2, u8; /// GPIO IRQ "Both Edges" selection for GPIO2 input
        gibes3,  3,  3, u8; /// GPIO IRQ "Both Edges" selection for GPIO3 input
        gibes4,  4,  4, u8; /// GPIO IRQ "Both Edges" selection for GPIO4 input
        gibes5,  5,  5, u8; /// GPIO IRQ "Both Edges" selection for GPIO5 input
        gibes6,  6,  6, u8; /// GPIO IRQ "Both Edges" selection for GPIO6 input
        gibes7,  7,  7, u8; /// GPIO IRQ "Both Edges" selection for GPIO7 input
        gibes8,  8,  8, u8; /// GPIO IRQ "Both Edges" selection for GPIO8 input
    }
    0x26, 0x20, 4, RW, GPIO_ICLR(gpio_iclr) { /// GPIO Interrupt Latch Clear
        giclr0,  0,  0, u8; /// GPIO IRQ latch clear for GPIO0 input
        giclr1,  1,  1, u8; /// GPIO IRQ latch clear for GPIO1 input
        giclr2,  2,  2, u8; /// GPIO IRQ latch clear for GPIO2 input
        giclr3,  3,  3, u8; /// GPIO IRQ latch clear for GPIO3 input
        giclr4,  4,  4, u8; /// GPIO IRQ latch clear for GPIO4 input
        giclr5,  5,  5, u8; /// GPIO IRQ latch clear for GPIO5 input
        giclr6,  6,  6, u8; /// GPIO IRQ latch clear for GPIO6 input
        giclr7,  7,  7, u8; /// GPIO IRQ latch clear for GPIO7 input
        giclr8,  8,  8, u8; /// GPIO IRQ latch clear for GPIO8 input
    }
    0x26, 0x24, 4, RW, GPIO_IDBE(gpio_idbe) { /// GPIO Interrupt De-bounce Enable
        gidbe0,  0,  0, u8; /// GPIO IRQ de-bounce enable for GPIO0
        gidbe1,  1,  1, u8; /// GPIO IRQ de-bounce enable for GPIO1
        gidbe2,  2,  2, u8; /// GPIO IRQ de-bounce enable for GPIO2
        gidbe3,  3,  3, u8; /// GPIO IRQ de-bounce enable for GPIO3
        gidbe4,  4,  4, u8; /// GPIO IRQ de-bounce enable for GPIO4
        gidbe5,  5,  5, u8; /// GPIO IRQ de-bounce enable for GPIO5
        gidbe6,  6,  6, u8; /// GPIO IRQ de-bounce enable for GPIO6
        gidbe7,  7,  7, u8; /// GPIO IRQ de-bounce enable for GPIO7
        gidbe8,  8,  8, u8; /// GPIO IRQ de-bounce enable for GPIO8
    }
    0x26, 0x28, 4, RW, GPIO_RAW(gpio_raw) { /// GPIO raw state
        grawp0,  0,  0, u8; /// GPIO0 port raw state
        grawp1,  1,  1, u8; /// GPIO1 port raw state
        grawp2,  2,  2, u8; /// GPIO2 port raw state
        grawp3,  3,  3, u8; /// GPIO3 port raw state
        grawp4,  4,  4, u8; /// GPIO4 port raw state
        grawp5,  5,  5, u8; /// GPIO5 port raw state
        grawp6,  6,  6, u8; /// GPIO6 port raw state
        grawp7,  7,  7, u8; /// GPIO7 port raw state
        grawp8,  8,  8, u8; /// GPIO8 port raw state
    }
    0x27, 0x02, 2, RW, DRX_TUNE0B(drx_tune0b) { /// Digital Tuning Register 0b
        value, 0, 15, u16; /// DRX_TUNE0B tuning value
    }
    0x27, 0x04, 2, RW, DRX_TUNE1A(drx_tune1a) { /// Digital Tuning Register 1a
        value, 0, 15, u16; /// DRX_TUNE1A tuning value
    }
    0x27, 0x06, 2, RW, DRX_TUNE1B(drx_tune1b) { /// Digital Tuning Register 1b
        value, 0, 15, u16; /// DRX_TUNE1B tuning value
    }
    0x27, 0x08, 4, RW, DRX_TUNE2(drx_tune2) { /// Digital Tuning Register 2
        value, 0, 31, u32; /// DRX_TUNE2 tuning value
    }
    0x27, 0x20, 2, RW, DRX_SFDTOC(drx_sfdtoc) { /// SFD timeout
        count, 0, 15, u16; /// SFD detection timeout count
    }
    0x27, 0x24, 2, RW, DRX_PRETOC(drx_pretoc) { /// Preamble detection timeou
        count, 0, 15, u16; /// Preamble detection timeout count
    }
    0x27, 0x26, 2, RW, DRX_TUNE4H(drx_tune4h) { /// Digital Tuning Register 4h
        value, 0, 15, u16; /// DRX_TUNE4H tuning value
    }
    0x27, 0x28, 2, RO, DRX_CAR_INT(dxr_car_int) { /// Carrier Recovery Integrator Register
        value, 0, 15, u16; /// value
    }
    0x27, 0x2C, 2, RO, RXPACC_NOSAT(rxpacc_nosat) { /// Digital debug register. Unsaturated accumulated preamble symbols.
        value, 0, 15, u16; /// value
    }
    0x28, 0x0B, 1, RW, RF_RXCTRLH(rf_rxctrlh) { /// Analog RX Control Register
        value, 0, 7, u8; /// Analog RX Control Register
    }
    0x28, 0x0C, 3, RW, RF_TXCTRL(rf_txctrl) { /// Analog TX Control Register
        txmtune, 5,  8, u8; /// Transmit mixer tuning register
        txmq,    9, 11, u8; /// Transmit mixer Q-factor tuning register
        value, 0, 23, u32; /// The entire register
    }
    0x28, 0x30, 5, RW, LDOTUNE(ldotune) { /// LDO voltage tuning parameter
        value, 0, 39, u64; /// Internal LDO voltage tuning parameter
    }
    0x2A, 0x0B, 1, RW, TC_PGDELAY(tc_pgdelay) { /// Pulse Generator Delay
        value, 0, 7, u8; /// Transmitter Calibration - Pulse Generator Delay
    }
    0x2B, 0x07, 4, RW, FS_PLLCFG(fs_pllcfg) { /// Frequency synth - PLL configuration
        value, 0, 31, u32; /// Frequency synth - PLL configuration
    }
    0x2B, 0x0B, 1, RW, FS_PLLTUNE(fs_plltune) { /// Frequency synth - PLL Tuning
        value, 0, 7, u8; /// Frequency synthesiser - PLL Tuning
    }
    0x2D, 0x04, 2, RW, OTP_ADDR(otp_addr) { /// OTP Address
        value, 0, 10, u16; /// OTP Address
    }
    0x2D, 0x06, 2, RW, OTP_CTRL(otp_ctrl) { /// OTP Control
        otprden,  0,  0, u8; /// Forces OTP into manual read mode
        otpread,  1,  1, u8; /// Commands a read operation
        otpmrwr,  3,  3, u8; /// OTP mode register write
        otpprog,  6,  6, u8; /// Write OTP_WDAT to OTP_ADDR
        otpmr,    7, 10, u8; /// OTP mode register
        ldeload, 15, 15, u8; /// Force load of LDE microcode
    }
    0x2D, 0x0A, 4, RO, OTP_RDAT(otp_rdat) { /// OTP Read Data
        value, 0, 31, u32; /// OTP Read Data
    }
    0x2E, 0x0806, 1, RW, LDE_CFG1(lde_cfg1) { /// LDE Configuration Register 1
        ntm,   0, 4, u8; /// Noise Threshold Multiplier
        pmult, 5, 7, u8; /// Peak Multiplier
    }
    0x2E, 0x1804, 2, RW, LDE_RXANTD(lde_rxantd) { /// RX Antenna Delay
        value, 0, 15, u16; /// RX Antenna Delay
    }
    0x2E, 0x1806, 2, RW, LDE_CFG2(lde_cfg2) { /// LDE Configuration Register 2
        value, 0, 15, u16; /// The LDE_CFG2 configuration value
    }
    0x2F, 0x00, 4, RW, EVC_CTRL(evc_ctrl) { /// Event Counter Control
        evc_en,  0, 0, u8; /// Event Counters Enable
        evc_clr, 1, 1, u8; /// Event Counters Clear
    }
    0x2F, 0x18, 2, RO, EVC_HPW(evc_hpw) { /// Half Period Warning Counter
        value, 0, 11, u16; /// Half Period Warning Event Counter
    }
    0x2F, 0x1A, 2, RO, EVC_TPW(evc_tpw) { /// TX Power-Up Warning Counter
        value, 0, 11, u16; /// TX Power-Up Warning Event Counter
    }
    0x36, 0x00, 4, RW, PMSC_CTRL0(pmsc_ctrl0) { /// PMSC Control Register 0
        sysclks,    0,  1, u8; /// System Clock Selection
        rxclks,     2,  3, u8; /// Receiver Clock Selection
        txclks,     4,  5, u8; /// Transmitter Clock Selection
        face,       6,  6, u8; /// Force Accumulator Clock Enable
        adcce,     10, 10, u8; /// ADC Clock Enable
        amce,      15, 15, u8; /// Accumulator Memory Clock Enable
        gpce,      16, 16, u8; /// GPIO Clock Enable
        gprn,      17, 17, u8; /// GPIO Reset (Not), active low
        gpdce,     18, 18, u8; /// GPIO De-bounce Clock Enable
        gpdrn,     19, 19, u8; /// GPIO De-bounce Reset (Not), active low
        khzclken,  23, 23, u8; /// Kilohertz Clock Enable
        softreset, 28, 31, u8; /// Soft Reset
    }
    0x36, 0x04, 4, RW, PMSC_CTRL1(pmsc_ctrl1) { /// PMSC Control Register 1
        arx2init,   1,  1, u8; /// Automatic transition from receive to init
        pktseq,     3, 10, u8; /// Control PMSC control of analog RF subsystem
        atxslp,    11, 11, u8; /// After TX automatically sleep
        arxslp,    12, 12, u8; /// After RX automatically sleep
        snoze,     13, 13, u8; /// Snooze Enable
        snozr,     14, 14, u8; /// Snooze Repeat
        pllsyn,    15, 15, u8; /// Enable clock used for external sync modes
        lderune,   17, 17, u8; /// LDE Run Enable
        khzclkdiv, 26, 31, u8; /// Kilohertz Clock Divisor
    }
    0x36, 0x28, 4, RW, PMSC_LEDC(pmsc_ledc) { /// PMSC LED Control Register
        blink_tim, 0, 7, u8; /// Blink time count value
        blnken, 8, 8, u8; /// Blink Enable
        blnknow, 16, 19, u8; /// Manually triggers an LED blink. There is one trigger bit per LED IO
    }
}
*/

/// Transmit Data Buffer
///
/// Currently only the first 127 bytes of the buffer are supported, which is
/// enough to support standard Standard IEEE 802.15.4 UWB frames.
#[allow(non_camel_case_types)]
pub struct TX_BUFFER;

impl Register for TX_BUFFER {
    const ID:     u8    = 0x14;
    const SUB_ID: u16   = 0x00;
    const LEN:    usize = 127;
}

impl Writable for TX_BUFFER {
    type Write = tx_buffer::W;

    fn write() -> Self::Write {
        tx_buffer::W([0; 127 + 1])
    }

    fn buffer(w: &mut Self::Write) -> &mut [u8] {
        &mut w.0
    }
}

impl<SPI, CS> DW1000<SPI, CS> {
    /// Transmit Data Buffer
    pub fn tx_buffer(&mut self) -> RegAccessor<TX_BUFFER, SPI, CS> {
        RegAccessor(self, PhantomData)
    }
}


/// Transmit Data Buffer
pub mod tx_buffer {
    /// Used to write to the register
    pub struct W(pub(crate) [u8; 127 + 1]);

    impl W {
        /// Provides write access to the buffer contents
        pub fn data(&mut self) -> &mut [u8] {
            &mut self.0[1..]
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
    const ID:     u8    = 0x12;
    const SUB_ID: u16   = 0x00;
    const LEN:    usize = 127;
}

impl Readable for RX_BUFFER_0 {
    type Read = rx_buffer_0::R;

    fn read() -> Self::Read {
        rx_buffer_0::R([0; 127 + 1])
    }

    fn buffer(w: &mut Self::Read) -> &mut [u8] {
        &mut w.0
    }
}

impl<SPI, CS> DW1000<SPI, CS> {
    /// Receive Data Buffer
    pub fn rx_buffer_0(&mut self) -> RegAccessor<RX_BUFFER_0, SPI, CS> {
        RegAccessor(self, PhantomData)
    }
}


/// Receive Data Buffer
pub mod rx_buffer_0 {
    use core::fmt;


    const HEADER_LEN: usize = 1;
    const LEN:        usize = 127;


    /// Used to read from the register
    pub struct R(pub(crate) [u8; HEADER_LEN + LEN]);

    impl R {
        /// Provides read access to the buffer contents
        pub fn data(&self) -> &[u8] {
            &self.0[HEADER_LEN .. HEADER_LEN + LEN]
        }
    }

    impl fmt::Debug for R {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "0x")?;
            for i in (0 .. LEN).rev() {
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
    const ID:     u8    = 0x13;
    const SUB_ID: u16   = 0x00;
    const LEN:    usize = 127;
}

impl Readable for RX_BUFFER_1 {
    type Read = rx_buffer_1::R;

    fn read() -> Self::Read {
        rx_buffer_1::R([0; 127 + 1])
    }

    fn buffer(w: &mut Self::Read) -> &mut [u8] {
        &mut w.0
    }
}

impl<SPI, CS> DW1000<SPI, CS> {
    /// Receive Data Buffer1
    pub fn rx_buffer_1(&mut self) -> RegAccessor<RX_BUFFER_1, SPI, CS> {
        RegAccessor(self, PhantomData)
    }
}


/// Receive Data Buffer
pub mod rx_buffer_1 {
    use core::fmt;


    const HEADER_LEN: usize = 1;
    const LEN:        usize = 127;


    /// Used to read from the register
    pub struct R(pub(crate) [u8; HEADER_LEN + LEN]);

    impl R {
        /// Provides read access to the buffer contents
        pub fn data(&self) -> &[u8] {
            &self.0[HEADER_LEN .. HEADER_LEN + LEN]
        }
    }

    impl fmt::Debug for R {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "0x")?;
            for i in (0 .. LEN).rev() {
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
