#![no_main]
#![no_std]

// crates de gestion des messages de debug
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use cortex_m_rt::entry;
use stm32f1xx_hal::{
	delay::Delay,
	pac,
	prelude::*,
	spi::{Mode, Phase, Polarity, Spi},
};
use embedded_hal::{blocking::spi, digital::v2::OutputPin};
use dw3000::{hl, RxConfig, Config};

use core::mem::size_of;

fn check_states<SPI, CS, State>(
	dw3000: &mut hl::DW1000<SPI, CS, State>,
) -> Result<(), hl::Error<SPI, CS>>
where
	SPI: spi::Transfer<u8> + spi::Write<u8>,
	CS: OutputPin,
	State: hl::Awake,
{
	if dw3000.init_rc_passed()? {
		rprintln!("Après la fonction new, on est dans l'état INIT_RC (rcinit = 1)");
	}
	if dw3000.idle_rc_passed()? {
		rprintln!("Après la fonction new, on est dans l'état IDLE_RC (spirdy = 1)");
	}
	if dw3000.idle_pll_passed()? {
		rprintln!("Après la fonction new, on est dans l'état IDLE_PLL (cpclock = 1)");
	}
	rprintln!(
		"Après la fonction new, on est dans l'état {:#x?}\n\n",
		dw3000.state()?
	);
	Ok(())
}

#[entry]
fn main() -> ! {
	rtt_init_print!();
	rprintln!("Coucou copain !\n\n");

	/******************************************************* */
	/************       CONFIGURATION DE BASE     ********** */
	/******************************************************* */

	// Get access to the device specific peripherals from the peripheral access
	// crate
	let dp = pac::Peripherals::take().unwrap();
	let cp = cortex_m::Peripherals::take().unwrap();

	// Take ownership over the raw flash and rcc devices and convert them into the
	// corresponding HAL structs
	let mut flash = dp.FLASH.constrain();
	let mut rcc = dp.RCC.constrain();
	let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

	let clocks = rcc
		.cfgr
		.use_hse(8.mhz())
		.sysclk(36.mhz())
		.freeze(&mut flash.acr);

	let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
	let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

	/***************************************************** */
	/************       CONFIGURATION DU SPI       ******* */
	/***************************************************** */

	let pins = (
		gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
		gpioa.pa6.into_floating_input(&mut gpioa.crl),
		gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
	);

	let cs = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);

	let spi_mode = Mode {
		polarity: Polarity::IdleLow,
		phase:    Phase::CaptureOnFirstTransition,
	};
	let spi = Spi::spi1(
		dp.SPI1,
		pins,
		&mut afio.mapr,
		spi_mode,
		100.khz(),
		clocks,
		&mut rcc.apb2,
	);

	/****************************************************** */
	/*****       CONFIGURATION DU RESET du DW3000   ******* */
	/****************************************************** */

	// NEW
	let mut delay = Delay::new(cp.SYST, clocks);

	let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);

	// UWB module reset
	rst_n.set_low().unwrap();
	rst_n.set_high().unwrap();

	/****************************************************** */
	/*********       CONFIGURATION du DW3000       ******** */
	/****************************************************** */

	let mut dw3000 = hl::DW1000::new(spi, cs);

	check_states(&mut dw3000).unwrap();
	delay.delay_ms(1000u16);

	check_states(&mut dw3000).unwrap();

	// activation de la calibration auto
	// dw3000.ll().aon_dig_cfg().write(|w| w.onw_pgfcal(1));

	// INIT
	let mut dw3000 = dw3000.init(Config::default()).expect("Failed init.");

	check_states(&mut dw3000).unwrap();

	// CONF DE LA PLL pour passer en mode IDLE_PLL
/*
	// set CAL_EN in PLL_CAL register
	dw3000
		.ll()
		.pll_cal()
		.write(|w| w.cal_en(1))
		.expect("Write to PLL_CAL failed");
	//clear CP_LOCK
*/
	// In CLK_CTRL sub register, the 2 bits of SYS_CLK are set to AUTO

/*	

	// set PLL_CFG (4 bytes) / PLL_CFG_CH
	dw3000
		.ll()
		.pll_cfg()
		.write(|w| w.value(0x1F3C))
		.expect("Write 0x1F3C to PLL_CFG failed");
*/
	
	
	rprintln!("la pll est elle lock ? = {:#x?}", dw3000.idle_pll_passed());

	/*
	// ON PASSE EN MODE RECEVEUR
	let mut receiving = dw3000
		.receive(RxConfig {
			frame_filtering: false,
			..RxConfig::default()
		})
		.expect("Failed configure receiver.");
	rprintln!("On passe en mode reception = {:#x?}", receiving.state());

	delay.delay_ms(1000u16);

	rprintln!("\nOn regarde ou en est le receveur\n");
	rprintln!("Etat ? : {:#x?}", receiving.rx_state());
	*/

	loop {
		delay.delay_ms(10000u16);
	}
}
