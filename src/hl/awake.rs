use crate::{
    ll, mac,
    time::{Duration, Instant},
    Error, DW1000,
};
use embedded_hal::{blocking::spi, digital::v2::OutputPin};

use super::Awake;

impl<SPI, CS, State> DW1000<SPI, CS, State>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
    State: Awake,
{
    /// Returns the TX antenna delay
    pub fn get_tx_antenna_delay(&mut self) -> Result<Duration, Error<SPI, CS>> {
        let tx_antenna_delay = self.ll.tx_antd().read()?.value();

        // Since `tx_antenna_delay` is `u16`, the following will never panic.
        let tx_antenna_delay = Duration::new(tx_antenna_delay.into()).unwrap();

        Ok(tx_antenna_delay)
    }

    /// Returns the RX antenna delay
    pub fn get_rx_antenna_delay(&mut self) -> Result<Duration, Error<SPI, CS>> {
        let rx_antenna_delay = self.ll.cia_conf().read()?.rxantd();

        // Since `rx_antenna_delay` is `u16`, the following will never panic.
        let rx_antenna_delay = Duration::new(rx_antenna_delay.into()).unwrap();

        Ok(rx_antenna_delay)
    }

    /// Returns the network id and address used for sending and receiving
    pub fn get_address(&mut self) -> Result<mac::Address, Error<SPI, CS>> {
        let panadr = self.ll.panadr().read()?;

        Ok(mac::Address::Short(
            mac::PanId(panadr.pan_id()),
            mac::ShortAddress(panadr.short_addr()),
        ))
    }

    /// Returns the current system time
    pub fn sys_time(&mut self) -> Result<Instant, Error<SPI, CS>> {
        let sys_time = self.ll.sys_time().read()?.value();

        // Since hardware timestamps fit within 40 bits, the following should
        // never panic.
        Ok(Instant::new(sys_time.into()).unwrap())
    }

    /// Provides direct access to the register-level API
    ///
    /// Be aware that by using the register-level API, you can invalidate
    /// various assumptions that the high-level API makes about the operation of
    /// the DW1000. Don't use the register-level and high-level APIs in tandem,
    /// unless you know what you're doing.
    pub fn ll(&mut self) -> &mut ll::DW1000<SPI, CS> {
        &mut self.ll
    }

    /// Force the DW1000 into IDLE mode
    ///
    /// Any ongoing RX/TX operations will be aborted.
    pub(super) fn force_idle(&mut self, double_buffered: bool) -> Result<(), Error<SPI, CS>> {

        let mut saved_sys_mask = [0; 8];

        if double_buffered {
            // Mask the double buffered status bits
            self.ll.sys_enable().modify(|r, w| {
                saved_sys_mask = r.0.clone();
                w.rxfce_en(0).rxfcg_en(0).rxfr_en(0)
            })?;
        }

        self.ll.fast_command(0)?;

        if double_buffered {
            // Clear the bits
            self.ll().sys_status().write(
                |w| {
                    w   .rxprd(0b1) // Receiver Preamble Detected
                        .rxsfdd(0b1) // Receiver SFD Detected
                        .rxphd(0b1) // Receiver PHY Header Detected
                        .rxphe(0b1) // Receiver PHY Header Error
                        .rxfr(0b1) // Receiver Data Frame Ready
                        .rxfcg(0b1) // Receiver FCS Good
                        .rxfce(0b1) // Receiver FCS Error
                        .rxfsl(0b1) // Receiver Reed Solomon Frame Sync Loss
                        .rxfto(0b1) // Receiver Frame Wait Timeout
                        .rxovrr(0b1) // Receiver Overrun
                        .rxpto(0b1) // Preamble Detection Timeout
                        .rxsto(0b1) // Receiver SFD Timeout
                        .rxprej(0b1)
                }, // Receiver Preamble Rejection
            )?;

            // Restore the mask
            self.ll.sys_enable().write(|w| {
                w.0.copy_from_slice(&saved_sys_mask);
                w
            })?;
        }

        Ok(())
        
    }
/*
    pub(crate) fn read_otp(&mut self, address: u16) -> Result<u32, Error<SPI, CS>> {
        // Set address
        self.ll.otp_addr().write(|w| w.value(address))?;
        // Switch into read mode
        self.ll.otp_ctrl().write(|w| w.otprden(0b1).otpread(0b1))?;
        self.ll.otp_ctrl().write(|w| w.otprden(0b1))?;
        // Read back value
        let value = self.ll.otp_rdat().read()?.value();
        // End read mode
        self.ll.otp_ctrl().write(|w| w)?;
        Ok(value)
    }
*/
}
