//! Configuration structs for sending and receiving
//!
//! This module houses the datastructures that control how frames are
//! transmitted and received. The configs are passed to the send and receive
//! functions.

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Fast command enumeration to easily control the module
pub enum FastCommand {
	/// Fast command to go to IDLE state and clears any events
	CMD_TXRXOFF     = 0x0,
	/// Fast command to Immediate start of transmission
	CMD_TX          = 0x1,
	/// Fast command to Enable RX immediately
	CMD_RX          = 0x2,
	/// Fast command to Delayed TX w.r.t. DX_TIME
	CMD_DTX         = 0x3,
	/// Fast command to Delayed RX w.r.t. DX_TIME
	CMD_DRX         = 0x4,
	/// Fast command to Delayed TX w.r.t. TX timestamp + DX_TIME
	CMD_DTX_TS      = 0x5,
	/// Fast command to Delayed RX w.r.t. TX timestamp + DX_TIME
	CMD_DRX_TS      = 0x6,
	/// Fast command to Delayed TX w.r.t. RX timestamp + DX_TIME
	CMD_DTX_RS      = 0x7,
	/// Fast command to Delayed RX w.r.t. RX timestamp + DX_TIME
	CMD_DRX_RS      = 0x8,
	/// Fast command to Delayed TX w.r.t. DREF_TIME + DX_TIME
	CMD_DTX_REF     = 0x9,
	/// Fast command to Delayed RX w.r.t. DREF_TIME + DX_TIME
	CMD_DRX_REF     = 0xA,
	/// Fast command to TX if no preamble detected
	CMD_CCA_TX      = 0xB,
	/// Fast command to Start TX immediately, then when TX is done, enable the receiver
	CMD_TX_W4R      = 0xC,
	/// Fast command to Delayed TX w.r.t. DX_TIME, then enable receiver
	CMD_DTX_W4R     = 0xD,
	/// Fast command to Delayed TX w.r.t. TX timestamp + DX_TIME, then enable receiver
	CMD_DTX_TS_W4R  = 0xE,
	/// Fast command to Delayed TX w.r.t. RX timestamp + DX_TIME, then enable receiver
	CMD_DTX_RS_W4R  = 0xF,
	/// Fast command to Delayed TX w.r.t. DREF_TIME + DX_TIME, then enable receiver
	CMD_DTX_REF_W4R = 0x10, 
	/// Fast command to TX packet if no preamble detected, then enable receiver
	CMD_CCA_TX_W4R  = 0x11,
	/// Fast command to Clear all interrupt events
	CMD_CLR_IRQS    = 0x12,
	/// Fast command to Toggle double buffer pointer / notify the device that the host has finished processing the received buffer/data.
	CMD_DB_TOGGLE   = 0x13,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// General configuration for TX and RX
pub struct Config {
	/// The channel that the DW3000 will transmit at.
	pub channel:                    UwbChannel,
	/// The SFD sequence that is used to transmit a frame.
	pub sfd_sequence:               SfdSequence,
	/// Sets the PRF value of the transmission.
	pub pulse_repetition_frequency: PulseRepetitionFrequency,
	/// The length of the preamble.
	pub preamble_length:            PreambleLength,
	/// Sets the bitrate of the transmission.
	pub bitrate:                    BitRate,
	/// Defaults to `true`.
	pub frame_filtering:            bool,
	/// Sets the ranging bit in the transmitted frame.
	/// This has no effect on the capabilities of the DW3000.
	/// maybe can be degaged
	pub ranging_enable:             bool,

}

impl Default for Config {
	fn default() -> Self {
		Config {
			channel:                    Default::default(),
			sfd_sequence:               Default::default(),
			pulse_repetition_frequency: Default::default(),
			preamble_length:            Default::default(),
			bitrate:                    Default::default(),
			frame_filtering:            false,
			ranging_enable:				false,
		}
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// The bitrate at which a message is transmitted
pub enum BitRate {
	/// 850 kilobits per second.
	Kbps850  = 0,
	/// 6.8 megabits per second.
	Kbps6800 = 1,
}

impl Default for BitRate {
	fn default() -> Self { BitRate::Kbps6800 }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// The PRF value
pub enum PulseRepetitionFrequency {
	/// 16 megahertz
	Mhz16 = 0b01,
	/// 64 megahertz
	Mhz64 = 0b10,
}

impl Default for PulseRepetitionFrequency {
	fn default() -> Self { PulseRepetitionFrequency::Mhz64 }
}

/// imple
impl PulseRepetitionFrequency {
	/// activate rx_tune_en if prf = 64MHz
	pub fn get_recommended_rx_tune_en(&self) -> u8 {
		match self {
			| PulseRepetitionFrequency::Mhz16 => 0,
			| PulseRepetitionFrequency::Mhz64 => 1,
		}
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// An enum that specifies the length of the preamble.
///
/// Longer preambles improve the reception quality and thus range.
/// This comes at the cost of longer transmission times and thus power
/// consumption and bandwidth use.
///
/// For the bit pattern, see table 16 in the user manual. Two bits TXPSR,then
/// two bits PE.
pub enum PreambleLength {
	/// 64 symbols of preamble.
	/// Only supported at Bitrate::Kbps6800.
	Symbols64   = 0b0001,
	/// 1024 symbols of preamble.
	/// Only supported at Bitrate::Kbps850 & Bitrate::Kbps6800.
	Symbols1024 = 0b0010,
	/// 128 symbols of preamble.
	/// Only supported at Bitrate::Kbps850 & Bitrate::Kbps6800.
	/// Unofficial extension from decawave.
	Symbols4096 = 0b0011,
	/// 4096 symbols of preamble.
	Symbols32   = 0b0100,
	/// 32 symbols of preamble.
	Symbols128  = 0b0101,
	/// 128 symbols of preamble.
	Symbols1536 = 0b0110,
	/// 1536 symbols of preamble.
	Symbols256  = 0b1001,
	/// 256 symbols of preamble.
	Symbols2048 = 0b1010,
	/// 512 symbols of preamble.
	Symbols512  = 0b1101,
}

impl Default for PreambleLength {
	fn default() -> Self { PreambleLength::Symbols64 }
}

impl PreambleLength {
	/// Gets the recommended PAC size based on the preamble length.
	pub fn get_recommended_pac_size(&self) -> u8 {
		// Values are taken from Table 6 of the DW3000 User manual
		match self {
			| PreambleLength::Symbols32 => 3, // 4
			| PreambleLength::Symbols64 => 0, // 8
			| PreambleLength::Symbols128 => 1, // 16
			| PreambleLength::Symbols256 => 1,
			| PreambleLength::Symbols512 => 1,
			| PreambleLength::Symbols1024 => 1,
			| PreambleLength::Symbols1536 => 1,
			| PreambleLength::Symbols2048 => 1,
			| PreambleLength::Symbols4096 => 1,
		}
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// An enum that allows the selection between different SFD sequences
///
/// The difference between the two Decawave sequences is that there are two ways
/// to enable it in the chip. Decawave will only set the DWSFD bit and
/// DecawaveAlt set the DWSFD and the \[T,R\]NSSFD bits.
pub enum SfdSequence {
	/// The standard sequence defined by the IEEE standard.
	/// Most likely the best choice for 6.8 Mbps connections.
	IeeeShort  = 0b00,
	/// A sequence defined by Decawave that is supposed to be more robust.
	/// This is an unofficial addition.
	/// Most likely the best choice for 110 Kbps connections.
	Decawave8  = 0b01,
	/// A sequence defined by Decawave that is supposed to be more robust.
	/// This is an unofficial addition.
	/// Most likely the best choice for 850 Kbps connections.
	Decawave16 = 0b10,
	/// Uses the sequence that is programmed in by the user.
	/// This is an unofficial addition.
	Ieee       = 0b11,
}

impl Default for SfdSequence {
	fn default() -> Self { SfdSequence::IeeeShort }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// All the available UWB channels.
///
/// Note that while a channel may have more bandwidth than ~900 Mhz, the DW3000
/// can only send up to ~900 Mhz
pub enum UwbChannel {
	/// Channel 5
	/// - Center frequency: 6489.6 Mhz
	/// - Bandwidth: 499.2 Mhz
	/// - Preamble Codes (16 MHz PRF) : 3, 4
	/// - Preamble Codes (64 MHz PRF) : 9, 10, 11, 12
	Channel5 = 0,
	/// Channel 9
	/// - Center frequency: 7987.2 Mhz
	/// - Bandwidth: 499.2 Mhz
	/// - Preamble Codes (16 MHz PRF) : 3, 4
	/// - Preamble Codes (64 MHz PRF) : 9, 10, 11, 12
	Channel9 = 1,
}

impl Default for UwbChannel {
	fn default() -> Self { UwbChannel::Channel5 }
}

impl UwbChannel {
	/// Gets the recommended preamble code
	pub fn get_recommended_preamble_code(&self, prf_value: PulseRepetitionFrequency) -> u8 {
		// Many have overlapping possibilities, so the numbers have been chosen so that
		// there's no overlap here
		match (self, prf_value) {
			| (UwbChannel::Channel5, PulseRepetitionFrequency::Mhz16) => 4, // ou 3
			| (UwbChannel::Channel9, PulseRepetitionFrequency::Mhz16) => 4, // ou 3
			| (UwbChannel::Channel5, PulseRepetitionFrequency::Mhz64) => 9, // ou 10,11,12
			| (UwbChannel::Channel9, PulseRepetitionFrequency::Mhz64) => 9, // ou 10,11,12
		}
	}

	/// Gets the recommended value for rf_tx_ctrl_2
	pub fn get_recommended_rf_tx_ctrl_2(&self) -> u32 {
		match self {
			| UwbChannel::Channel5 => 0x1C071134,
			| UwbChannel::Channel9 => 0x1C010034,
		}
	}

	/// Gets the recommended value for pll conf
	pub fn get_recommended_pll_conf(&self) -> u16 {
		match self {
			| UwbChannel::Channel5 => 0x1F3C,
			| UwbChannel::Channel9 => 0x0F3C,
		}
	}

	/// Gets the recommended value for pll conf
	pub fn get_recommended_dgc_lut_0(&self) -> u32 {
		match self {
			| UwbChannel::Channel5 => 0x0001C0FD,
			| UwbChannel::Channel9 => 0x0002A8FE,
		}
	}

	/// Gets the recommended value for pll conf
	pub fn get_recommended_dgc_lut_1(&self) -> u32 {
		match self {
			| UwbChannel::Channel5 => 0x0001C43E,
			| UwbChannel::Channel9 => 0x0002AC36,
		}
	}

	/// Gets the recommended value for pll conf
	pub fn get_recommended_dgc_lut_2(&self) -> u32 {
		match self {
			| UwbChannel::Channel5 => 0x0001C6BE,
			| UwbChannel::Channel9 => 0x0002A5FE,
		}
	}

	/// Gets the recommended value for pll conf
	pub fn get_recommended_dgc_lut_3(&self) -> u32 {
		match self {
			| UwbChannel::Channel5 => 0x0001C77E,
			| UwbChannel::Channel9 => 0x0002AF3E,
		}
	}

	/// Gets the recommended value for pll conf
	pub fn get_recommended_dgc_lut_4(&self) -> u32 {
		match self {
			| UwbChannel::Channel5 => 0x0001CF36,
			| UwbChannel::Channel9 => 0x0002AF7D,
		}
	}

	/// Gets the recommended value for pll conf
	pub fn get_recommended_dgc_lut_5(&self) -> u32 {
		match self {
			| UwbChannel::Channel5 => 0x0001CFB5,
			| UwbChannel::Channel9 => 0x0002AFB5,
		}
	}

	/// Gets the recommended value for pll conf
	pub fn get_recommended_dgc_lut_6(&self) -> u32 {
		match self {
			| UwbChannel::Channel5 => 0x0001CFF5,
			| UwbChannel::Channel9 => 0x0002AFB5,
		}
	}
}
