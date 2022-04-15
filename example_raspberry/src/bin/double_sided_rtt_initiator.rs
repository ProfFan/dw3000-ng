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
		let result = block!(sending.s_wait());
		let t1: u64 = result.unwrap().value();
		dw3000 = sending.finish_sending().expect("Failed to finish sending");


		/****************************************************** */
		/*********      WAITING FOR THE RESPONSE       ******** */
		/****************************************************** */
		println!("SECOND STEP : Wainting answer...");

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
		println!("THIRD STEP : Computing timestamps");

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
		println!("FOURTH STEP : Sending final response...");

		let mut sending = dw3000
			.send(
				&response_tab,
				hl::SendTime::Delayed(Instant::new(reply_delay).unwrap()),
				Config::default(),
			)
			.expect("Failed configure transmitter");
		block!(sending.s_wait()).expect("Error sending");
		dw3000 = sending.finish_sending().expect("Failed to finish sending");
		
		println!("--- RTT FINISHED ---\n");
		thread::sleep(Duration::from_millis(1000));
	}
}