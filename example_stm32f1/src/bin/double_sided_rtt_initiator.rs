/*
	A simple exemple to be used with double_sided_rtt_responder. It will send a frame in a loop
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

use dw3000::{hl, Config, time::Instant};
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
		/****************************************************** */
		/*********     INITIATE WITH A TRANSMITION     ******** */
		/****************************************************** */
		defmt::println!("FIRST STEP : Requesting new measurement...");

		let mut sending = dw3000
				.send(
					&[0],
					hl::SendTime::Now,
					Config::default(),
				)
				.expect("Failed configure transmitter");
		let result = block!(sending.s_wait());
		let t1: u64 = result.unwrap().value();
		dw3000 = sending.finish_sending().expect("Failed to finish sending");


		/****************************************************** */
		/*********      WAITING FOR THE RESPONSE       ******** */
		/****************************************************** */
		defmt::println!("SECOND STEP : Wainting answer...");

		let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");
	    let mut buffer = [0; 1024];
	    let result = block!(receiving.r_wait(&mut buffer)).expect("error during the reception");
		let t4 = result.rx_time.value();
	    dw3000 = receiving.finish_receiving().expect("Failed to finish receiving");
		

		/****************************************************** */
		/*********   BUILDING REPLY WITH T1, T4, T5    ******** */
		/****************************************************** */
		defmt::println!("THIRD STEP : Computing timestamps");

		// We need to calculate the time T5 (in ticks) at which we want to send the response
		let reply_delay = t4 + (100000_u64 * 63898) as u64; // T4(ticks) + (chosen_delay(µs) * clock_speed)
		let t5 = ((reply_delay >> 9) << 9) + dw3000.get_tx_antenna_delay().unwrap().value();  // T3(ticks) = delay(31 MSB) + sending_antenna_delay 

		let response_tab = [
			((t1 >> (8 * 4)) & 0xFF ) as u8, // T1
			((t1 >> (8 * 3)) & 0xFF ) as u8,
			((t1 >> (8 * 2)) & 0xFF ) as u8,
			( t1 >>  8       & 0xFF ) as u8,
			( t1             & 0xFF ) as u8,
			((t4 >> (8 * 4)) & 0xFF ) as u8, // T4
			((t4 >> (8 * 3)) & 0xFF ) as u8,
			((t4 >> (8 * 2)) & 0xFF ) as u8,
			( t4 >>  8       & 0xFF ) as u8,
			( t4             & 0xFF ) as u8,
			((t5 >> (8 * 4)) & 0xFF ) as u8, // T5
			((t5 >> (8 * 3)) & 0xFF ) as u8,
			((t5 >> (8 * 2)) & 0xFF ) as u8,
			( t5 >>  8       & 0xFF ) as u8,
			( t5             & 0xFF ) as u8,
		];
		
		/****************************************************** */
		/********  SENDING FINAL REPLY WITH T1, T4, T5  ******* */
		/****************************************************** */
		defmt::println!("FOURTH STEP : Sending final response...");

		let mut sending = dw3000
			.send(
				&response_tab,
				hl::SendTime::Delayed(Instant::new(reply_delay).unwrap()),
				Config::default(),
			)
			.expect("Failed configure transmitter");
		block!(sending.s_wait()).expect("Error sending");
		dw3000 = sending.finish_sending().expect("Failed to finish sending");
		
		defmt::println!("--- RTT FINISHED ---\n");
		delay.delay_ms(1000u16);
	}
}