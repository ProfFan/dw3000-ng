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
use embedded_hal::digital::v2::OutputPin;
use dw3000::hl;
// use dw3000::time::{TIME_MAX,Instant,};
use dw3000::Config;

#[entry]
fn main() -> ! {
	rtt_init_print!();
	rprintln!("Coucou copain !");

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

	let mut delay = Delay::new(cp.SYST, clocks);

	let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);

	// UWB module reset
	rst_n.set_low().unwrap();
	rst_n.set_high().unwrap();

	/****************************************************** */
	/*********       CONFIGURATION du DW3000       ******** */
	/****************************************************** */

	// initialisation of the UWB module
	let mut dw3000 = hl::DW1000::new(spi, cs)
		.init()
		.expect("Failed init.")
		.config(Config::default())
		.expect("Failed init.");
	delay.delay_ms(1000u16);

	// conf du big registre RX_TUNE
	dw3000
		.ll()
		.dgc_cfg0()
		.write(|w| w.value(0x10000240))
		.expect("Aie");
	dw3000
		.ll()
		.dgc_cfg1()
		.write(|w| w.value(0x1b6da489))
		.expect("Aie");
	dw3000
		.ll()
		.dgc_lut_0()
		.write(|w| w.value(0x0001C0FD))
		.expect("Aie");
	dw3000
		.ll()
		.dgc_lut_1()
		.write(|w| w.value(0x0001C43E))
		.expect("Aie");
	dw3000
		.ll()
		.dgc_lut_2()
		.write(|w| w.value(0x0001C6BE))
		.expect("Aie");
	dw3000
		.ll()
		.dgc_lut_3()
		.write(|w| w.value(0x0001C77E))
		.expect("Aie");
	dw3000
		.ll()
		.dgc_lut_4()
		.write(|w| w.value(0x0001CF36))
		.expect("Aie");
	dw3000
		.ll()
		.dgc_lut_5()
		.write(|w| w.value(0x0001CFB5))
		.expect("Aie");
	dw3000
		.ll()
		.dgc_lut_6()
		.write(|w| w.value(0x0001CFF5))
		.expect("Aie");

	loop {
		delay.delay_ms(1000u16);
	}
}
