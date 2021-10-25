#![allow(unused_imports)]

use core::num::Wrapping;

use byte::BytesExt as _;
use embedded_hal::{blocking::spi, digital::v2::OutputPin};
use ieee802154::mac::{self, FooterMode};

use super::AutoDoubleBufferReceiving;
use crate::{
	configs::SfdSequence,
	time::Instant,
	Error,
	Ready,
	Config,
	Sending,
	SingleBufferReceiving,
	Sleeping,
	DW3000,
	FastCommand,
};

/// The behaviour of the sync pin
pub enum SyncBehaviour {
	/// The sync pin does nothing
	None,
	/// The radio time will reset to 0 when the sync pin is high and the clock
	/// gives a rising edge
	TimeBaseReset,
	/// When receiving, instead of reading the internal timestamp, the time
	/// since the last sync is given back.
	ExternalSync,
	/// When receiving, instead of reading the internal timestamp, the time
	/// since the last sync is given back. Also resets the internal timebase
	/// back to 0.
	ExternalSyncWithReset,
}

/// The time at which the transmission will start
pub enum SendTime {
	/// As fast as possible
	Now,
	/// After some time
	Delayed(Instant),
	/// After the sync pin is engaged. (Only works when sync setup is in
	/// ExternalSync mode)
	OnSync,
}

/// The polarity of the irq signal
pub enum IrqPolarity {
	/// The signal will be high when the interrupt is active
	ActiveHigh = 1,
	/// The signal will be low when the interrupt is active
	ActiveLow  = 0,
}

impl<SPI, CS> DW3000<SPI, CS, Ready>
where
	SPI: spi::Transfer<u8> + spi::Write<u8>,
	CS: OutputPin,
{
	/// Sets the RX and TX antenna delays
	pub fn set_antenna_delay(
		&mut self,
		rx_delay: u16,
		tx_delay: u16,
	) -> Result<(), Error<SPI, CS>> {
		self.ll.cia_conf().modify(|_, w| w.rxantd(rx_delay))?;
		self.ll.tx_antd().write(|w| w.value(tx_delay))?;

		Ok(())
	}

	/// Sets the network id and address used for sending and receiving
	pub fn set_address(
		&mut self,
		pan_id: mac::PanId,
		addr: mac::ShortAddress,
	) -> Result<(), Error<SPI, CS>> {
		self.ll
			.panadr()
			.write(|w| w.pan_id(pan_id.0).short_addr(addr.0))?;

		Ok(())
	}

	/// Send an IEEE 802.15.4 MAC frame
	///
	/// The `data` argument is wrapped into an IEEE 802.15.4 MAC frame and sent
	/// to `destination`.
	///
	/// This operation can be delayed to aid in distance measurement, by setting
	/// `delayed_time` to `Some(instant)`. If you want to send the frame as soon
	/// as possible, just pass `None` instead.
	///
	/// The config parameter struct allows for setting the channel, bitrate, and
	/// more. This configuration needs to be the same as the configuration used
	/// by the receiver, or the message may not be received.
	/// The defaults are a sane starting point.
	///
	/// This method starts the transmission and returns immediately thereafter.
	/// It consumes this instance of `DW3000` and returns another instance which
	/// is in the `Sending` state, and can be used to wait for the transmission
	/// to finish and check its result.
	pub fn send(
		mut self,
		data: &[u8],
		send_time: SendTime,
		config: Config,
	) -> Result<DW3000<SPI, CS, Sending>, Error<SPI, CS>> {
		/*
				// Clear event counters
				self.ll.evc_ctrl().write(|w| w.evc_clr(0b1))?;
				while self.ll.evc_ctrl().read()?.evc_clr() == 0b1 {}

				// (Re-)Enable event counters
				self.ll.evc_ctrl().write(|w| w.evc_en(0b1))?;
				while self.ll.evc_ctrl().read()?.evc_en() == 0b1 {}

				// Sometimes, for unknown reasons, the DW3000 gets stuck in RX mode.
				// Starting the transmitter won't get it to enter TX mode, which means
				// all subsequent send operations will fail. Let's disable the
				// transceiver and force the chip into IDLE mode to make sure that
				// doesn't happen.
				self.force_idle(false)?;
		*/
		let seq = self.seq.0;
		self.seq += Wrapping(1);

		let frame = mac::Frame {
			header:  mac::Header {
				frame_type: mac::FrameType::Data,
				version: mac::FrameVersion::Ieee802154_2006,
				security: mac::Security::None,
				frame_pending: false,
				ack_request: false,
				pan_id_compress: false,
				destination: mac::Address::broadcast(&mac::AddressMode::Short),
				source: Some(self.get_address()?),
				seq,
			},
			content: mac::FrameContent::Data,
			payload: data,
			footer:  [0; 2],
		};

		match send_time {
			| SendTime::Delayed(time) => {
				// Put the time into the delay register
				// By setting this register, the chip knows to delay before transmitting
				self.ll
					.dx_time()
					.write(|w| // 32-bits value of the most significant bits
                    w.value( (time.value() >> 8) as u32 ))?;
			},
			| SendTime::OnSync => {
				self.ll.ec_ctrl().modify(|_, w| w.ostr_mode(1))?;
				self.ll.ec_ctrl().modify(|_, w| w.osts_wait(33))?;
			},
			| _ => {},
		}

		// Prepare transmitter
		let mut len = 0;
		self.ll.tx_buffer().write(|w| {
			let result = w.data().write_with(&mut len, frame, FooterMode::None);

			if let Err(err) = result {
				panic!("Failed to write frame: {:?}", err);
			}

			w
		})?;

		let txb_offset = 0; // no offset in TX_BUFFER
		let mut txb_offset_errata = txb_offset;
		if txb_offset > 127 {
			// Errata in DW3000, see page 86
			txb_offset_errata += 128;
		}

		self.ll.tx_fctrl().modify(|_, w| {
			let txflen = len as u16 + 2;
			w   .txflen(txflen) // data length + two-octet CRC
				.txbr(config.bitrate as u8) // configured bitrate
				.tr(config.ranging_enable as u8) // configured ranging bit
				.txb_offset(txb_offset_errata) // no offset in TX_BUFFER
				.txpsr(config.preamble_length as u8) // configure preamble length
				.fine_plen(0) // Not implemented, replacing txpsr
		})?;

		match send_time {
			SendTime::Now => self.fast_cmd(FastCommand::CMD_TX)?,
			_ =>  self.fast_cmd(FastCommand::CMD_DTX)?, // Start TX
		}

		Ok(DW3000 {
			ll:    self.ll,
			seq:   self.seq,
			state: Sending { finished: false },
		})
	}

	/// Attempt to receive a single IEEE 802.15.4 MAC frame
	///
	/// Initializes the receiver. The method consumes this instance of `DW3000`
	/// and returns another instance which is in the [SingleBufferReceiving]
	/// state, and can be used to wait for a message.
	///
	/// The config parameter allows for the configuration of bitrate, channel
	/// and more. Make sure that the values used are the same as of the frames
	/// that are transmitted. The default works with the TxConfig's default and
	/// is a sane starting point.
	pub fn receive(
		self,
		config: Config,
	) -> Result<DW3000<SPI, CS, SingleBufferReceiving>, Error<SPI, CS>> {
		let mut rx_radio = DW3000 {
			ll:    self.ll,
			seq:   self.seq,
			state: SingleBufferReceiving {
				finished: false,
				config,
			},
		};

		// Start rx'ing
		rx_radio.start_receiving(config)?;

		// Return the double buffer state
		Ok(rx_radio)
	}

	/// Enables transmit interrupts for the events that `wait` checks
	///
	/// Overwrites any interrupt flags that were previously set.
	pub fn enable_tx_interrupts(&mut self) -> Result<(), Error<SPI, CS>> {
		self.ll.sys_enable().modify(|_, w| w.txfrs_en(0b1))?;
		Ok(())
	}

	/// Enables receive interrupts for the events that `wait` checks
	///
	/// Overwrites any interrupt flags that were previously set.
	pub fn enable_rx_interrupts(&mut self) -> Result<(), Error<SPI, CS>> {
		self.ll().sys_enable().modify(|_, w| {
			w.rxprd_en(0b1)
				.rxsfdd_en(0b1)
				.rxphd_en(0b1)
				.rxphe_en(0b1)
				.rxfr_en(0b1)
				.rxfcg_en(0b1)
				.rxfce_en(0b1)
				.rxrfsl_en(0b1)
				.rxfto_en(0b1)
				.rxovrr_en(0b1)
				.rxpto_en(0b1)
				.rxsto_en(0b1)
				.rxprej_en(0b1)
		})?;
		Ok(())
	}

	/// Disables all interrupts
	pub fn disable_interrupts(&mut self) -> Result<(), Error<SPI, CS>> {
		self.ll.sys_enable().write(|w| w)?;
		Ok(())
	}
	
}
