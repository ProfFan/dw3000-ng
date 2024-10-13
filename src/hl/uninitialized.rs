use core::num::Wrapping;

use crate::configs::{PhrRate, StsLen, UwbChannel};
use crate::{
    configs::{PdoaMode, PhrMode, PreambleLength, StsMode},
    ll, Config, Error, Ready, Uninitialized, DW3000,
};

use crate::{maybe_async_attr, spi_type};

#[cfg(not(feature = "async"))]
use embedded_hal::delay::DelayNs;
#[cfg(feature = "async")]
use embedded_hal_async::delay::DelayNs;

impl<SPI> DW3000<SPI, Uninitialized>
where
    SPI: spi_type::spi::SpiDevice<u8>,
{
    /// Create a new instance of `DW3000`
    ///
    /// Requires the SPI peripheral and the chip select pin that are connected
    /// to the DW3000.
    pub fn new(spi: SPI) -> Self {
        DW3000 {
            ll: ll::DW3000::new(spi),
            seq: Wrapping(0),
            state: Uninitialized,
        }
    }

    /// Read the OTP memory at the given address
    #[maybe_async_attr]
    pub async fn read_otp(&mut self, addr: u16) -> Result<u32, ll::Error<SPI>> {
        // Set OTP_MAN to 1
        self.ll.otp_cfg().write(|w| w.otp_man(1)).await?;
        // Set the 10-bit address
        self.ll.otp_addr().modify(|_, w| w.otp_addr(addr)).await?;
        // Set OTP_READ to 1
        self.ll.otp_cfg().write(|w| w.otp_read(1)).await?;
        // Read the data (32 bits)
        let data = self.ll.otp_rdata().read().await?.value();
        Ok(data)
    }

    /// Initialize the DW3000
    /// Basicaly, this is the pll configuration. We want to have a locked pll in order to provide a constant speed clock.
    /// This is important when using th clock to measure distances.
    /// At the end of this function, pll is locked and it can be checked by the bit CPLOCK in SYS_STATUS register (see state_test example)
    #[maybe_async_attr]
    pub async fn init(mut self) -> Result<DW3000<SPI, Uninitialized>, Error<SPI>> {
        // Wait for the INIT_RC state
        for _ in 0..1000 {
            if self.ll.sys_status().read().await?.rcinit() == 1 {
                break;
            }
        }
        if self.ll.sys_status().read().await?.rcinit() == 0 {
            return Err(Error::InitializationFailed);
        }

        // Try reading the device ID
        let device_id = self.ll().dev_id().read().await?;

        if device_id.ridtag() != 0xDECA || device_id.model() != 0x3 {
            #[cfg(feature = "defmt")]
            defmt::error!("ID = {}", device_id.ridtag());
            return Err(Error::InvalidConfiguration);
        }

        // Read LDO_TUNE value from OTP memory
        let ldo_tune_l = self.read_otp(0x04).await?;
        let ldo_tune_h = self.read_otp(0x05).await?;

        // Read BIASTUNE_CAL value from OTP memory (bit 16 to 20)
        let biastune_cal = self.read_otp(0x0A).await? >> 0x10 & 0x1F;

        #[cfg(feature = "defmt")]
        defmt::trace!(
            "LDO_TUNE_L = {:x}, LDO_TUNE_H = {:x}, BIASTUNE_CAL = {:x}",
            ldo_tune_l,
            ldo_tune_h,
            biastune_cal
        );

        // Set LDO_TUNE and BIASTUNE_CAL values if OTP memory is valid
        if ldo_tune_l != 0 && ldo_tune_h != 0 && (biastune_cal != 0) {
            self.ll()
                .otp_cfg()
                .write(|w| w.ldo_kick(1).bias_kick(1))
                .await?;
            self.ll()
                .bias_ctrl()
                .modify(|r, w| w.value(r.value() & 0xFFE0 | biastune_cal as u16))
                .await?
        }

        // Configuration of `XTAL_TRIM`
        self.ll.otp_cfg().modify(|_, w| w.otp_man(1)).await?;
        self.ll.otp_addr().modify(|_, w| w.otp_addr(0x1E)).await?;
        self.ll.otp_cfg().modify(|_, w| w.otp_read(1)).await?;
        let xtrim = self.ll.otp_rdata().read().await?.value() & 0x3F;

        if xtrim != 0 {
            self.ll.xtal().modify(|_, w| w.value(xtrim as u8)).await?;
        } else {
            self.ll.xtal().modify(|_, w| w.value(0x2E)).await?;
        }

        // Load the PLL code
        let pll_lock_code = self.read_otp(0x35).await?;

        if pll_lock_code != 0 {
            self.ll.pll_cc().write(|w| w.value(pll_lock_code)).await?;
        }

        Ok(DW3000 {
            ll: self.ll,
            seq: self.seq,
            state: Uninitialized,
        })
    }

    // /// Configuration of the DW3000, need to be called after an init.
    // /// This function need to be improved. TODO
    // /// There is several steps to do on this function that improve the sending and reception of a message.
    // /// Without doing this, the receiver almost never receive a frame from transmitter
    // pub fn config<DELAY>(
    //     self,
    //     config: Config,
    //     mut delay_us: DELAY,
    // ) -> Result<DW3000<SPI, Ready>, Error<SPI>>
    // where
    //     DELAY: FnMut(u32),
    // {
    //     spin_on::spin_on(self.config_async(config, |us| {
    //         delay_us(us);
    //         async {}
    //     }))
    // }

    /// Async version of the `config` function
    ///
    /// Configuration of the DW3000, need to be called after an init.
    /// This function need to be improved. TODO
    /// There is several steps to do on this function that improve the sending and reception of a message.
    /// Without doing this, the receiver almost never receive a frame from transmitter
    #[maybe_async_attr]
    pub async fn config<DELAY>(
        mut self,
        config: Config,
        mut delay_ns: DELAY,
    ) -> Result<DW3000<SPI, Ready>, Error<SPI>>
    where
        DELAY: DelayNs,
    {
        // New configuration method that match the offical driver
        let channel = config.channel;
        let mut preamble_length_actual = config.preamble_length.get_num_of_symbols();
        let tx_preamble_code = config
            .tx_preamble_code
            .unwrap_or(channel.get_recommended_preamble_code(config.pulse_repetition_frequency));
        let rx_preamble_code = config
            .rx_preamble_code
            .unwrap_or(channel.get_recommended_preamble_code(config.pulse_repetition_frequency));

        // Check if the channel is SCP or not
        let is_scp = rx_preamble_code > 24 || tx_preamble_code > 24;

        // Are we using the special PHR mode?
        let is_extended_phr = config.phr_mode == PhrMode::Extended;
        let phr_rate = config.phr_rate;
        let sts_mode = config.sts_mode;
        let sts_len = config.sts_len;
        let pdoa_mode = config.pdoa_mode;

        self.config_basic_params(
            &mut preamble_length_actual,
            is_scp,
            is_extended_phr,
            phr_rate,
            sts_mode,
            sts_len,
            pdoa_mode,
        )
        .await?;

        self.config_txrx_params(config, tx_preamble_code, rx_preamble_code)
            .await?;

        self.start_pll_calibration(config).await?;

        // wait for CPLOCK to be set
        let mut timeout = 1000;
        while self.ll.sys_status().read().await?.cplock() == 0 {
            delay_ns.delay_us(20).await;

            if self.ll.sys_status().read().await?.cplock() != 0 {
                break;
            }

            if timeout == 0 {
                #[cfg(feature = "defmt")]
                defmt::error!("PLL CPLOCK timeout");
                return Err(Error::InitializationFailed);
            }
            timeout -= 1;
        }

        // PLL is locked from here on

        self.config_set_dgc_dtune4(channel, preamble_length_actual, rx_preamble_code)
            .await?;

        // Start PGF calibration

        let ldo_ctrl_low = self.ll.ldo_ctrl().read().await?.low();
        self.ll.ldo_ctrl().modify(|_, w| w.low(0x105)).await?;

        delay_ns.delay_us(20).await;

        let pgf_cal_result = self.run_pgf_cal_async(delay_ns).await;

        self.ll
            .ldo_ctrl()
            .modify(|_, w| w.low(ldo_ctrl_low))
            .await?; // restore LDO_CTRL
        pgf_cal_result?;

        Ok(DW3000 {
            ll: self.ll,
            seq: self.seq,
            state: Ready,
        })
    }

    /// Run the PGF calibration, async version
    #[maybe_async_attr]
    async fn run_pgf_cal_async<DELAY>(&mut self, mut delay_ns: DELAY) -> Result<(), Error<SPI>>
    where
        DELAY: DelayNs,
    {
        self.ll
            .rx_cal()
            .modify(|_, w| w.comp_dly(0x2).cal_mode(1))
            .await?;

        self.ll.rx_cal().modify(|_, w| w.cal_en(1)).await?;

        let mut max_retries = 3;
        let mut success = true;
        delay_ns.delay_us(20).await;
        while self.ll.rx_cal_sts().read().await?.value() == 0 {
            max_retries -= 1;
            if max_retries == 0 {
                success = false;
                break;
            }
            delay_ns.delay_us(20).await;
        }

        if !success {
            return Err(Error::PGFCalibrationFailed);
        }

        self.ll
            .rx_cal()
            .modify(|_, w| w.cal_mode(0).cal_en(0))
            .await?;
        self.ll.rx_cal_sts().modify(|_, w| w.value(1)).await?;
        self.ll
            .rx_cal()
            .modify(|r, w| w.comp_dly(r.comp_dly() | 0x1))
            .await?;

        let rx_cal_resi = self.ll.rx_cal_resi().read().await?.value();
        let rx_cal_resq = self.ll.rx_cal_resq().read().await?.value();
        if rx_cal_resi == 0x1fffffff || rx_cal_resq == 0x1fffffff {
            return Err(Error::PGFCalibrationFailed);
        }

        Ok(())
    }

    #[maybe_async_attr]
    async fn config_set_dgc_dtune4(
        &mut self,
        channel: UwbChannel,
        preamble_length_actual: usize,
        rx_preamble_code: u8,
    ) -> Result<(), Error<SPI>> {
        if (9..=24).contains(&rx_preamble_code) {
            let dgc_otp = self.read_otp(0x20).await?;

            if dgc_otp == 0x10000240 {
                #[cfg(feature = "defmt")]
                defmt::trace!("Configuring DGC from OTP");

                self.ll.otp_cfg().modify(|_, w| w.dgc_kick(1)).await?;
                self.ll
                    .otp_cfg()
                    .modify(|_, w| w.dgc_sel(channel as u8))
                    .await?; // 0 if channel5 and 1 if channel9
            } else {
                #[cfg(feature = "defmt")]
                defmt::trace!("Configuring DGC from hardcoded values");

                self.ll
                    .dgc_lut_0()
                    .modify(|_, w| w.value(channel.get_recommended_dgc_lut_0()))
                    .await?;
                self.ll
                    .dgc_lut_1()
                    .modify(|_, w| w.value(channel.get_recommended_dgc_lut_1()))
                    .await?;
                self.ll
                    .dgc_lut_2()
                    .modify(|_, w| w.value(channel.get_recommended_dgc_lut_2()))
                    .await?;
                self.ll
                    .dgc_lut_3()
                    .modify(|_, w| w.value(channel.get_recommended_dgc_lut_3()))
                    .await?;
                self.ll
                    .dgc_lut_4()
                    .modify(|_, w| w.value(channel.get_recommended_dgc_lut_4()))
                    .await?;
                self.ll
                    .dgc_lut_5()
                    .modify(|_, w| w.value(channel.get_recommended_dgc_lut_5()))
                    .await?;
                self.ll
                    .dgc_lut_6()
                    .modify(|_, w| w.value(channel.get_recommended_dgc_lut_6()))
                    .await?;
                self.ll
                    .dgc_cfg0()
                    .modify(|_, w| w.value(0x10000240))
                    .await?;
                self.ll
                    .dgc_cfg1()
                    .modify(|_, w| w.value(0x1b6da489))
                    .await?;
            }

            self.ll.dgc_cfg().modify(|_, w| w.thr_64(0x32)).await?;
        } else {
            self.ll.dgc_cfg().modify(|_, w| w.rx_tune_en(0)).await?;
        }

        // Set DTUNE4 according to current preamble length
        if preamble_length_actual > 64 {
            self.ll.dtune4().modify(|_, w| w.dtune4(0x20)).await?;
        } else {
            self.ll.dtune4().modify(|_, w| w.dtune4(0x14)).await?;
        }
        Ok(())
    }

    #[maybe_async_attr]
    async fn start_pll_calibration(&mut self, config: Config) -> Result<(), Error<SPI>> {
        // Read the sys_state register
        let pmsc_state = self.ll.sys_state().read().await?.pmsc_state();

        // Force the PLL to unlock if it is locked
        if pmsc_state == 0x3 {
            #[cfg(feature = "defmt")]
            defmt::trace!("PLL is locked, forcing unlock");

            self.ll.clk_ctrl().modify(|_, w| w.sys_clk(0x3)).await?; // Set system to IDLERC
            self.ll.seq_ctrl().modify(|_, w| w.force2init(0x1)).await?; // Force PLL unlock
            self.ll.seq_ctrl().modify(|_, w| w.force2init(0x0)).await?; // Clear force PLL unlock

            self.ll
                .clk_ctrl()
                .modify(|_, w| {
                    w.sys_clk(0)
                        .rx_clk(0)
                        .tx_clk(0)
                        .acc_clk_en(0)
                        .cia_clk_en(0)
                        .sar_clk_en(0)
                        .acc_mclk_en(0)
                })
                .await?;
        }

        self.ll
            .rf_tx_ctrl_2()
            .modify(|_, w| w.value(config.channel.get_recommended_rf_tx_ctrl_2()))
            .await?;
        self.ll
            .pll_cfg()
            .modify(|_, w| w.value(config.channel.get_recommended_pll_conf()))
            .await?;

        self.ll.ldo_rload().modify(|_, w| w.value(0x14)).await?;

        self.ll.rf_tx_ctrl_1().modify(|_, w| w.value(0x0E)).await?;

        self.ll.pll_cal().modify(|_, w| w.use_old(0x0)).await?;
        self.ll.pll_cal().modify(|_, w| w.pll_cfg_ld(0x8)).await?;

        // CPLOCK is a write-to-clear bit, so we need to write 1 to clear it
        self.ll
            .sys_status()
            .modify(|_, w| {
                w.irqs(0)
                    .cplock(0x1)
                    .spicrce(0)
                    .aat(0)
                    .txfrb(0)
                    .txprs(0)
                    .txphs(0)
                    .txfrs(0)
            })
            .await?;

        self.ll
            .clk_ctrl()
            .modify(|_, w| {
                w.sys_clk(0b00)
                    .rx_clk(0b00)
                    .tx_clk(0b00)
                    .acc_clk_en(0b0)
                    .cia_clk_en(0b0)
                    .sar_clk_en(0b0)
                    .acc_mclk_en(0b0)
            })
            .await?;

        self.ll
            .pll_cal()
            .modify(|_, w| w.use_old(0x1).cal_en(0x1))
            .await?;

        self.ll.seq_ctrl().modify(|_, w| w.ainit2idle(1)).await?;

        // select PLL mode auto
        self.ll.clk_ctrl().modify(|_, w| w.sys_clk(0)).await?;
        // set ainit2idle
        self.ll.seq_ctrl().modify(|_, w| w.ainit2idle(1)).await?;
        Ok(())
    }

    #[maybe_async_attr]
    async fn config_txrx_params(
        &mut self,
        config: Config,
        tx_preamble_code: u8,
        rx_preamble_code: u8,
    ) -> Result<(), Error<SPI>> {
        self.ll
            .dtune0()
            .modify(|_, w| {
                w.pac(config.preamble_length.get_recommended_pac_size())
                    .dt0b4(if config.pdoa_mode == PdoaMode::Mode1 {
                        0x0
                    } else {
                        0x1
                    })
            })
            .await?;

        self.ll
            .sts_cfg()
            .modify(|_, w| w.cps_len(config.sts_len as u8 - 1))
            .await?;

        if config.preamble_length == PreambleLength::Symbols72 {
            self.ll.tx_fctrl().modify(|_, w| w.fine_plen(0x8)).await?;
        } else {
            self.ll.tx_fctrl().modify(|_, w| w.fine_plen(0x0)).await?;
        }

        self.ll.dtune3().modify(|_, w| w.value(0xAF5F35CC)).await?;

        self.ll
            .chan_ctrl()
            .modify(|_, w| {
                w.rf_chan(config.channel as u8) // 0 if channel5 and 1 if channel9
                    .sfd_type(config.sfd_sequence as u8)
                    .tx_pcode(tx_preamble_code)
                    .rx_pcode(rx_preamble_code)
            })
            .await?;

        // TXBR is set to 1 when using 6M8 data rate
        self.ll
            .tx_fctrl()
            .modify(|_, w| w.txbr(config.bitrate as u8))
            .await?;
        self.ll
            .tx_fctrl()
            .modify(|_, w| w.txpsr(config.preamble_length as u8))
            .await?;

        self.ll
            .rx_sfd_toc()
            .modify(|_, w| w.value(config.sfd_timeout as u16))
            .await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    #[maybe_async_attr]
    async fn config_basic_params(
        &mut self,
        preamble_length_actual: &mut usize,
        is_scp: bool,
        is_extended_phr: bool,
        phr_rate: PhrRate,
        sts_mode: StsMode,
        sts_len: StsLen,
        pdoa_mode: PdoaMode,
    ) -> Result<(), Error<SPI>> {
        self.ll
            .sys_cfg()
            .modify(|_, w| w.phr_mode(is_extended_phr as u8))
            .await?;
        self.ll
            .sys_cfg()
            .modify(|_, w| w.phr_6m8(phr_rate as u8))
            .await?;
        self.ll
            .sys_cfg()
            .modify(|_, w| w.cp_spc(sts_mode as u8))
            .await?;
        self.ll
            .sys_cfg()
            .modify(|_, w| w.pdoa_mode(pdoa_mode as u8))
            .await?;
        self.ll.sys_cfg().modify(|_, w| w.cp_sdc(0)).await?;

        // SCP Mode specific configuration
        if is_scp {
            // TODO: We probably need to adjust our sleep mode accordingly
            //
            // But we don't have a sleep mode yet
            self.ll
                .otp_cfg()
                .modify(|_, w| w.ops_sel(0x1).ops_kick(0x1))
                .await?;
            self.ll
                .ip_conf_lo()
                .write(|w| w.ip_ntm(0x6).ip_scp(0x3))
                .await?;
            self.ll.ip_conf_hi().write(|w| w.value(0)).await?;
            self.ll
                .sts_conf_0()
                .write(|w| w.sts_ntm(0xA).sts_scp(0x5A).sts_rtm(0xC))
                .await?;
            self.ll
                .sts_conf_1()
                .modify(|_, w| {
                    w.res_b0(0x9D)
                        .fp_agreed_en(0)
                        .sts_cq_en(0)
                        .sts_pgr_en(0)
                        .sts_ss_en(0)
                })
                .await?;
        } else {
            if sts_mode != StsMode::StsModeOff {
                #[cfg(feature = "defmt")]
                defmt::trace!("STS Mode is enabled, calculating STS_MNTH");

                // Configure CIA STS minimum thresholds for security
                let mut sts_mnth = sts_len.get_sts_mnth(pdoa_mode);

                if sts_mnth > 0x7F {
                    #[cfg(feature = "defmt")]
                    defmt::warn!("STS_MNTH is too high, setting to 0x7F");
                    sts_mnth = 0x7F;
                }

                *preamble_length_actual += sts_len.get_sts_length() as usize;
                self.ll
                    .sts_conf_0()
                    .modify(|_, w| w.sts_rtm(sts_mnth as u8))
                    .await?;
                self.ll
                    .sts_conf_1()
                    .modify(|_, w| {
                        w.res_b0(0x94)
                            .fp_agreed_en(0)
                            .sts_cq_en(0)
                            .sts_pgr_en(0)
                            .sts_ss_en(0)
                    })
                    .await?;
            }

            if *preamble_length_actual > 256 {
                // TODO: Need custom sleep kick mode for long preamble
                // This is DWT_ALT_OPS | DWT_SEL_OPS0 in official driver
                #[cfg(feature = "defmt")]
                defmt::trace!("Long preamble detected, setting OTP to DWT_OPSET_LONG");
                self.ll
                    .otp_cfg()
                    .modify(|_, w| w.ops_sel(0x0).ops_kick(1))
                    .await?; // DWT_OPSET_LONG
            } else {
                self.ll
                    .otp_cfg()
                    .modify(|_, w| w.ops_sel(0x2).ops_kick(1))
                    .await?; // DWT_OPSET_SHORT
            }
        }
        Ok(())
    }
}
