/*
	A simple exemple to be used with simple_receive. It will send a frame on a loop
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
        // Initiate Sending
		let mut sending = dw3000
            .send(&[1, 2, 3, 4, 5], hl::SendTime::Now, Config::default())
            .expect("Failed configure transmitter");

        // Waiting for the frame to be sent
        let result = match block!(sending.s_wait()) {
            Ok(t) => t,
            Err(_e) => {
                println!("Error");
                dw3000 = sending.finish_sending().expect("Failed to finish sending");
                continue // Start a new loop iteration
            }
        };

		println!("Sending at {}", result.value());
		dw3000 = sending.finish_sending().expect("Failed to finish sending");
	}
}
