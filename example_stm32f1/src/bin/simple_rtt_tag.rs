#![no_main]
#![no_std]

/*  
	simple tag example to be used with simple anchor example to perform RTT measurements (simple sided method)

	SIMPLE SIDED RTT MEASUREMENT TECHNIQUE :

	   TAG					ANCHOR
	T1	|--------____		  |
		| 		     -------> |	T2	
		|					  |						
		|		  ____--------| T3
	T4	| <-------
*/


use cortex_m_rt::entry;
use stm32f1xx_hal::{
	delay::Delay,
	pac,
	prelude::*,
	spi::{Mode, Phase, Polarity, Spi},
};
use dw3000::{
	hl,
	Config,
	time::Instant,
};
use nb::block;

#[entry]
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
	/*****               RESET DW3000               ******* */
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

	dw3000.set_antenna_delay(4416, 16500).expect("Failed set antenna delay.");

	loop {
		/**************************** */
		/********* RECEIVER ********* */
		/**************************** */
		let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");
		let mut buffer = [0; 1024];
		let t2 = block!(receiving.r_wait(&mut buffer)).expect("error during the reception").rx_time.value();
		dw3000 = receiving
			.finish_receiving()
			.expect("Failed to finish receiving");

		// We need to calculate a time (in ticks) at which we want to send the response
		let response_delay = t2 + (5000 * 63898) as u64; // T2(ticks) + (chosen_delay(µs) * clock_speed)
		let t3 = ((response_delay >> 9) << 9) + 16500 ;  // T3(ticks) = delay(31 MSB) + sending_antenna_delay 

		let response_tab = [
			((t2 >> (8 * 4) ) & 0xFF ) as u8,
			((t2 >> (8 * 3) ) & 0xFF ) as u8,
			((t2 >> (8 * 2) ) & 0xFF ) as u8,
			((t2 >>  8      ) & 0xFF ) as u8,
			 (t2              & 0xFF ) as u8,
			((t3 >> (8 * 4) ) & 0xFF ) as u8,
			((t3 >> (8 * 3) ) & 0xFF ) as u8,
			((t3 >> (8 * 2) ) & 0xFF ) as u8,
			((t3 >>  8      ) & 0xFF ) as u8,
			 (t3              & 0xFF ) as u8,
		];

		/**************************** */
		/*** TRANSMITTER T2 and T3 ** */
		/**************************** */

		let mut sending = dw3000
			.send(
				&response_tab,
				hl::SendTime::Delayed(Instant::new(response_delay).unwrap()),
				Config::default(),
			)
			.expect("Failed configure transmitter");
		block!(sending.s_wait()).expect("Error sending");
		dw3000 = sending.finish_sending().expect("Failed to finish sending");

		defmt::println!("T2 = {:?}", t2);
		defmt::println!("T3 = {:?}\n", t3);
	}
}
