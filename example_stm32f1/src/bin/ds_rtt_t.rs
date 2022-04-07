/*
	Double sided RTT example, to use with ds_rtt_r
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

use embedded_hal::{blocking::spi, digital::v2::OutputPin};
use dw3000::{hl, Config};
use nb::block;
use stm32f1xx_hal::timer::Timer;
use embedded_timeout_macros::block_timeout;


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
    let i = 0;
    let mut buff : [u8;5] = [0;5];
    let mut t2 : u64 = 0;
    let mut t3 : u64 = 0;
    let mut t6 : u64 = 0;

    loop {

        //Reception 1er message + T2
        let mut receiving = dw3000
			.receive(Config::default())
			.expect("Failed configure receiver.");
	    let mut buffer = [0; 1024];
	    let result = block!(receiving.r_wait(&mut buffer));
	    dw3000 = receiving
		    .finish_receiving()
		    .expect("Failed to finish receiving");
        t2 = dw3000.ll().rx_time().read().unwrap().rx_stamp();
        
        //T2
        convert_u64_u8(t2,&mut buff);

        //Envoie 2nd message (T2) + T3
        delay.delay_ms(6u16);
	    let mut sending = dw3000
			.send(&buff, hl::SendTime::Now, Config::default())
			.expect("Failed configure transmitter");
	    let result = block!(sending.s_wait());
	    t3 = result.unwrap().value();
	    dw3000 = sending.finish_sending().expect("Failed to finish sending");
    
        //Reception 3eme message + T6
        let mut receiving = dw3000
            .receive(Config::default())
            .expect("Failed configure receiver.");
        let mut buffer = [0; 1024];
        let result = block!(receiving.r_wait(&mut buffer));
        dw3000 = receiving
            .finish_receiving()
            .expect("Failed to finish receiving");
        t6 = dw3000.ll().rx_time().read().unwrap().rx_stamp();
            //convert_u64_u8(dw3000.ll().rx_time().read().unwrap().rx_stamp(),&mut buff);
        
        //Envoie T3
        delay.delay_ms(6u16);
        convert_u64_u8(t3,&mut buff);
	    let mut sending = dw3000
			.send(&buff, hl::SendTime::Now, Config::default())
			.expect("Failed configure transmitter");
	    let result = block!(sending.s_wait());
	    //let t3: u64 = result.unwrap().value();
	    dw3000 = sending.finish_sending().expect("Failed to finish sending");

        //Envoie T6
        delay.delay_ms(6u16);
        convert_u64_u8(t6,&mut buff);
	    let mut sending = dw3000
			.send(&buff, hl::SendTime::Now, Config::default())
			.expect("Failed configure transmitter");
	    let result = block!(sending.s_wait());
	    //let t3: u64 = result.unwrap().value();
	    dw3000 = sending.finish_sending().expect("Failed to finish sending");
    }
}

fn convert_u64_u8 (u64time: u64, u8_array : & mut [u8;5]) {
	
	for i in 0..5 {
		u8_array[i] = (u64time >> (8* (4-i))) as u8;
		//defmt::info!("u8Array[{}] = {}", 4-i, u8_array[(4-i)]);

	}
}