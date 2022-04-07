/*
	Double sided RTT example, to use with ds_rtt_t
*/
#![no_main]
#![no_std]

use dw3000 as _; // global logger + panicking-behavior + memory layout

use stm32f1xx_hal::{
	delay::Delay,
	pac,
	prelude::*,
	spi::{Mode, Phase, Polarity, Spi},
};

use dw3000::{hl, Config};
use nb::block;


#[cortex_m_rt::entry]
fn main() -> ! {

    /******************************************************* */
	/************       CONFIGURATION DE BASE     ********** */
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
	/************       CONFIGURATION DU SPI       ******* */
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
	);

	/****************************************************** */
	/*****       CONFIGURATION DU RESET du DW3000   ******* */
	/****************************************************** */

	let mut delay = Delay::new(cp.SYST, clocks);

	let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);

	// UWB module reset
	rst_n.set_low();
	rst_n.set_high();

	/****************************************************** */
	/*********       CONFIGURATION du DW3000       ******** */
	/****************************************************** */

	let mut dw3000 = hl::DW3000::new(spi, cs)
		.init()
		.expect("Failed init.")
		.config(Config::default())
		.expect("Failed config.");
	delay.delay_ms(3000u16);
    defmt::println!("configuration du DW3000 terminée");
    //check_states(&mut dw3000).unwrap();
    let mut i = 0;
	let add_dest = dw3000::mac::ShortAddress::broadcast();
	loop {
		delay.delay_ms(6u16);
		// Envoie 1er message + T1 
		let mut sending = dw3000
				.send(&[1, 2, 3, 4, 5], hl::SendTime::Now, Config::default())
				.expect("Failed configure transmitter");
		let result = block!(sending.s_wait());
		let t1: u64 = result.unwrap().value();
		dw3000 = sending.finish_sending().expect("Failed to finish sending");
		//defmt::println!("t {} = {}", 1, t1);
		//let t1 = dw3000.ll().tx_time().read().unwrap().tx_stamp();

		//Reception 2nd message (T2) + T4
		let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");
	    let mut buffer = [0; 1024];
	    let result = block!(receiving.r_wait(&mut buffer)).expect("pb receive");
	    dw3000 = receiving
		    .finish_receiving()
		    .expect("Failed to finish receiving");
		let t2 = convert_u8_u64(result.frame.payload);
		let t4 = result.rx_time.value();

		delay.delay_ms(6u16);
		//Envoie du 3ème message + T5
		let mut sending = dw3000
				.send(&[1, 2, 3, 4, 5], hl::SendTime::Now, Config::default())
				.expect("Failed configure transmitter");
		let result = block!(sending.s_wait());
		let t5: u64 = result.unwrap().value();
		dw3000 = sending.finish_sending().expect("Failed to finish sending");

		//Reception de T3
		let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");
		let mut buffer = [0; 1024];
		let result = block!(receiving.r_wait(&mut buffer)).expect("pb receive");
		dw3000 = receiving
			.finish_receiving()
			.expect("Failed to finish receiving");
		let t3 = convert_u8_u64(result.frame.payload);

		//Reception de T6
		let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");
		let mut buffer = [0; 1024];
		let result = block!(receiving.r_wait(&mut buffer)).expect("pb receive");
		dw3000 = receiving
			.finish_receiving()
			.expect("Failed to finish receiving");
		let t6 = convert_u8_u64(result.frame.payload);


		//defmt::println!("T1 = {}, T2 = {}, T3 = {}, T4 = {}, T5 = {}, T6 = {}", t1, t2, t3, t4, t5, t6);
		let TA_ro = t4 - t1;
		let TA_re = t5 - t4;
		let TB_ro = t6 - t3;
		let TB_re = t3 - t2;
		let tof : f64 = (TA_ro * TB_ro - TA_re * TB_re) as f64 / (TA_ro + TA_re + TB_ro + TB_re) as f64;
		//defmt::println!("TOF = {}", tof);

		let f : f64 = 63897600000.0;   // fréquence de DW3000 en GHz
		let s_light : f64 = 299792458.0; // vitesse de la lumière 10⁹m/s
		let distance : f64 = (tof as f64 / f) * s_light;
		defmt::println!("distance = {}", distance);
	}
}

fn convert_u8_u64 (u8_array: &[u8]) -> u64 {

	let u64_ = ((u8_array[0] as u64) << (8 * 4)) 
		+ ((u8_array[1] as u64) << (8 * 3))
		+ ((u8_array[2] as u64) << (8 * 2))
		+ ((u8_array[3] as u64) << (8 * 1))
		+ (u8_array[4] as u64);
		//defmt::println!("u64 = {}", u64_);
		u64_
}