#![allow(unused_imports)]

use embedded_hal_async::spi;

use crate::{Error, Ready, Sleeping, DW3000};

impl<SPI> DW3000<SPI, Sleeping>
where
    SPI: spi::SpiDevice<u8>,
{
    /*
    /// Wakes the radio up.
    pub fn wake_up<DELAY: embedded_hal::blocking::delay::DelayUs<u16>>(
        mut self,
        delay: &mut DELAY,
    ) -> Result<DW3000<SPI, Ready>, Error<SPI>> {
        // Wake up using the spi
        self.ll.assert_cs_low().map_err(|e| Error::Spi(e))?;
        delay.delay_us(850 * 2);
        self.ll.assert_cs_high().map_err(|e| Error::Spi(e))?;

        // Now we must wait 4 ms so all the clocks start running.
        delay.delay_us(4000 * 2);

        // Let's check that we're actually awake now
        if self.ll.dev_id().read()?.ridtag() != 0xDECA {
            // Oh dear... We have not woken up!
            return Err(Error::StillAsleep);
        }

        // Reset the wakeupstatus
        self.ll.sys_status().write(|w| w.slp2init(1).cplock(1))?;

        // Restore the tx antenna delay
        let delay = self.state.tx_antenna_delay;
        self.ll.tx_antd().write(|w| w.value(delay.value() as u16))?;

        // All other values should be restored, so return the ready radio.
        Ok(DW3000 {
            ll: self.ll,
            seq: self.seq,
            state: Ready,
        })
    }*/
}
