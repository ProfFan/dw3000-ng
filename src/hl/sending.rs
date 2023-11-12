#![allow(unused_imports)]

use embedded_hal::{blocking::spi, digital::v2::OutputPin};
use nb;

use crate::{time::Instant, Error, Ready, Sending, DW3000};

impl<SPI, CS> DW3000<SPI, CS, Sending>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
{
    /// Returns the TX state of the DW3000
    pub fn tx_state(&mut self) -> Result<u8, Error<SPI, CS>> {
        Ok(self.ll.sys_state().read()?.tx_state())
    }

    /// Wait for the transmission to finish
    ///
    /// This method returns an `nb::Result` to indicate whether the transmission
    /// has finished, or whether it is still ongoing. You can use this to busily
    /// wait for the transmission to finish, for example using `nb`'s `block!`
    /// macro, or you can use it in tandem with [`DW3000::enable_tx_interrupts`]
    /// and the DW3000 IRQ output to wait in a more energy-efficient manner.
    ///
    /// Handling the DW3000's IRQ output line is out of the scope of this
    /// driver, but please note that if you're using the DWM1001 module or
    /// DWM1001-Dev board, that the `dwm1001` crate has explicit support for
    /// this.
    #[inline(always)]
    pub fn s_wait(&mut self) -> nb::Result<Instant, Error<SPI, CS>> {
        // Check Half Period Warning Counter. If this is a delayed transmission,
        // this will indicate that the delay was too short, and the frame was
        // sent too late.
        let evc_hpw = self
            .ll
            .evc_hpw()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?
            .value();

        if evc_hpw != 0 {
            return Err(nb::Error::Other(Error::DelayedSendTooLate));
        }

        // WARNING s:
        // If you're changing anything about which SYS_STATUS flags are being
        // checked in this method, also make sure to update `enable_interrupts`.
        let fint_stat = self
            .ll
            .fint_stat()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?;

        // Has the frame been sent?
        if fint_stat.txok() == 0b0 {
            // Frame has not been sent
            return Err(nb::Error::WouldBlock);
        }

        // Frame sent
        self.reset_flags().map_err(nb::Error::Other)?;
        self.state.mark_finished();

        let tx_timestamp = self
            .ll
            .tx_time()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?
            .tx_stamp();
        // This is safe because the value read from the device will never be higher than
        // the allowed value.
        let tx_timestamp = Instant::new(tx_timestamp);

        if let Some(ts) = tx_timestamp {
            Ok(ts)
        } else {
            Err(nb::Error::Other(Error::Fcs))
        }
    }

    #[allow(clippy::type_complexity)]
    /// Finishes sending and returns to the `Ready` state
    ///
    /// If the send operation has finished, as indicated by `wait`, this is a
    /// no-op. If the send operation is still ongoing, it will be aborted.
    pub fn finish_sending(mut self) -> Result<DW3000<SPI, CS, Ready>, (Self, Error<SPI, CS>)> {
        // In order to avoid undetermined states after a sending, we will force the state to idle

        if !self.state.is_finished() {
            match self.force_idle() {
                Ok(()) => (),
                Err(error) => return Err((self, error)),
            }
            match self.reset_flags() {
                Ok(()) => (),
                Err(error) => return Err((self, error)),
            }
        }

        Ok(DW3000 {
            ll: self.ll,
            seq: self.seq,
            state: Ready,
        })
    }

    fn reset_flags(&mut self) -> Result<(), Error<SPI, CS>> {
        self.ll.sys_status().write(|w| {
            w.txfrb(0b1) // Transmit Frame Begins
                .txprs(0b1) // Transmit Preamble Sent
                .txphs(0b1) // Transmit PHY Header Sent
                .txfrs(0b1) // Transmit Frame Sent
        })?;

        Ok(())
    }
}
