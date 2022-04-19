/*  
	simple INITIATOR example to be used with simple RESPONDER example to perform RTT measurements (simple sided method)

	SIMPLE SIDED RTT MEASUREMENT TECHNIQUE :

	INITIATOR				RESPONDER
	T1	|--------____		  |
		| 		     -------> |	T2	
		|					  |						
		|		  ____--------| T3
	T4	| <-------

	/!\ A speed difference between the clocks exists, which impacts the measures. The use of the Double Sided is recommended /!\

*/
use rppal::gpio::{Gpio};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

use std::thread;
use std::time::Duration;

use dw3000::{
	self,
	hl,
	Config
};
use nb::block;

fn main() -> ! {
    
    /******************************************************* */
	/************        BASIC CONFIGURATION      ********** */
	/******************************************************* */

    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 4_500_000, Mode::Mode0)
        .expect("Failed to configure the spi");
    let gpio = Gpio::new()
        .expect("Failed to configure GPIO");
    let cs = gpio
        .get(8)
        .expect("Failed to set up CS PIN")
        .into_output();

    /****************************************************** */
	/*****                DW3000 RESET              ******* */
	/****************************************************** */

    let mut reset = gpio
        .get(4)
        .expect("Failed to set up RESET PIN")
        .into_output();
    reset.set_low();
    reset.set_high();

    /****************************************************** */
	/*********         DW3000 CONFIGURATION        ******** */
	/****************************************************** */

	let mut dw3000 = hl::DW3000::new(spi, cs)
		.init()
		.expect("Failed init.")
		.config(Config::default())
		.expect("Failed config.");

    thread::sleep(Duration::from_millis(500));
    println!("Init OK");


	loop {
		/****************************************************** */
		/*********     INITIATE WITH A TRANSMITION     ******** */
		/****************************************************** */
		println!("FIRST STEP : Requesting new measurement...");

		let mut sending = dw3000
				.send(
					&[0],
					hl::SendTime::Now,
					Config::default(),
				)
				.expect("Failed configure transmitter");
		let result = block!(sending.s_wait()).expect("Error Sending");
		let t1: u64 = result.value();
		dw3000 = sending.finish_sending().expect("Failed to finish sending");
		
		/****************************************************** */
		/*********          WAITING T2 AND T3          ******** */
		/****************************************************** */
		println!("SECOND STEP : Wainting answer...");

		let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");
	    let mut buffer = [0; 1024];
	    let result = block!(receiving.r_wait(&mut buffer)).expect("error during the reception");

		dw3000 = receiving
			.finish_receiving()
			.expect("Failed to finish receiving");

		let x = result.frame.payload;
		let t2: u64 = ((x[0] as u64) << (8 * 4)) 
					+ ((x[1] as u64) << (8 * 3))
					+ ((x[2] as u64) << (8 * 2))
					+ ((x[3] as u64) <<  8)
					+  (x[4] as u64);
		let t3: u64 = ((x[5] as u64) << (8 * 4)) 
					+ ((x[6] as u64) << (8 * 3))
					+ ((x[7] as u64) << (8 * 2))
					+ ((x[8] as u64) <<  8)
					+  (x[9] as u64);
		let t4: u64 = result.rx_time.value();
		
		println!("Distance = {} m", calc_distance_simple(t1, t2, t3, t4));
		println!("--- RTT FINISHED ---\n");
		thread::sleep(Duration::from_millis(500));
	}
}

fn calc_distance_simple(t1: u64, t2: u64, t3: u64, t4: u64) -> f64 {
	let f: f64 = 1.0/499200000.0/128.0; // DW3000 frequency (Hz)
    let s_light: f64 = 299792458.0; // light speed 10⁹m/s

    let tround: f64 = (t4 - t1) as f64;
    let treply: f64 = (t3 - t2) as f64;

	let tof_tick: f64 = (tround-treply) / 2_f64;

	let tof_sec: f64 = tof_tick * f ;
	
	let distance: f64 = s_light * tof_sec;
	distance
}
