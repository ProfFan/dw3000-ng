//! Enumeration for fast commands

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