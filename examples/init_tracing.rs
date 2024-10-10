// This example uses a dummy SPI/GPIO implementation to test what happens when
// the `dw3000_ng` driver is initialized.
use dw3000_ng::{Config, DW3000};

use embedded_hal_bus::spi::ExclusiveDevice;

/// Simulated DW3000 state
#[derive(Debug, PartialEq, Eq)]
enum SimulatedState {
    /// Startup
    Startup,
    /// Starting PLL calibration
    StartingPLLCalibration,
    /// PLL calibration done
    PLLCalibrationDone,
}

/// Dummy SPI driver that implements the `embedded-hal` SPI traits.
struct DummySpi {
    state: SimulatedState,
}

impl DummySpi {
    /// Decode the SPI message header generated by `init_header`
    fn decode_header(header: &[u8]) -> (u8, u8, bool) {
        let write = header[0] & 0x80 != 0;
        let id = (header[0] & 0x3e) >> 1;
        let sub_id = ((header[0] & 0x01) << 6) | (header[1] >> 2);
        (id, sub_id, write)
    }
}

#[derive(Debug)]
struct DummyError;
impl embedded_hal::spi::Error for DummyError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        embedded_hal::spi::ErrorKind::Other
    }
}

impl embedded_hal::spi::ErrorType for DummySpi {
    type Error = DummyError;
}

impl embedded_hal::spi::SpiBus<u8> for DummySpi {
    fn read(&mut self, _data: &mut [u8]) -> Result<(), Self::Error> {
        log::debug!("SPI read");
        Ok(())
    }

    fn write(&mut self, _data: &[u8]) -> Result<(), Self::Error> {
        let (id, sub_id, write) = DummySpi::decode_header(_data);

        if id == 0x11 && sub_id == 0x08 && self.state == SimulatedState::Startup {
            // [e2, 20, 00, 01, 00, 00]
            if _data == [0xe2, 0x20, 0x00, 0x01, 0x00, 0x00] {
                log::info!("PLL calibration initiated");
                self.state = SimulatedState::StartingPLLCalibration;
            }
        }
        log::debug!(
            "SPI write: {:02x?} (id: {:02x}, sub_id: {:02x}, write: {})",
            _data,
            id,
            sub_id,
            write
        );
        Ok(())
    }

    fn transfer(&mut self, _in: &mut [u8], _out: &[u8]) -> Result<(), Self::Error> {
        log::debug!("SPI transfer: {:02x?} -> {:02x?}", _in, _out);
        Ok(())
    }

    fn transfer_in_place(&mut self, _data: &mut [u8]) -> Result<(), Self::Error> {
        let (id, sub_id, write) = DummySpi::decode_header(_data);
        log::debug!(
            "SPI in-place transfer: {:02x?} (id: {:02x}, sub_id: {:02x}, write: {})",
            _data,
            id,
            sub_id,
            write
        );

        if self.state == SimulatedState::StartingPLLCalibration {
            log::info!("PLL calibration done");
            self.state = SimulatedState::PLLCalibrationDone;
        }

        if self.state == SimulatedState::PLLCalibrationDone {
            if id == 0x00 && sub_id == 0x44 {
                // Reading SYS_STATUS register (6 bytes)
                // Let's return a success
                _data[2..].copy_from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00, 0x00]);
            }

            // rx_cal_sts
            if id == 0x04 && sub_id == 0x20 {
                // Reading RX_CAL_STS register (1 byte)
                // Let's return a success
                _data[2] = 0x01;

                log::info!("Driver reading RX_CAL_STS register");
            }
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        log::debug!("SPI flush");
        Ok(())
    }
}

/// Dummy GPIO driver that implements the `embedded-hal` GPIO traits.
struct DummyGpio;

#[derive(Debug)]
struct DummyGpioError;
impl embedded_hal::digital::Error for DummyGpioError {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

impl embedded_hal::digital::OutputPin for DummyGpio {
    fn set_high(&mut self) -> Result<(), DummyGpioError> {
        log::trace!("GPIO set high");
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), DummyGpioError> {
        log::trace!("GPIO set low");
        Ok(())
    }
}

impl embedded_hal::digital::ErrorType for DummyGpio {
    type Error = DummyGpioError;
}

fn main() {
    // logging setup
    env_logger::init();

    let spi = DummySpi {
        state: SimulatedState::Startup,
    };
    let gpio = DummyGpio {};
    let spi_dev = ExclusiveDevice::new_no_delay(spi, gpio).unwrap();

    let dw3000 = DW3000::new(spi_dev);

    let config = Config::default();

    let dw3000 = dw3000
        .config(config, |delay| {
            std::thread::sleep(std::time::Duration::from_micros(delay as u64))
        })
        .unwrap();

    log::info!("DW3000 initialized");
}