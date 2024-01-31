use embedded_hal::spi;

use super::Awake;
use crate::{fast_command, ll, time::Duration, Error, DW3000};

use smoltcp::wire::{Ieee802154Address, Ieee802154Pan};

impl<SPI, State> DW3000<SPI, State>
where
    SPI: spi::SpiDevice<u8>,
    State: Awake,
{
    /// Returns the TX antenna delay
    pub fn get_tx_antenna_delay(&mut self) -> Result<Duration, Error<SPI>> {
        let tx_antenna_delay = self.ll.tx_antd().read()?.value();

        // Since `tx_antenna_delay` is `u16`, the following will never panic.
        let tx_antenna_delay = Duration::new(tx_antenna_delay.into()).unwrap();

        Ok(tx_antenna_delay)
    }

    /// Returns the RX antenna delay
    pub fn get_rx_antenna_delay(&mut self) -> Result<Duration, Error<SPI>> {
        let rx_antenna_delay = self.ll.cia_conf().read()?.rxantd();

        // Since `rx_antenna_delay` is `u16`, the following will never panic.
        let rx_antenna_delay = Duration::new(rx_antenna_delay.into()).unwrap();

        Ok(rx_antenna_delay)
    }

    /// Returns the network id and address used for sending and receiving
    pub fn get_address(&mut self) -> Result<(Ieee802154Pan, Ieee802154Address), Error<SPI>> {
        let panadr = self.ll.panadr().read()?;

        Ok((
            smoltcp::wire::Ieee802154Pan(panadr.pan_id()),
            Ieee802154Address::Short(panadr.short_addr().to_be_bytes()),
        ))
    }

    /// Returns the current system time (32-bit)
    pub fn sys_time(&mut self) -> Result<u32, Error<SPI>> {
        let sys_time = self.ll.sys_time().read()?.value();

        Ok(sys_time)
    }

    /// Returns the state of the DW3000
    pub fn state(&mut self) -> Result<u8, Error<SPI>> {
        Ok(self.ll.sys_state().read()?.pmsc_state())
    }

    /// Returns the current fast command of the DW3000
    pub fn cmd_status(&mut self) -> Result<u8, Error<SPI>> {
        Ok(self.ll.fcmd_stat().read()?.value())
    }

    /// Returns true if the DW3000 has been in init_rc
    pub fn init_rc_passed(&mut self) -> Result<bool, Error<SPI>> {
        Ok(self.ll.sys_status().read()?.rcinit() == 0x1)
    }

    /// Returns true if the DW3000 has been in idle_rc
    pub fn idle_rc_passed(&mut self) -> Result<bool, Error<SPI>> {
        Ok(self.ll.sys_status().read()?.spirdy() == 0x1)
    }

    /// Returns true if the DW3000 pll is lock
    pub fn idle_pll_passed(&mut self) -> Result<bool, Error<SPI>> {
        Ok(self.ll.sys_status().read()?.cplock() == 0x1)
    }

    /// Provides direct access to the register-level API
    ///
    /// Be aware that by using the register-level API, you can invalidate
    /// various assumptions that the high-level API makes about the operation of
    /// the DW3000. Don't use the register-level and high-level APIs in tandem,
    /// unless you know what you're doing.
    pub fn ll(&mut self) -> &mut ll::DW3000<SPI> {
        &mut self.ll
    }

    /// Force the DW3000 into IDLE mode
    ///
    /// Any ongoing RX/TX operations will be aborted.
    pub fn force_idle(&mut self) -> Result<(), Error<SPI>> {
        // our probleme on this function is that we never come back to IDLE_PLL with a locked PLL after usng fast command 0

        self.ll.fast_command(0)?;
        //while self.ll.sys_status().read()?.rcinit() == 0 {}
        //while self.ll.sys_status().read()?.cplock() == 0 {}
        Ok(())
    }

    /// Use fast command ll in hl
    pub fn fast_cmd(&mut self, fc: fast_command::FastCommand) -> Result<(), Error<SPI>> {
        self.ll.fast_command(fc as u8)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

    #[test]
    fn test_new_device() {
        let spi = SpiMock::new(&[
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![0x40, 0x30, 0, 0, 0, 0],
                vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
            ),
            SpiTransaction::transaction_end(),
        ]);

        let mut dw3000 = DW3000::new(spi);

        let addr = dw3000.get_address().unwrap();

        assert_eq!(
            addr,
            (
                Ieee802154Pan(0x0605),
                Ieee802154Address::Short([0x04, 0x03])
            )
        );

        let mut spi = dw3000.ll.spi;

        spi.done();
    }
}
