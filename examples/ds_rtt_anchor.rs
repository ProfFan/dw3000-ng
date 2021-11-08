#![no_main]
#![no_std]

// This exemple should be used with ds_rtt_tag exemple and is the implementation of 
// RTT double sided localisation process

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
use dw3000::{Config, hl, time::Instant};
use nb::block;

#[entry]
fn main() -> ! {
	rtt_init_print!();
	rprintln!("Coucou copain !");

	/******************************************************* */
	/************       BASE CONFIGURATION        ********** */
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
		.sysclk(72.mhz())
		.pclk1(36.mhz())
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
	/*****                DW3000 RESET              ******* */
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
		.expect("alo")
		.config(Config::default())
		.expect("Failed init.");

	dw3000.set_antenna_delay(0,0).expect("Failed set antenna delay.");

	let mut buffer = [0; 1024]; // buffer to store reveived frame
	let fixed_delay = 0x800000000; // fixed delay for the transmission after a message reception

	loop {

		delay.delay_ms(500_u32);

		/**************************** */
		/******** TRANSMITTER T1 **** */
		/**************************** */
		let mut sending = dw3000
			.send(
				&[1, 2, 3, 4, 5],
				hl::SendTime::Now,
				Config::default(),
			)
			.expect("Failed configure transmitter");
		let result = block!(sending.s_wait());
		let t1:u64 = result.unwrap().value();

		dw3000 = sending.finish_sending().expect("Failed to finish sending");

		/**************************** */
		/*  RECEIVER delta_ar + T4    */
		/**************************** */
		let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");

		// block until reveive message
		let result = block!(receiving.r_wait(&mut buffer));

		dw3000 = receiving
			.finish_receiving()
			.expect("Failed to finish receiving");
		let result = result.unwrap();
		let t4:u64 = result.rx_time.value();
		let delta_tl = t4 - t1;
		let x = result.frame.payload;
		let delta_ar: u64 = ((x[0] as u64) << (8 * 3))
					+ ((x[1] as u64) << (8 * 2))
					+ ((x[2] as u64) << (8 * 1))
					+ (x[3] as u64);
		// rprintln!("delta_ar recu = {:?}", delta_ar); 
		let y = t4 & 0x1FF; // on prend les 9 derniers bits de T4
		let mut t5 = t4 + fixed_delay;
		let delta_tr = fixed_delay - y;
		let delta_tr_tl = [
			((delta_tl >> (8 * 3) ) & 0xFF ) as u8,
			((delta_tl >> (8 * 2) ) & 0xFF ) as u8,
			((delta_tl >>  8      ) & 0xFF ) as u8,
			 (delta_tl              & 0xFF ) as u8,

			((delta_tr >> (8 * 3) ) & 0xFF ) as u8,
			((delta_tr >> (8 * 2) ) & 0xFF ) as u8,
			((delta_tr >>  8      ) & 0xFF ) as u8,
			 (delta_tr              & 0xFF ) as u8,
		];

		if t5 > 0xFFFFFFFFFF {
			t5 %= 0xFFFFFFFFFF;
		}

		/**************************** */
		/*** TRANSMITTER delta_tr AND delta_tl *** */
		/**************************** */
		let mut sending = dw3000
			.send(
				&delta_tr_tl,
				hl::SendTime::Delayed(Instant::new(t5).unwrap()),
				Config::default(),
			)
			.expect("Failed configure transmitter");
		block!(sending.s_wait());

		dw3000 = sending.finish_sending().expect("Failed to finish sending");
	}
}
