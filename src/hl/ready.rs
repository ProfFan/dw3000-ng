#![allow(unused_imports)]

use core::num::Wrapping;

use byte::BytesExt as _;
use embedded_hal::{blocking::spi, digital::v2::OutputPin};

use super::AutoDoubleBufferReceiving;
use crate::{
    configs::SfdSequence, time::Instant, Config, Error, FastCommand, Ready, Sending,
    SingleBufferReceiving, Sleeping, DW3000,
};

use smoltcp::wire::{Ieee802154Address, Ieee802154Frame, Ieee802154Pan, Ieee802154Repr};

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
    ActiveLow = 0,
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
        pan_id: Ieee802154Pan,
        addr: Ieee802154Address,
    ) -> Result<(), Error<SPI, CS>> {
        let Ieee802154Address::Short(short_addr) = addr else {
            return Err(Error::InvalidConfiguration);
        };

        self.ll.panadr().write(|w| {
            w.pan_id(pan_id.0)
                .short_addr(u16::from_be_bytes(short_addr))
        })?;

        Ok(())
    }

    /// Send an raw UWB PHY frame
    ///
    /// The `data` argument is wrapped into an raw UWB PHY frame.
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
    pub fn send_raw(
        mut self,
        data: &[u8],
        send_time: SendTime,
        config: &Config,
    ) -> Result<DW3000<SPI, CS, Sending>, Error<SPI, CS>> {
        // Clear event counters
        self.ll.evc_ctrl().write(|w| w.evc_clr(0b1))?;
        // while self.ll.evc_ctrl().read()?.evc_clr() == 0b1 {}

        // (Re-)Enable event counters
        self.ll.evc_ctrl().write(|w| w.evc_en(0b1))?;
        // while self.ll.evc_ctrl().read()?.evc_en() == 0b1 {}

        // self.ll.clk_ctrl().modify(|_, w| w.tx_clk(0b10))?;

        // Prepare transmitter
        let mut len: usize = 0;
        self.ll.tx_buffer().write(|w| {
            let result = w.data().write(&mut len, data);

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
            w.txflen(txflen) // data length + two-octet CRC
                .txbr(config.bitrate as u8) // configured bitrate
                .tr(config.ranging_enable as u8) // configured ranging bit
                .txb_offset(txb_offset_errata) // no offset in TX_BUFFER
                .txpsr(config.preamble_length as u8) // configure preamble length
                .fine_plen(0) // Not implemented, replacing txpsr
        })?;

        match send_time {
            SendTime::Delayed(time) => {
                // Put the time into the delay register
                // By setting this register, the chip knows to delay before transmitting
                self.ll
                    .dx_time()
                    .modify(|_, w| // 32-bits value of the most significant bits
                    w.value( (time.value() >> 8) as u32 ))?;
                self.fast_cmd(FastCommand::CMD_DTX)?;
            }
            SendTime::OnSync => {
                self.ll.ec_ctrl().modify(|_, w| w.ostr_mode(1))?;
                self.ll.ec_ctrl().modify(|_, w| w.osts_wait(33))?;
            }
            SendTime::Now => self.fast_cmd(FastCommand::CMD_TX)?,
        }

        Ok(DW3000 {
            ll: self.ll,
            seq: self.seq,
            state: Sending { finished: false },
        })
    }

    /// Send an IEEE 802.15.4 MAC frame
    ///
    /// The `frame` argument is an IEEE 802.15.4 MAC frame and sent to `destination`.
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
    #[inline]
    pub fn send_frame<T>(
        mut self,
        frame: Ieee802154Frame<T>,
        send_time: SendTime,
        config: Config,
    ) -> Result<DW3000<SPI, CS, Sending>, Error<SPI, CS>>
    where
        T: AsRef<[u8]>,
    {
        // Clear event counters
        self.ll.evc_ctrl().write(|w| w.evc_clr(0b1))?;
        while self.ll.evc_ctrl().read()?.evc_clr() == 0b1 {}

        // (Re-)Enable event counters
        self.ll.evc_ctrl().write(|w| w.evc_en(0b1))?;
        while self.ll.evc_ctrl().read()?.evc_en() == 0b1 {}

        self.ll.clk_ctrl().modify(|_, w| w.tx_clk(0b10))?;

        // Prepare transmitter
        let buf = frame.into_inner();
        let mut len = 0;
        self.ll.tx_buffer().write(|w| {
            let result = w.data();
            len = buf.as_ref().len();
            result[0..len].copy_from_slice(buf.as_ref());

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
            w.txflen(txflen) // data length + two-octet CRC
                .txbr(config.bitrate as u8) // configured bitrate
                .tr(config.ranging_enable as u8) // configured ranging bit
                .txb_offset(txb_offset_errata) // no offset in TX_BUFFER
                .txpsr(config.preamble_length as u8) // configure preamble length
                .fine_plen(0) // Not implemented, replacing txpsr
        })?;

        match send_time {
            SendTime::Delayed(time) => {
                // Put the time into the delay register
                // By setting this register, the chip knows to delay before transmitting
                self.ll
                    .dx_time()
                    .modify(|_, w| // 32-bits value of the most significant bits
                    w.value( (time.value() >> 8) as u32 ))?;
                self.fast_cmd(FastCommand::CMD_DTX)?;
            }
            SendTime::OnSync => {
                self.ll.ec_ctrl().modify(|_, w| w.ostr_mode(1))?;
                self.ll.ec_ctrl().modify(|_, w| w.osts_wait(33))?;
            }
            SendTime::Now => self.fast_cmd(FastCommand::CMD_TX)?,
        }

        Ok(DW3000 {
            ll: self.ll,
            seq: self.seq,
            state: Sending { finished: false },
        })
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
    #[inline(always)]
    pub fn send(
        mut self,
        data: &[u8],
        send_time: SendTime,
        config: Config,
    ) -> Result<DW3000<SPI, CS, Sending>, Error<SPI, CS>> {
        // Clear event counters
        self.ll.evc_ctrl().write(|w| w.evc_clr(0b1))?;
        while self.ll.evc_ctrl().read()?.evc_clr() == 0b1 {}

        // (Re-)Enable event counters
        self.ll.evc_ctrl().write(|w| w.evc_en(0b1))?;
        while self.ll.evc_ctrl().read()?.evc_en() == 0b1 {}

        self.ll.clk_ctrl().modify(|_, w| w.tx_clk(0b10))?;

        let seq = self.seq.0;
        self.seq += Wrapping(1);

        let frame_repr = Ieee802154Repr {
            frame_type: smoltcp::wire::Ieee802154FrameType::Data,
            frame_version: smoltcp::wire::Ieee802154FrameVersion::Ieee802154_2006,
            security_enabled: false,
            sequence_number: Some(seq),
            frame_pending: false,
            ack_request: false,
            pan_id_compression: true,
            dst_addr: Some(Ieee802154Address::BROADCAST),
            src_addr: Some(self.get_address()?.1),
            src_pan_id: Some(self.get_address()?.0),
            dst_pan_id: None,
        };

        // Prepare transmitter
        let mut len = 0;
        self.ll.tx_buffer().write(|w| {
            let result = w.data();

            let mut frame = Ieee802154Frame::new_unchecked(&mut result[0..]);
            frame_repr.emit(&mut frame);

            // copy data
            result[frame_repr.buffer_len()..frame_repr.buffer_len() + data.len()]
                .copy_from_slice(data);

            // footer
            result[frame_repr.buffer_len() + data.len()] = 0x00;

            len = frame_repr.buffer_len() + data.len() + 2;

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
            w.txflen(txflen) // data length + two-octet CRC
                .txbr(config.bitrate as u8) // configured bitrate
                .tr(config.ranging_enable as u8) // configured ranging bit
                .txb_offset(txb_offset_errata) // no offset in TX_BUFFER
                .txpsr(config.preamble_length as u8) // configure preamble length
                .fine_plen(0) // Not implemented, replacing txpsr
        })?;

        match send_time {
            SendTime::Delayed(time) => {
                // Put the time into the delay register
                // By setting this register, the chip knows to delay before transmitting
                self.ll
                    .dx_time()
                    .modify(|_, w| // 32-bits value of the most significant bits
                    w.value( (time.value() >> 8) as u32 ))?;
                self.fast_cmd(FastCommand::CMD_DTX)?;
            }
            SendTime::OnSync => {
                self.ll.ec_ctrl().modify(|_, w| w.ostr_mode(1))?;
                self.ll.ec_ctrl().modify(|_, w| w.osts_wait(33))?;
            }
            SendTime::Now => self.fast_cmd(FastCommand::CMD_TX)?,
        }

        Ok(DW3000 {
            ll: self.ll,
            seq: self.seq,
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
            ll: self.ll,
            seq: self.seq,
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

    /// Disable the SPIRDY interrupt flag
    pub fn disable_spirdy_interrupt(&mut self) -> Result<(), Error<SPI, CS>> {
        self.ll.sys_enable().modify(|_, w| w.spirdy_en(0b0))?;
        Ok(())
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
            w
                // .rxprd_en(0b1)
                //     .rxsfdd_en(0b1)
                //     .rxphd_en(0b1)
                .rxphe_en(0b1)
                .rxfr_en(0b1)
                // .rxfcg_en(0b1)
                .rxfce_en(0b1)
                .rxrfsl_en(0b1)
                .rxfto_en(0b1)
                .rxovrr_en(0b1)
                .rxpto_en(0b1)
                .rxsto_en(0b1)
                // .rxprej_en(0b1)
        })?;
        Ok(())
    }

    /// Disables all interrupts
    pub fn disable_interrupts(&mut self) -> Result<(), Error<SPI, CS>> {
        self.ll.sys_enable().write(|w| w)?;
        Ok(())
    }

    /// GPIO SECTION, gpios seems to have a problem with its register.
    /// Init GPIO WRT Config
    pub fn gpio_config(&mut self, config: ConfigGPIOs) -> Result<(), Error<SPI, CS>> {
        self.gpio_config_clocks()?;

        self.ll.gpio_pull_en().modify(|_, w| {
            w.mgpen0(config.enabled[0])
                .mgpen1(config.enabled[1])
                .mgpen2(config.enabled[2])
                .mgpen3(config.enabled[3])
                .mgpen4(config.enabled[4])
                .mgpen5(config.enabled[5])
                .mgpen6(config.enabled[6])
                .mgpen7(config.enabled[7])
                .mgpen8(config.enabled[8])
        })?;

        self.ll.gpio_mode().modify(|_, w| {
            w.msgp0(0x0)
                .msgp1(0x0)
                .msgp2(0x0)
                .msgp3(0x0)
                .msgp4(0x0)
                .msgp5(0x0)
                .msgp6(0x0)
                .msgp7(0x0)
                .msgp8(0x0)
        })?;
        self.ll.gpio_mode().modify(|_, w| {
            w.msgp0(config.mode[0])
                .msgp1(config.mode[1])
                .msgp2(config.mode[2])
                .msgp3(config.mode[3])
                .msgp4(config.mode[4])
                .msgp5(config.mode[5])
                .msgp6(config.mode[6])
                .msgp7(config.mode[7])
                .msgp8(config.mode[8])
        })?;

        self.ll.gpio_dir().modify(|_, w| {
            w.gpd0(config.gpio_dir[0])
                .gpd1(config.gpio_dir[1])
                .gpd2(config.gpio_dir[2])
                .gpd3(config.gpio_dir[3])
                .gpd4(config.gpio_dir[4])
                .gpd5(config.gpio_dir[5])
                .gpd6(config.gpio_dir[6])
                .gpd7(config.gpio_dir[7])
                .gpd8(config.gpio_dir[8])
        })?;

        self.ll.gpio_out().modify(|_, w| {
            w.gop0(config.output[0])
                .gop1(config.output[1])
                .gop2(config.output[2])
                .gop3(config.output[3])
                .gop4(config.output[4])
                .gop5(config.output[5])
                .gop6(config.output[6])
                .gop7(config.output[7])
                .gop8(config.output[8])
        })?;

        Ok(())
    }

    /// Enable gpios clocks
    pub fn gpio_config_clocks(&mut self) -> Result<(), Error<SPI, CS>> {
        self.ll.clk_ctrl().modify(|_, w| {
            w.gpio_clk_en(0b1)
                .gpio_dclk_en(0b1)
                .gpio_drst_n(0b1)
                .lp_clk_en(0b1)
        })?;

        self.ll
            .led_ctrl()
            .modify(|_, w| w.blink_en(0b1).blink_tim(0x10).force_trig(0x0))?;

        Ok(())
    }

    /// Enables single pin
    pub fn gpio_config_enable(&mut self, pin: u8, enable: u8) -> Result<(), Error<SPI, CS>> {
        self.ll.gpio_pull_en().modify(|_, w| match pin {
            0 => w.mgpen0(enable),
            1 => w.mgpen1(enable),
            2 => w.mgpen2(enable),
            3 => w.mgpen3(enable),
            4 => w.mgpen4(enable),
            5 => w.mgpen5(enable),
            6 => w.mgpen6(enable),
            7 => w.mgpen7(enable),
            8 => w.mgpen8(enable),
            _ => w,
        })?;
        Ok(())
    }

    /// Configures mode for a single pin
    pub fn gpio_config_mode(&mut self, pin: u8, mode: u8) -> Result<(), Error<SPI, CS>> {
        self.ll.gpio_mode().modify(|_, w| match pin {
            0 => w.msgp0(mode),
            1 => w.msgp1(mode),
            2 => w.msgp2(mode),
            3 => w.msgp3(mode),
            4 => w.msgp4(mode),
            5 => w.msgp5(mode),
            6 => w.msgp6(mode),
            7 => w.msgp7(mode),
            8 => w.msgp8(mode),
            _ => w,
        })?;
        Ok(())
    }

    /// Configures direction for a single pin
    pub fn gpio_config_dir(&mut self, pin: u8, dir: u8) -> Result<(), Error<SPI, CS>> {
        self.ll.gpio_dir().modify(|_, w| match pin {
            0 => w.gpd0(dir),
            1 => w.gpd1(dir),
            2 => w.gpd2(dir),
            3 => w.gpd3(dir),
            4 => w.gpd4(dir),
            5 => w.gpd5(dir),
            6 => w.gpd6(dir),
            7 => w.gpd7(dir),
            8 => w.gpd8(dir),
            _ => w,
        })?;
        Ok(())
    }

    /// Configures output for a single pin
    pub fn gpio_config_out(&mut self, pin: u8, output: u8) -> Result<(), Error<SPI, CS>> {
        self.ll.gpio_out().modify(|_, w| match pin {
            0 => w.gop0(output),
            1 => w.gop1(output),
            2 => w.gop2(output),
            3 => w.gop3(output),
            4 => w.gop4(output),
            5 => w.gop5(output),
            6 => w.gop6(output),
            7 => w.gop7(output),
            8 => w.gop8(output),
            _ => w,
        })?;
        Ok(())
    }

    /// Returns GPIO config
    pub fn get_gpio_config(&mut self) -> Result<ConfigGPIOs, Error<SPI, CS>> {
        let enabled = self.get_gpio_enabled()?;
        let mode = self.get_gpio_mode()?;
        let gpio_dir = self.get_gpio_dir()?;
        let output = self.get_gpio_out()?;

        Ok(ConfigGPIOs {
            enabled,
            mode,
            gpio_dir,
            output,
        })
    }

    /// Returns current gpio enable state
    pub fn get_gpio_enabled(&mut self) -> Result<[u8; 9], Error<SPI, CS>> {
        let gpio_pull_en = self.ll.gpio_pull_en().read()?;
        let enabled: [u8; 9] = [
            gpio_pull_en.mgpen0(),
            gpio_pull_en.mgpen1(),
            gpio_pull_en.mgpen2(),
            gpio_pull_en.mgpen3(),
            gpio_pull_en.mgpen4(),
            gpio_pull_en.mgpen5(),
            gpio_pull_en.mgpen6(),
            gpio_pull_en.mgpen7(),
            gpio_pull_en.mgpen8(),
        ];

        Ok(enabled)
    }

    /// Returns current gpio pin mode
    pub fn get_gpio_mode(&mut self) -> Result<[u8; 9], Error<SPI, CS>> {
        let gpio_mode = self.ll.gpio_mode().read()?;
        let mode: [u8; 9] = [
            gpio_mode.msgp0(),
            gpio_mode.msgp1(),
            gpio_mode.msgp2(),
            gpio_mode.msgp3(),
            gpio_mode.msgp4(),
            gpio_mode.msgp5(),
            gpio_mode.msgp6(),
            gpio_mode.msgp7(),
            gpio_mode.msgp8(),
        ];

        Ok(mode)
    }

    /// Returns current gpio dir
    pub fn get_gpio_dir(&mut self) -> Result<[u8; 9], Error<SPI, CS>> {
        let gpio_direction = self.ll.gpio_dir().read()?;
        let gpio_dir = [
            gpio_direction.gpd0(),
            gpio_direction.gpd1(),
            gpio_direction.gpd2(),
            gpio_direction.gpd3(),
            gpio_direction.gpd4(),
            gpio_direction.gpd5(),
            gpio_direction.gpd6(),
            gpio_direction.gpd7(),
            gpio_direction.gpd8(),
        ];

        Ok(gpio_dir)
    }

    /// Returns current output
    pub fn get_gpio_out(&mut self) -> Result<[u8; 9], Error<SPI, CS>> {
        let gpio_out = self.ll.gpio_out().read()?;
        let output = [
            gpio_out.gop0(),
            gpio_out.gop1(),
            gpio_out.gop2(),
            gpio_out.gop3(),
            gpio_out.gop4(),
            gpio_out.gop5(),
            gpio_out.gop6(),
            gpio_out.gop7(),
            gpio_out.gop8(),
        ];

        Ok(output)
    }

    /// Returns current raw state / input
    pub fn get_gpio_raw_state(&mut self) -> Result<[u8; 9], Error<SPI, CS>> {
        let gpio_raw = self.ll.gpio_raw().read()?;
        let raw = [
            gpio_raw.grawp0(),
            gpio_raw.grawp1(),
            gpio_raw.grawp2(),
            gpio_raw.grawp3(),
            gpio_raw.grawp4(),
            gpio_raw.grawp5(),
            gpio_raw.grawp6(),
            gpio_raw.grawp7(),
            gpio_raw.grawp8(),
        ];

        Ok(raw)
    }
}

/// General confirugation for GPIO
#[derive(Debug)]
pub struct ConfigGPIOs {
    /// Enables (1) or disables (0) pins
    pub enabled: [u8; 9],
    /// Pin mode
    pub mode: [u8; 9],
    /// Set GPIO pins as input (1) or output (0)
    pub gpio_dir: [u8; 9],
    /// Set GPIO high (1) or low (0)
    pub output: [u8; 9],
}
impl Default for ConfigGPIOs {
    fn default() -> Self {
        ConfigGPIOs {
            enabled: [1, 1, 1, 1, 1, 1, 1, 1, 1],
            mode: [0, 0, 0, 0, 0, 0, 0, 0, 0],
            gpio_dir: [1, 1, 1, 1, 1, 1, 1, 1, 1],
            output: [0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}
impl ConfigGPIOs {
    /// Disables all 4 leds
    pub fn disable_led() -> Self {
        ConfigGPIOs {
            enabled: [1, 1, 0, 0, 1, 1, 1, 1, 1],
            mode: [0, 0, 1, 1, 0, 0, 0, 0, 0],
            gpio_dir: [1, 1, 1, 1, 1, 1, 1, 1, 1],
            output: [0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
    /// Enables only RX and TX led
    pub fn enable_led() -> Self {
        ConfigGPIOs {
            enabled: [1, 1, 1, 1, 1, 1, 1, 1, 1],
            mode: [0, 0, 1, 1, 0, 0, 0, 0, 0],
            gpio_dir: [1, 1, 1, 1, 1, 1, 1, 1, 1],
            output: [0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
    /// Set everything to 0
    pub fn all_0() -> Self {
        ConfigGPIOs {
            enabled: [0, 0, 0, 0, 0, 0, 0, 0, 0],
            mode: [0, 0, 0, 0, 0, 0, 0, 0, 0],
            gpio_dir: [0, 0, 0, 0, 0, 0, 0, 0, 0],
            output: [0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
    /// Set everything to 1
    pub fn all_1() -> Self {
        ConfigGPIOs {
            enabled: [1, 1, 1, 1, 1, 1, 1, 1, 1],
            mode: [1, 1, 1, 1, 1, 1, 1, 1, 1],
            gpio_dir: [1, 1, 1, 1, 1, 1, 1, 1, 1],
            output: [1, 1, 1, 1, 1, 1, 1, 1, 1],
        }
    }
    /// Custom config for debug
    pub fn custom() -> Self {
        ConfigGPIOs {
            enabled: [1, 1, 1, 1, 0, 0, 0, 0, 0],
            mode: [0, 0, 0, 0, 0, 0, 0, 0, 0],
            gpio_dir: [1, 1, 0, 0, 0, 0, 0, 0, 0],
            output: [0, 0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}
