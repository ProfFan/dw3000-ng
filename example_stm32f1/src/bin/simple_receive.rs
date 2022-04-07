/*
	A simple example to be used with simple_sending. It will receive a frame on a loop
*/

#![no_main]
#![no_std]

use dw3000 as _; // global logger + panicking-behavior + memory layout

use stm32f1xx_hal::{
	delay::Delay,
	pac,
	prelude::*,
	spi::{Mode, Phase, Polarity, Spi},
};

use dw3000::{hl::{self,ConfigGPIOs}, Config};
use nb::block;


#[cortex_m_rt::entry]
fn main() -> ! {

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
	);

	/****************************************************** */
	/*****       CONFIGURATION DU RESET du DW3000   ******* */
	/****************************************************** */

	let mut delay = Delay::new(cp.SYST, clocks);

	let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);

	// UWB module reset
	rst_n.set_low();
	rst_n.set_high();

	/****************************************************** */
	/*********       CONFIGURATION du DW3000       ******** */
	/****************************************************** */

	let mut dw3000 = hl::DW3000::new(spi, cs)
		.init()
		.expect("Failed init.")
		.config(Config::default())
		.expect("Failed config.");
	delay.delay_ms(3000u16);
    defmt::println!("configuration du DW3000 terminée");

	loop {
        let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");
	    let mut buffer = [0; 1024];
	    let result = match block!(receiving.r_wait(&mut buffer)) {
			Ok(t) => {
				defmt::println!("Received something at {}", receiving.ll().rx_time().read().unwrap().rx_stamp());
				t
			},
			Err(e) => match e {
				_ => {
					defmt::println!("Erreur Receive");
					dw3000 = receiving.finish_receiving().expect("Failed to finish receiving");
					continue
				}
			}
		};
		dw3000 = receiving.finish_receiving().expect("Failed to finish receiving");
	}
}
