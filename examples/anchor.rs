#![no_main]
#![no_std]

// crates de gestion des messages de debug
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use cortex_m_rt::entry;
use stm32f1xx_hal::{
	delay::Delay,
	pac,
	prelude::*,
	spi::{Mode, Phase, Polarity, Spi},
};
use ieee802154::mac;
use ieee802154::mac::frame;
use embedded_hal::digital::v2::OutputPin;
use dw3000::{hl, time::Duration, Config, RxConfig, TxConfig};
use nb::block;

#[entry]
fn main() -> ! {
	rtt_init_print!();
	rprintln!("Coucou copain !");

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
		&mut rcc.apb2,
	);

	/****************************************************** */
	/*****       CONFIGURATION DU RESET du DW3000   ******* */
	/****************************************************** */

	let mut delay = Delay::new(cp.SYST, clocks);

	let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);

	// UWB module reset
	rst_n.set_low().unwrap();
	rst_n.set_high().unwrap();

	/****************************************************** */
	/*********       CONFIGURATION du DW3000       ******** */
	/****************************************************** */

	let mut dw3000 = hl::DW1000::new(spi, cs)
		.init()
		.expect("alo")
		.config(Config::default())
		.expect("Failed init.");
	//rprintln!("dm3000 = {:?}", dw3000);

	// let FIXED_DELAY = Duration::from_nanos(5_000_000_u32);

	// on cré un buffer pour stoquer le resultat message du receveur
	let mut buffer = [0; 1024];

	dw3000.set_antenna_delay(0,0).expect("Failed set antenna delay.");;

	loop {
		/**************************** */
		/******** TRANSMITTER ******* */
		/**************************** */
		// let delayed_tx_time = dw3000.sys_time().expect("Failed to get time");
		delay.delay_ms(5000_u32);
		let mut sending = dw3000
			.send(
				&[1, 2, 3, 4, 5],
				mac::Address::broadcast(&mac::AddressMode::Short),
				hl::SendTime::Now,
				TxConfig::default(),
			)
			.expect("Failed configure transmitter");
		let result = block!(sending.wait());
		let t1:u64 = result.unwrap().value();


		// on affiche le resultat
		//rprintln!("Trame envoyée !!!\n");

		dw3000 = sending.finish_sending().expect("Failed to finish sending");

		/**************************** */
		/********* RECEIVER T2 ****** */
		/**************************** */
		let mut receiving = dw3000
			.receive(RxConfig::default())
			.expect("Failed configure receiver.");

		// on cré un buffer pour stoquer le resultat message du receveur
		let mut buffer = [0; 1024];
		// delay.delay_ms(10u8);

		// on recupère un message avec une fonction bloquante
		let result = block!(receiving.wait(&mut buffer));
		// let rx_time : u64;// = result.unwrap().rx_time.value();

		dw3000 = receiving
			.finish_receiving()
			.expect("Failed to finish receiving");
		let result = result.unwrap();
		let t4:u64 = result.rx_time.value();
		let x = result.frame.payload;
		let t2: u64 = ((x[0] as u64) << (8 * 4)) 
					+ ((x[1] as u64) << (8 * 3))
					+ ((x[2] as u64) << (8 * 2))
					+ ((x[3] as u64) << (8 * 1))
					+ (x[4] as u64);
		//rprintln!("data = {:x?}", x);
		//rprintln!("data = {:x?}", t2);

		/**************************** */
		/********* RECEIVER T3 ****** */
		/**************************** */
		let mut receiving = dw3000
			.receive(RxConfig::default())
			.expect("Failed configure receiver.");
		let result = block!(receiving.wait(&mut buffer));
		dw3000 = receiving
			.finish_receiving()
			.expect("Failed to finish receiving");
		let x = result.unwrap().frame.payload;
		let t3: u64 = ((x[0] as u64) << (8 * 4)) 
					+ ((x[1] as u64) << (8 * 3))
					+ ((x[2] as u64) << (8 * 2))
					+ ((x[3] as u64) << (8 * 1))
					+ (x[4] as u64);

		rprintln!("T1 = {:?}", t1);
		rprintln!("T2 = {:?}", t2);
		rprintln!("T3 = {:?}", t3);
		rprintln!("T4 = {:?}", t4);
		rprintln!("distance = {:?}\n\n", ((t4-t1-t3+t2) / 2) as i64);
		// (((result_time - transmit_time - 320_000_000)as f64 * 299.792_458) / 128.0) as u64);
	}
}
