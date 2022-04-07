/*
	A simple example to be used with simple_sending. It will receive a frame on a loop
*/

use rppal::gpio::{Gpio};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::hal::Timer;
use embedded_hal::timer::CountDown;

use std::thread;
use std::time::Duration;
use embedded_timeout_macros::block_timeout;

use dw3000::{
	self,
	hl,
	Config,
    time::Instant,

};
use nb::block;

fn main() -> ! {
    let mut timing_data: [u64; 10]=[0; 10];
    let mut buffer = [0; 1024];

    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 4_500_000, Mode::Mode0)
        .expect("Failed to configure the spi");
    let gpio = Gpio::new().expect("Failed to configure GPIO");
    let cs = gpio.get(24).expect("Failed to set up CS PIN").into_output();
    let mut reset = gpio
        .get(7)
        .expect("Failed to set up RESET PIN")
        .into_output();

    // reset DW3000 module
    thread::sleep(Duration::from_millis(500));
    reset.set_low();
    reset.set_high();

	let mut dw3000 = hl::DW3000::new(spi, cs)
		.init()
		.expect("Failed init.")
		.config(Config::default())
		.expect("Failed config.");

    let tx_antenna_delay: u16 = 16500;
    dw3000.set_antenna_delay(4416, tx_antenna_delay);

    dw3000.ll().tx_power().modify(|_, w| {
        w.value(0xfdfdfdfd)
    });

    println!("Init OK");

	loop {
        let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");
	    let mut buffer = [0; 1024];
	    let result = match block!(receiving.r_wait(&mut buffer)) {
			Ok(t) => {
				println!("Received something at {}", receiving.ll().rx_time().read().unwrap().rx_stamp());
				t
			},
			Err(e) => match e {
				_ => {
					println!("Erreur Receive");
					dw3000 = receiving.finish_receiving().expect("Failed to finish receiving");
					continue
				}
			}
		};
		dw3000 = receiving.finish_receiving().expect("Failed to finish receiving");
	}
}
