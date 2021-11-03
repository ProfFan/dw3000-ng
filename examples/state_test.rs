#![no_main]
#![no_std]

// Exemple to look at the different module states 

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
use dw3000::{configs::FastCommand, hl, Config};

fn check_states<SPI, CS, State>(
	dw3000: &mut hl::DW3000<SPI, CS, State>,
) -> Result<(), hl::Error<SPI, CS>>
where
	SPI: spi::Transfer<u8> + spi::Write<u8>,
	CS: OutputPin,
	State: hl::Awake,
{
	if dw3000.init_rc_passed()? {
		rprintln!("INIT_RC state (rcinit = 1)");
	}
	if dw3000.idle_rc_passed()? {
		rprintln!("IDLE_RC state (spirdy = 1)");
	}
	if dw3000.idle_pll_passed()? {
		rprintln!("IDLE_PLL state (cpclock = 1)");
	}
	rprintln!(
		"the state is {:#x?}\n\n",
		dw3000.state()?
	);
	Ok(())
}

#[entry]
fn main() -> ! {
	rtt_init_print!();
	rprintln!("Coucou copain !\n\n");

	/******************************************************* */
	/************       BASE CONFIGURATION        ********** */
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
	/************       SPI CONFIGURATION        ********* */
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
	/*****                 DW3000 RESET             ******* */
	/****************************************************** */

	// NEW
	let mut delay = Delay::new(cp.SYST, clocks);

	let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);

	// UWB module reset
	rst_n.set_low().unwrap();
	rst_n.set_high().unwrap();

	/****************************************************** */
	/*********          DW3000 CONFIGURATION       ******** */
	/****************************************************** */

	let mut dw3000 = hl::DW3000::new(spi, cs);

	check_states(&mut dw3000).unwrap();
	delay.delay_ms(1000u16);

	check_states(&mut dw3000).unwrap();

	// auto calibration activation
	// dw3000.ll().aon_dig_cfg().write(|w| w.onw_pgfcal(1));

	// INIT
	let mut dw3000 = dw3000.init().expect("Failed init.");

	check_states(&mut dw3000).unwrap();

	rprintln!("la pll est elle lock ? = {:#x?}", dw3000.idle_pll_passed());

	delay.delay_ms(10_000_u16);

	let mut dw3000 = dw3000.config(Config::default()).expect("Failed init.");

	check_states(&mut dw3000).unwrap();
	rprintln!("la pll est elle lock ? = {:#x?}", dw3000.idle_pll_passed());

	// TRANSMITTER

	let delayed_tx_time = dw3000.sys_time().expect("Failed to get time");

	let mut sending = dw3000
		.send(
			b"ping",
			hl::SendTime::Delayed(delayed_tx_time),
			Config::default(),
		)
		.expect("Failed configure transmitter");
	rprintln!("changing to transmitter = {:#x?}", sending.state());

	delay.delay_ms(1000u16);

	rprintln!("\nhow is the transmitter ?\n");
	rprintln!("State ? : {:#x?}", sending.tx_state());

	let mut dw3000 = sending.finish_sending().expect("");

	loop {
		dw3000
			.ll()
			.fast_command(FastCommand::CMD_RX as u8)
			.expect("");
		dw3000.fast_cmd(FastCommand::CMD_RX).expect("");
		delay.delay_ms(10000u16);
	}
}
