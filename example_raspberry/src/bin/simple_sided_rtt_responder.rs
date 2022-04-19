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
		/*********         COMPUTING T2 AND T3         ******** */
		/****************************************************** */
		println!("SECOND STEP : Computing timestamps...");

		// We need to calculate a time (in ticks) at which we want to send the response
		let delay_to_reply = t2 + (100000 * 63898); // T2(ticks) + (chosen_delay(µs) * clock_speed) % 1_0995_1162_7776
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
		println!("THIRD STEP : Offset response...");

		let mut sending = dw3000
			.send(
				&response_tab,
				hl::SendTime::Delayed(Instant::new(delay_to_reply).unwrap()),
				Config::default())
			.expect("Failed configure transmitter");
		let _result = block!(sending.s_wait()).expect("Error sending");
		dw3000 = sending.finish_sending().expect("Failed to finish sending");
		
		println!("--- RTT FINISHED ---\n");
	}
}
