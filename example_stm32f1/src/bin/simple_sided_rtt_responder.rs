/*  
	simple RESPONDER example to be used with simple INITIATOR example to perform RTT measurements (simple sided method)

	SIMPLE SIDED RTT MEASUREMENT TECHNIQUE :

	INITIATOR				RESPONDER
	T1	|--------____		  |
		| 		     -------> |	T2	
		|					  |						
		|		  ____--------| T3
	T4	| <-------

	/!\ A speed difference between the clocks exists, which impacts the measures. The use of the Double Sided is recommended /!\

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
		/*********    WAITING REQUEST FROM INITIATOR   ******** */
		/****************************************************** */
		defmt::println!("FIRST STEP : Waiting measurement request ...");

        let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");
	    let mut buffer = [0; 1024];
		let t2: u64 = block!(receiving.r_wait(&mut buffer)).expect("error during the reception").rx_time.value();
	    dw3000 = receiving
		    .finish_receiving()
		    .expect("Failed to finish receiving");

		/****************************************************** */
		/*********         COMPUTING T2 AND T3         ******** */
		/****************************************************** */
		defmt::println!("SECOND STEP : Computing timestamps...");

		// We need to calculate a time (in ticks) at which we want to send the response
		let delay_to_reply = t2 + (5000 * 63898); // T2(ticks) + (chosen_delay(µs) * clock_speed) % 1_0995_1162_7776
		let t3: u64 = ((delay_to_reply >> 9) << 9) + dw3000.get_tx_antenna_delay().unwrap().value();  // T3(ticks) = delay(31 MSB) + sending_antenna_delay 

		let response_tab = [
			((t2 >> (8 * 4)) & 0xFF ) as u8,
			((t2 >> (8 * 3)) & 0xFF ) as u8,
			((t2 >> (8 * 2)) & 0xFF ) as u8,
			((t2 >>  8)      & 0xFF ) as u8,
			( t2             & 0xFF ) as u8,
			((t3 >> (8 * 4)) & 0xFF ) as u8,
			((t3 >> (8 * 3)) & 0xFF ) as u8,
			((t3 >> (8 * 2)) & 0xFF ) as u8,
			((t3 >>  8)      & 0xFF ) as u8,
			( t3             & 0xFF ) as u8,
		];

		/****************************************************** */
		/*********          SENDING T2 AND T3          ******** */
		/****************************************************** */
		defmt::println!("THIRD STEP : Offset response...");

		let mut sending = dw3000
			.send(
				&response_tab,
				hl::SendTime::Delayed(Instant::new(delay_to_reply).unwrap()),
				Config::default())
			.expect("Failed configure transmitter");
		let _result = block!(sending.s_wait()).expect("Error sending");
		dw3000 = sending.finish_sending().expect("Failed to finish sending");
		
		defmt::println!("--- RTT FINISHED ---\n");
	}
}
