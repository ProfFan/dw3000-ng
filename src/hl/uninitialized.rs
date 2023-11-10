use core::num::Wrapping;

use embedded_hal::{blocking::spi, digital::v2::OutputPin};

use crate::{ll, Config, Error, Ready, Uninitialized, DW3000};
//use rtt_target::{rprintln};

impl<SPI, CS> DW3000<SPI, CS, Uninitialized>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
{
    /// Create a new instance of `DW3000`
    ///
    /// Requires the SPI peripheral and the chip select pin that are connected
    /// to the DW3000.
    pub fn new(spi: SPI, chip_select: CS) -> Self {
        DW3000 {
            ll: ll::DW3000::new(spi, chip_select),
            seq: Wrapping(0),
            state: Uninitialized,
        }
    }

    /// Initialize the DW3000
    /// Basicaly, this is the pll configuration. We want to have a locked pll in order to provide a constant speed clock.
    /// This is important when using th clock to measure distances.
    /// At the end of this function, pll is locked and it can be checked by the bit CPLOCK in SYS_STATUS register (see state_test example)
    pub fn init(mut self) -> Result<DW3000<SPI, CS, Uninitialized>, Error<SPI, CS>> {
        // Wait for the IDLE_RC state
        while self.ll.sys_status().read()?.rcinit() == 0 {}
        // select PLL mode auto
        self.ll.clk_ctrl().modify(|_, w| w.sys_clk(0))?;
        // set ainit2idle
        self.ll.seq_ctrl().modify(|_, w| w.ainit2idle(1))?;
        // wait for CPLOCK to be set
        while self.ll.sys_status().read()?.cplock() == 0 {}

        // Configuration du xtal_trim
        self.ll.otp_cfg().modify(|_, w| w.otp_man(1))?;
        self.ll.otp_addr().modify(|_, w| w.otp_addr(0x1E))?;
        self.ll.otp_cfg().modify(|_, w| w.otp_read(1))?;
        let xtrim = self.ll.otp_rdata().read()?.value() & 0x7F;

        if xtrim == 0 {
            self.ll.xtal().modify(|_, w| w.value(0x2E))?;
        }

        self.ll.xtal().modify(|_, w| w.value(xtrim as u8))?;

        Ok(DW3000 {
            ll: self.ll,
            seq: self.seq,
            state: Uninitialized,
        })
    }

    /// Configuration of the DW3000, need to be called after an init.
    /// This function need to be improved. TODO
    /// There is several steps to do on this function that improve the sending and reception of a message.
    /// Without doing this, the receiver almost never receive a frame form transmitter
    /// FIRST STEP : configuration depending on CONFIG chosen. Lot of register all around the datasheet can be changed in order to improve the signal
    /// Some register needs to be changed without a lot of explanation so we tried to gather all of them in this function
    pub fn config(mut self, config: Config) -> Result<DW3000<SPI, CS, Ready>, Error<SPI, CS>> {
        //Configuration du sys_cfg
        self.ll.sys_cfg().modify(|_, w| w.phr_mode(0))?;
        self.ll.sys_cfg().modify(|_, w| w.phr_6m8(0))?;
        self.ll.sys_cfg().modify(|_, w| w.cp_spc(0))?;
        self.ll.sys_cfg().modify(|_, w| w.pdoa_mode(0))?;
        self.ll.sys_cfg().modify(|_, w| w.cp_sdc(0))?;

        self.ll.otp_cfg().modify(|_, w| w.ops_sel(0x2))?;
        self.ll.otp_cfg().modify(|_, w| w.ops_kick(0b1))?;

        self.ll
            .dtune0()
            .modify(|_, w| w.pac(config.preamble_length.get_recommended_pac_size()))?;

        self.ll
            .sts_cfg()
            .modify(|_, w| w.cps_len(config.sts_len as u8 - 1))?;

        self.ll.tx_fctrl().modify(|_, w| w.fine_plen(0x0))?;

        self.ll.dtune3().modify(|_, w| w.value(0xAF5F584C))?;

        self.ll.chan_ctrl().modify(|_, w| {
            w.rf_chan(config.channel as u8) // 0 if channel5 and 1 if channel9
                .sfd_type(config.sfd_sequence as u8)
                .tx_pcode(
                    // set the PRF for transmitter
                    config
                        .channel
                        .get_recommended_preamble_code(config.pulse_repetition_frequency),
                )
                .rx_pcode(
                    // set the PRF for receiver
                    config
                        .channel
                        .get_recommended_preamble_code(config.pulse_repetition_frequency),
                )
        })?;

        self.ll.tx_fctrl().modify(|_, w| w.txbr(0x1))?;
        self.ll.tx_fctrl().modify(|_, w| w.txpsr(0x5))?;

        self.ll
            .rx_sfd_toc()
            .modify(|_, w| w.value(config.sfd_timeout as u16))?;

        self.ll
            .rf_tx_ctrl_2()
            .modify(|_, w| w.value(config.channel.get_recommended_rf_tx_ctrl_2()))?;
        self.ll
            .pll_cfg()
            .modify(|_, w| w.value(config.channel.get_recommended_pll_conf()))?;

        self.ll.ldo_rload().modify(|_, w| w.value(0x14))?;

        self.ll.rf_tx_ctrl_1().modify(|_, w| w.value(0x0E))?;

        // A VERIFIER !
        self.ll.pll_cal().modify(|_, w| w.use_old(0x0))?;
        self.ll.pll_cal().modify(|_, w| w.pll_cfg_ld(0x8))?;
        self.ll.pll_cal().modify(|_, w| w.cal_en(0x0))?;

        self.ll.seq_ctrl().modify(|_, w| w.ainit2idle(1))?;

        self.ll.clk_ctrl().modify(|_, w| {
            w.sys_clk(0b00)
                .rx_clk(0b00)
                .tx_clk(0b00)
                .acc_clk_en(0b0)
                .cia_clk_en(0b0)
                .sar_clk_en(0b0)
                .acc_mclk_en(0b0)
        })?;

        if config
            .channel
            .get_recommended_preamble_code(config.pulse_repetition_frequency)
            >= 9
            && config
                .channel
                .get_recommended_preamble_code(config.pulse_repetition_frequency)
                <= 24
        {
            self.ll.otp_cfg().modify(|_, w| w.otp_man(1))?;
            self.ll.otp_addr().modify(|_, w| w.otp_addr(0x20))?;
            self.ll.otp_cfg().modify(|_, w| w.otp_read(1))?;
            let dgc_otp = self.ll.otp_rdata().read()?.value();

            if dgc_otp == 0x10000240 {
                self.ll.otp_cfg().modify(|_, w| w.dgc_kick(1))?;
                self.ll.otp_cfg().modify(|_, w| w.dgc_sel(0))?;
            } else {
                self.ll
                    .dgc_lut_0()
                    .modify(|_, w| w.value(config.channel.get_recommended_dgc_lut_0()))?;
                self.ll
                    .dgc_lut_1()
                    .modify(|_, w| w.value(config.channel.get_recommended_dgc_lut_1()))?;
                self.ll
                    .dgc_lut_2()
                    .modify(|_, w| w.value(config.channel.get_recommended_dgc_lut_2()))?;
                self.ll
                    .dgc_lut_3()
                    .modify(|_, w| w.value(config.channel.get_recommended_dgc_lut_3()))?;
                self.ll
                    .dgc_lut_4()
                    .modify(|_, w| w.value(config.channel.get_recommended_dgc_lut_4()))?;
                self.ll
                    .dgc_lut_5()
                    .modify(|_, w| w.value(config.channel.get_recommended_dgc_lut_5()))?;
                self.ll
                    .dgc_lut_6()
                    .modify(|_, w| w.value(config.channel.get_recommended_dgc_lut_6()))?;
                self.ll.dgc_cfg0().modify(|_, w| w.value(0x10000240))?;
                self.ll.dgc_cfg1().modify(|_, w| w.value(0x1b6da489))?;
            }

            self.ll.dgc_cfg().modify(|_, w| w.thr_64(0x32))?;
        } else {
            self.ll.dgc_cfg().modify(|_, w| w.rx_tune_en(0))?;
        }

        let val = self.ll.ldo_ctrl().read()?.value();
        self.ll.ldo_ctrl().modify(|_, w| w.value(0x105))?;

        self.ll
            .rx_cal()
            .modify(|_, w| w.comp_dly(0x2).cal_mode(1))?;

        self.ll.rx_cal().modify(|_, w| w.cal_en(1))?;

        while self.ll.rx_cal_sts().read()?.value() == 0 {}

        self.ll.rx_cal().modify(|_, w| w.cal_mode(0).cal_en(0))?;
        self.ll.rx_cal_sts().modify(|_, w| w.value(1))?;

        if self.ll.rx_cal_resi().read()?.value() == 0x1fffffff
            || self.ll.rx_cal_resq().read()?.value() == 0x1fffffff
        {
            return Err(Error::InvalidConfiguration);
        }
        self.ll.ldo_ctrl().modify(|_, w| w.value(val))?;

        Ok(DW3000 {
            ll: self.ll,
            seq: self.seq,
            state: Ready,
        })
    }
}
