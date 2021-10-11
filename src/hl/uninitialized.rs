use crate::{ll, Error, Ready, Uninitialized, DW1000};
use core::num::Wrapping;
use embedded_hal::{
	blocking::{delay::DelayMs, spi},
	digital::v2::OutputPin,
};
//use rtt_target::{rprintln};

impl<SPI, CS> DW1000<SPI, CS, Uninitialized>
where
	SPI: spi::Transfer<u8> + spi::Write<u8>,
	CS: OutputPin,
{
	/// Create a new instance of `DW1000`
	///
	/// Requires the SPI peripheral and the chip select pin that are connected
	/// to the DW1000.
	pub fn new(spi: SPI, chip_select: CS) -> Self {
		DW1000 {
			ll: ll::DW1000::new(spi, chip_select),
			seq: Wrapping(0),
			state: Uninitialized,
		}
	}

	/// Initialize the DW1000
	///
	/// The DW1000's default configuration is somewhat inconsistent, and the
	/// user manual (section 2.5.5) has a long list of default configuration
	/// values that should be changed to guarantee everything works correctly.
	/// This method does just that.
	///
	/// Please note that this method assumes that you kept the default
	/// configuration. It is generally recommended not to change configuration
	/// before calling this method.
	pub fn init<D: DelayMs<u8>>(
		mut self,
		_delay: &mut D,
	) -> Result<DW1000<SPI, CS, Ready>, Error<SPI, CS>> {
		// no need for basic initialisation anymore !!!

		// Much of the systeme conf is conf in SYS_CFG register
		// page 26 section 2.5.2
		/*
			   // Set LDOTUNE. See user manual, section 2.5.5.11.
			   self.ll.otp_addr().write(|w| w.value(0x004))?;
			   self.ll.otp_ctrl().modify(|_, w|
				   w
					   .otprden(0b1)
					   .otpread(0b1)
			   )?;
			   while self.ll.otp_ctrl().read()?.otpread() == 0b1 {}
			   let ldotune_low = self.ll.otp_rdat().read()?.value();
			   if ldotune_low != 0 {
				   self.ll.otp_addr().write(|w| w.value(0x005))?;
				   self.ll.otp_ctrl().modify(|_, w|
					   w
						   .otprden(0b1)
						   .otpread(0b1)
				   )?;
				   while self.ll.otp_ctrl().read()?.otpread() == 0b1 {}
				   let ldotune_high = self.ll.otp_rdat().read()?.value();

				   let ldotune = ldotune_low as u64 | (ldotune_high as u64) << 32;
				   self.ll.ldotune().write(|w| w.value(ldotune))?;
			   }
		*/

		// Wait for the IDLE_RC state
		while self.ll.sys_status().read()?.rcinit() == 0 {}

		// CONFIGURATION GENERALE
		// DRX_CONF have some values that should be modified
		// for best performance
		// TO DO !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

		// RF_TX_CTRL_1 need to be changed for optimal performance (page 151)

		// CONFIGURATION DE LA PLL POUR PASSER DANS L'ETAT IDLE PLL
		// need to change default cal value for pll (page164)
		self.ll.pll_cal().modify(|_, w| w.pll_cfg_ld(0x81))?;
		// clear cplock
		self.ll.sys_status().write(|w| w.cplock(0))?;
		// select PLL mode auto
		self.ll.clk_ctrl().modify(|_, w| w.sys_clk(0))?;
		// set ainit2idle
		self.ll.seq_ctrl().modify(|_, w| w.ainit2idle(1))?;
		// Set the on wake up switch from idle RC to idle PLL
		self.ll.aon_dig_cfg().modify(|_, w| w.onw_go2idle(1))?;
		// wait for CPLOCK to be set
		while self.ll.sys_status().read()?.cplock() == 0 {}

		Ok(DW1000 {
			ll: self.ll,
			seq: self.seq,
			state: Ready,
		})
	}
}
