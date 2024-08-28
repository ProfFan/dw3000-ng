use core::fmt;
use core::fmt::{Display, Formatter};

use embedded_hal::spi;

#[cfg(feature = "defmt")]
use defmt::Format;

use crate::ll;

/// An error that can occur when sending or receiving data
pub enum Error<SPI>
where
    SPI: spi::ErrorType,
{
    /// Error occured while using SPI bus
    Spi(ll::Error<SPI>),

    /// Receiver FCS error
    Fcs,

    /// PHY header error
    Phy,

    /// Buffer too small
    BufferTooSmall {
        /// Indicates how large a buffer would have been required
        required_len: usize,
    },

    /// Receiver Reed Solomon Frame Sync Loss
    ReedSolomon,

    /// Receiver Frame Wait Timeout
    FrameWaitTimeout,

    /// Receiver Overrun
    Overrun,

    /// Preamble Detection Timeout
    PreambleDetectionTimeout,

    /// Receiver SFD Timeout
    SfdTimeout,

    /// Frame was rejected because due to automatic frame filtering
    ///
    /// It seems that frame filtering is typically handled transparently by the
    /// hardware, and filtered frames aren't usually visible to the driver.
    /// However, sometimes a filtered frame bubbles up and disrupts an ongoing
    /// receive operation, which then causes this error.
    FrameFilteringRejection,

    /// Frame could not be decoded
    Frame(byte::Error),

    /// A delayed frame could not be sent in time
    ///
    /// Please note that the frame was still sent. Replies could still arrive,
    /// and if it was a ranging frame, the resulting range measurement will be
    /// wrong.
    DelayedSendTooLate,

    /// Transmitter could not power up in time for delayed send
    ///
    /// The frame was still transmitted, but the first bytes of the preamble
    /// were likely corrupted.
    DelayedSendPowerUpWarning,

    /// The configuration was not valid. Some combinations of settings are not
    /// allowed.
    InvalidConfiguration,

    /// The receive operation hasn't finished yet
    RxNotFinished,

    /// It was expected that the radio would have woken up, but it hasn't.
    StillAsleep,

    /// The RSSI was not calculable.
    BadRssiCalculation,

    /// There are issues with frame filtering in double buffer mode.
    /// So it's not supported now.
    RxConfigFrameFilteringUnsupported,

    /// Failed Initialization
    InitializationFailed,

    /// Failed to calibrate the PGF values
    PGFCalibrationFailed,
}

impl<SPI> From<ll::Error<SPI>> for Error<SPI>
where
    SPI: spi::ErrorType,
{
    fn from(error: ll::Error<SPI>) -> Self {
        Error::Spi(error)
    }
}

impl<SPI> Display for Error<SPI>
where
    SPI: spi::ErrorType,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "std")]
impl<SPI> std::error::Error for Error<SPI> where SPI: spi::ErrorType {}

// We can't derive this implementation, as `Debug` is only implemented
// conditionally for `ll::Debug`.
impl<SPI> fmt::Debug for Error<SPI>
where
    SPI: spi::ErrorType,
    SPI::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Spi(error) => write!(f, "Spi({:?})", error),
            Error::Fcs => write!(f, "Fcs"),
            Error::Phy => write!(f, "Phy"),
            Error::BufferTooSmall { required_len } => {
                write!(f, "BufferTooSmall {{ required_len: {:?} }}", required_len,)
            }
            Error::ReedSolomon => write!(f, "ReedSolomon"),
            Error::FrameWaitTimeout => write!(f, "FrameWaitTimeout"),
            Error::Overrun => write!(f, "Overrun"),
            Error::PreambleDetectionTimeout => write!(f, "PreambleDetectionTimeout"),
            Error::SfdTimeout => write!(f, "SfdTimeout"),
            Error::FrameFilteringRejection => write!(f, "FrameFilteringRejection"),
            Error::Frame(error) => write!(f, "Frame({:?})", error),
            Error::DelayedSendTooLate => write!(f, "DelayedSendTooLate"),
            Error::DelayedSendPowerUpWarning => write!(f, "DelayedSendPowerUpWarning"),
            Error::InvalidConfiguration => write!(f, "InvalidConfiguration"),
            Error::RxNotFinished => write!(f, "RxNotFinished"),
            Error::StillAsleep => write!(f, "StillAsleep"),
            Error::BadRssiCalculation => write!(f, "BadRssiCalculation"),
            Error::RxConfigFrameFilteringUnsupported => {
                write!(f, "RxConfigFrameFilteringUnsupported")
            }
            Error::InitializationFailed => write!(f, "InitializationFailed"),
            Error::PGFCalibrationFailed => write!(f, "PGFCalibrationFailed"),
        }
    }
}

#[cfg(feature = "defmt")]

// We can't derive this implementation, as `Debug` is only implemented
// conditionally for `ll::Debug`.
impl<SPI> Format for Error<SPI>
where
    SPI: spi::SpiDevice<u8>,
    SPI::Error: defmt::Format,
{
    fn format(&self, f: defmt::Formatter) {
        match self {
            Error::Spi(error) => defmt::write!(f, "Spi({:?})", error),
            Error::Fcs => defmt::write!(f, "Fcs"),
            Error::Phy => defmt::write!(f, "Phy"),
            Error::BufferTooSmall { required_len } => {
                defmt::write!(f, "BufferTooSmall {{ required_len: {:?} }}", required_len,)
            }
            Error::ReedSolomon => defmt::write!(f, "ReedSolomon"),
            Error::FrameWaitTimeout => defmt::write!(f, "FrameWaitTimeout"),
            Error::Overrun => defmt::write!(f, "Overrun"),
            Error::PreambleDetectionTimeout => defmt::write!(f, "PreambleDetectionTimeout"),
            Error::SfdTimeout => defmt::write!(f, "SfdTimeout"),
            Error::FrameFilteringRejection => defmt::write!(f, "FrameFilteringRejection"),
            Error::Frame(error) => defmt::write!(f, "Frame({:?})", defmt::Debug2Format(error)),
            Error::DelayedSendTooLate => defmt::write!(f, "DelayedSendTooLate"),
            Error::DelayedSendPowerUpWarning => defmt::write!(f, "DelayedSendPowerUpWarning"),
            Error::InvalidConfiguration => defmt::write!(f, "InvalidConfiguration"),
            Error::RxNotFinished => defmt::write!(f, "RxNotFinished"),
            Error::StillAsleep => defmt::write!(f, "StillAsleep"),
            Error::BadRssiCalculation => defmt::write!(f, "BadRssiCalculation"),
            Error::RxConfigFrameFilteringUnsupported => {
                defmt::write!(f, "RxConfigFrameFilteringUnsupported")
            }
            Error::InitializationFailed => defmt::write!(f, "InitializationFailed"),
            Error::PGFCalibrationFailed => defmt::write!(f, "PGFCalibrationFailed"),
        }
    }
}

// Tests
#[cfg(test)]
mod test {
    use super::*;

    use embedded_hal_mock::eh1::spi::Mock as SpiMock;

    #[cfg(feature = "defmt")]
    #[test]
    fn test_defmt() {
        let error = Error::<SpiMock<u8>>::BufferTooSmall { required_len: 42 };

        defmt::info!("error: {:?}", error);
    }
}
