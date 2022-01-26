use embedded_hal::{blocking::spi, digital::v2::OutputPin};

use crate::{
	ll,
	mac,
	time::{Duration, Instant},
	Error,
	DW3000,
	configs,
	Ready,
};
use super::Awake;

impl<SPI, CS, State> DW3000<SPI, CS, State>
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

	/// Returns the state of the DW3000
	pub fn state(&mut self) -> Result<u8, Error<SPI, CS>> {
		Ok(self.ll.sys_state().read()?.pmsc_state())
	}

	/// Returns the current fast command of the DW3000
	pub fn cmd_status(&mut self) -> Result<u8, Error<SPI, CS>> {
		Ok(self.ll.fcmd_stat().read()?.value())
	}

	/// Returns true if the DW3000 has been in init_rc
	pub fn init_rc_passed(&mut self) -> Result<bool, Error<SPI, CS>> {
		Ok(self.ll.sys_status().read()?.rcinit() == 0x1)
	}

	/// Returns true if the DW3000 has been in idle_rc
	pub fn idle_rc_passed(&mut self) -> Result<bool, Error<SPI, CS>> {
		Ok(self.ll.sys_status().read()?.spirdy() == 0x1)
	}

	/// Returns true if the DW3000 pll is lock
	pub fn idle_pll_passed(&mut self) -> Result<bool, Error<SPI, CS>> {
		Ok(self.ll.sys_status().read()?.cplock() == 0x1)
	}

	/// Provides direct access to the register-level API
	///
	/// Be aware that by using the register-level API, you can invalidate
	/// various assumptions that the high-level API makes about the operation of
	/// the DW3000. Don't use the register-level and high-level APIs in tandem,
	/// unless you know what you're doing.
	pub fn ll(&mut self) -> &mut ll::DW3000<SPI, CS> { &mut self.ll }

	/// Force the DW3000 into IDLE mode
	///
	/// Any ongoing RX/TX operations will be aborted.
	pub fn force_idle(&mut self) -> Result<(), Error<SPI, CS>> {
		// our probleme on this function is that we never come back to IDLE_PLL with a locked PLL after usng fast command 0
		
		self.ll.fast_command(0)?;
		//while self.ll.sys_status().read()?.rcinit() == 0 {}
		//while self.ll.sys_status().read()?.cplock() == 0 {}
		Ok(())
	}

	/// Use fast command ll in hl
	pub fn fast_cmd(&mut self, fc: configs::FastCommand) -> Result<(), Error<SPI, CS>> {
		self.ll.fast_command(fc as u8)?;
		Ok(())
	}
}
