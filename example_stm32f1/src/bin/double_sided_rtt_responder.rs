/*
	An exemple to be used with double_sided_rtt_initiator. It will send a frame in a loop
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
	//dw3000.set_antenna_delay(4416, 16500).expect("Failed set antenna delay.");
	
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
		/*********       RESPONDING TO INITIATOR       ******** */
		/****************************************************** */
		defmt::println!("SECOND STEP : Offset response");

		// The buffer is empty because the initiator does not need timestamps
		// The final computation is done by the responder
		let buff: [u8;10] = [0;10];
		let delay_to_reply = t2 + (100000 * 63898); // micros * clock speed
		let mut sending = dw3000
			.send(
				&buff,
				hl::SendTime::Delayed(Instant::new(delay_to_reply).unwrap()),
				Config::default())
			.expect("Failed configure transmitter");
	    let result = block!(sending.s_wait());
	    let t3: u64 = result.unwrap().value();
	    dw3000 = sending.finish_sending().expect("Failed to finish sending");


		/****************************************************** */
		/*********          WAITING T1, T4, T5         ******** */
		/****************************************************** */
    	defmt::println!("THIRD STEP : Waiting for an answer...");

		let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");
		let mut buffer = [0; 1024];
		let result = block!(receiving.r_wait(&mut buffer));
		dw3000 = receiving.finish_receiving().expect("Failed to finish receiving");
		
		let result = result.unwrap();
		let x = result.frame.payload;
		let t1: u64 = ((x[0] as u64) << (8 * 4)) 
					+ ((x[1] as u64) << (8 * 3))
					+ ((x[2] as u64) << (8 * 2))
					+ ((x[3] as u64) <<  8)
					+  (x[4] as u64);
		let t4: u64 = ((x[5] as u64) << (8 * 4)) 
					+ ((x[6] as u64) << (8 * 3))
					+ ((x[7] as u64) << (8 * 2))
					+ ((x[8] as u64) <<  8)
					+  (x[9] as u64);
		let t5 : u64= ((x[10] as u64) << (8 * 4)) 
					+ ((x[11] as u64) << (8 * 3))
					+ ((x[12] as u64) << (8 * 2))
					+ ((x[13] as u64) <<  8)
					+ ( x[14] as u64);
		let t6: u64 = result.rx_time.value();

		defmt::println!("Distance = {} m", calc_distance_double(t1, t2, t3, t4, t5, t6));
		defmt::println!("--- RTT FINISHED ---\n");
	}
}

fn calc_distance_double(t1: u64, t2: u64, t3: u64, t4: u64, t5: u64, t6: u64) -> f64 {
	let f: f64 = 1.0/499200000.0/128.0; // Clock speed
	let s_light: f64 = 299792458.0; // speed of light m/s

	let tround1: f64 = (t4 - t1) as f64;
	let treply1: f64 = (t3 - t2) as f64;
	let tround2: f64 = (t6 - t3) as f64;
	let treply2: f64 = (t5 - t4) as f64;

	let tof_tick: f64 = (((tround1 as u128 * tround2 as u128) - (treply1 as u128 * treply2 as u128 )) / ((tround1 + treply1 + tround2 + treply2) as u128)) as f64;
	let tof_sec: f64 = tof_tick * f;

	let distance: f64 = tof_sec * s_light;	
	distance
}