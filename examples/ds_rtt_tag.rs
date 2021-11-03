#![no_main]
#![no_std]

// simple tag exemple to be used with ds_rtt_anchor exemple 
// to implement double sided RTT communication

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use cortex_m_rt::entry;
use stm32f1xx_hal::{
	delay::Delay,
	pac,
	prelude::*,
	spi::{Mode, Phase, Polarity, Spi},
};
use embedded_hal::digital::v2::OutputPin;
use dw3000::{
	hl,
	Config,
	time::{Instant},
};
use nb::block;

#[entry]
fn main() -> ! {
	rtt_init_print!();
	rprintln!("Coucou copain !");

	/******************************************************* */
	/************            BASE CONFIG          ********** */
	/******************************************************* */

	// Get access to the device specific peripherals from the peripheral access
	// crate
	let dp = pac::Peripherals::take().unwrap();
	let cp = cortex_m::Peripherals::take().unwrap();

	// Take ownership over the raw flash and rcc devices and convert them into the
	// corresponding HAL structs
	let mut flash = dp.FLASH.constrain();
	let mut rcc = dp.RCC.constrain();
	let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

	let clocks = rcc
		.cfgr
		.use_hse(8.mhz())
		.sysclk(36.mhz())
		.freeze(&mut flash.acr);

	let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
	let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

	/***************************************************** */
	/************       SPI CONFIGURATION          ******* */
	/***************************************************** */

	let pins = (
		gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
		gpioa.pa6.into_floating_input(&mut gpioa.crl),
		gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
	);

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
		&mut rcc.apb2,
	);

	/****************************************************** */
	/*****                  DW3000 RESET            ******* */
	/****************************************************** */

	let mut delay = Delay::new(cp.SYST, clocks);

	let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);

	// UWB module reset
	rst_n.set_low().unwrap();
	rst_n.set_high().unwrap();

	/****************************************************** */
	/*********       DW3000 CONFIGURATION          ******** */
	/****************************************************** */

	let mut dw3000 = hl::DW3000::new(spi, cs)
		.init()
		.expect("Failed init.")
		.config(Config::default())
		.expect("Failed config.");

	dw3000.set_antenna_delay(0,0).unwrap();

	let mut buffer = [0; 1024]; // buffer to store reveived frame
	let fixed_delay = 0x10000000; // fixed delay for the transmission after a message reception

	loop {

		/**************************** */
		/********* RECEIVER T1 ****** */
		/**************************** */
		let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");

		// block until receive RX_TIME (40 bits)
		let t2 = block!(receiving.r_wait(&mut buffer)).expect("failed receiving data").rx_time.value();
		
		dw3000 = receiving
			.finish_receiving()
			.expect("Failed to finish receiving");

		let y = t2 & 0x1FF; // we store the last 9 bits of T2
		let t3 = t2 + fixed_delay;
		let delta_ar = fixed_delay - y;
		//rprintln!("delta_ar = {:b}", delta_ar);
		let delta_ar_send = [
			((delta_ar >> (8 * 3) ) & 0xFF ) as u8,
			((delta_ar >> (8 * 2) ) & 0xFF ) as u8,
			((delta_ar >>  8      ) & 0xFF ) as u8,
			 (delta_ar              & 0xFF ) as u8,
		];

		//rprintln!("t2 = {:b}", t2);
		//rprintln!("y = {:b}", y);
		//rprintln!("t3 = {:b}", t3);

		/**************************** */
		/*** TRANSMITTER delta_ar *****/
		/**************************** */

		let mut sending = dw3000
			.send(
				&delta_ar_send,
				hl::SendTime::Delayed(Instant::new(t3).unwrap()),
				Config::default(),
			)
			.expect("Failed configure transmitter");	
		block!(sending.s_wait());

		dw3000 = sending.finish_sending().expect("Failed to finish sending");
		
		/**************************** */
		/*  RECEIVER delta_tr, delta_tl AND T6    */
		/**************************** */
		let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");
		
		// block until receive RX_TIME (40 bits)
		let result = block!(receiving.r_wait(&mut buffer));

		dw3000 = receiving
			.finish_receiving()
			.expect("Failed to finish receiving");

			let result = result.unwrap();
			let t6:u64 = result.rx_time.value();
			let x = result.frame.payload;
			let delta_tl: u64 = ((x[0] as u64) << (8 * 3))
						+ ((x[1] as u64) << (8 * 2))
						+ ((x[2] as u64) << (8 * 1))
						+ (x[3] as u64);
			let delta_tr: u64 = ((x[4] as u64) << (8 * 3))
						+ ((x[5] as u64) << (8 * 2))
						+ ((x[6] as u64) << (8 * 1))
						+ (x[7] as u64);
			let delta_al = t6 - t3;

		let tof =
			(delta_tl * delta_al) - (delta_ar * delta_al) 
			/ (delta_tl + delta_tr + delta_al + delta_ar);
		
			// print result (Time Of Flight)
		rprintln!("TOF = {:?}", tof);
	}
}
