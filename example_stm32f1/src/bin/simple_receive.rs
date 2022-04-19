/*
	A simple example to be used with simple_sending. It will receive a frame in a loop
*/
#![no_main]
#![no_std]

use example_stm32f1 as _; // global logger + panicking-behavior + memory layout

use stm32f1xx_hal::{
	delay::Delay,
	pac,
	prelude::*,
	spi::{Mode, Phase, Polarity, Spi},
};

use dw3000::{hl, Config};
use nb::block;


#[cortex_m_rt::entry]
fn main() -> ! {

    /******************************************************* */
	/************        BASIC CONFIGURATION      ********** */
	/******************************************************* */

	// Get access to the device specific peripherals from the peripheral access
	// crate
	let dp = pac::Peripherals::take().unwrap();
	let cp = cortex_m::Peripherals::take().unwrap();

	// Take ownership over the raw flash and rcc devices and convert them into the
	// corresponding HAL structs
	let mut flash = dp.FLASH.constrain();
	let rcc = dp.RCC.constrain();
	let mut afio = dp.AFIO.constrain();

	let clocks = rcc
		.cfgr
		.use_hse(8.mhz())
		.sysclk(36.mhz())
		.freeze(&mut flash.acr);

	let mut gpioa = dp.GPIOA.split();
	let mut gpiob = dp.GPIOB.split();

	/***************************************************** */
	/************         SPI CONFIGURATION        ******* */
	/***************************************************** */

	// CLOCK / MISO / MOSI
	let pins = (
		gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
		gpioa.pa6.into_floating_input(&mut gpioa.crl),
		gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
	);

	// Chip Select
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
	);

	/****************************************************** */
	/*****                DW3000 RESET              ******* */
	/****************************************************** */

	let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);
	rst_n.set_low();
	rst_n.set_high();

	/****************************************************** */
	/*********         DW3000 CONFIGURATION        ******** */
	/****************************************************** */

	let mut dw3000 = hl::DW3000::new(spi, cs)
		.init()
		.expect("Failed init.")
		.config(Config::default())
		.expect("Failed config.");
	
	let mut delay = Delay::new(cp.SYST, clocks);
	delay.delay_ms(500u16);

    defmt::println!("Init OK");

	loop {
		// Initiate Reception
		let mut buffer = [0; 1024];
        let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");

		// Waiting for an incomming frame
	    let result = match block!(receiving.r_wait(&mut buffer)) {
			Ok(t) => t,
			Err(_e) => {
				defmt::println!("Error");
				dw3000 = receiving.finish_receiving().expect("Failed to finish receiving");
				continue // Start a new loop iteration
			}
		};

		defmt::println!("Received '{}' at {}", result.frame.payload, result.rx_time.value());
		dw3000 = receiving.finish_receiving().expect("Failed to finish receiving");
	}
}
