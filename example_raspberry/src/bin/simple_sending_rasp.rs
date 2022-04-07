/*
	A simple exemple to be used with simple_receive. It will send a frame on a loop
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
		let mut sending = dw3000
				.send(&[1, 2, 3, 4, 5], hl::SendTime::Now, Config::default())
				.expect("Failed configure transmitter");

		println!("Sending at {}", sending.ll().tx_time().read().unwrap().tx_stamp());
		dw3000 = sending.finish_sending().expect("Failed to finish sending");
	}
}
