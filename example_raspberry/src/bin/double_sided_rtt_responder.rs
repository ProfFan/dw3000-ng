use rppal::gpio::{Gpio};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

use std::thread;
use std::time::Duration;

use dw3000::{
	self,
	hl,
	Config,
	time::Instant
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
        .get(7)
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
		/*********    WAITING REQUEST FROM INITIATOR   ******** */
		/****************************************************** */
		println!("FIRST STEP : Waiting measurement request ...");

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
		println!("SECOND STEP : Offset response");

		// The buffer is empty because the initiator does not need timestamps
		// The final computation is done by the responder
		let delay_to_reply = t2 + (100000 * 63898); // micros * clock speed
		let mut sending = dw3000
			.send(
				&[0],
				hl::SendTime::Delayed(Instant::new(delay_to_reply).unwrap()),
				Config::default())
			.expect("Failed configure transmitter");
	    let result = block!(sending.s_wait());
	    let t3: u64 = result.unwrap().value();
	    dw3000 = sending.finish_sending().expect("Failed to finish sending");


		/****************************************************** */
		/*********          WAITING T1, T4, T5         ******** */
		/****************************************************** */
    	println!("THIRD STEP : Waiting for an answer...");

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

		println!("Distance = {} m", calc_distance_double(t1, t2, t3, t4, t5, t6));
		println!("--- RTT FINISHED ---\n");
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