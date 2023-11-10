/*
    simple INITIATOR example to be used with simple RESPONDER example to perform RTT measurements (simple sided method)

    SIMPLE SIDED RTT MEASUREMENT TECHNIQUE :

    INITIATOR				RESPONDER
    T1	|--------____		  |
        | 		     -------> |	T2
        |					  |
        |		  ____--------| T3
    T4	| <-------

    /!\ A speed difference between the clocks exists, which impacts the measures. The use of the Double Sided is recommended /!\

*/

#![no_main]
#![no_std]

use example_stm32f1 as _; // global logger + panicking-behavior + memory layout

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
    /************        BASIC CONFIGURATION      ********** */
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
    /************         SPI CONFIGURATION        ******* */
    /***************************************************** */

    // CLOCK / MISO / MOSI
    let pins = (
        gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
        gpioa.pa6.into_floating_input(&mut gpioa.crl),
        gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
    );

    // Chip Select
    let cs = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);

    let spi_mode = Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };

    let spi = Spi::spi1(dp.SPI1, pins, &mut afio.mapr, spi_mode, 100.khz(), clocks);

    /****************************************************** */
    /*****                DW3000 RESET              ******* */
    /****************************************************** */

    let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);
    rst_n.set_low();
    rst_n.set_high();

    /****************************************************** */
    /*********         DW3000 CONFIGURATION        ******** */
    /****************************************************** */

    let mut dw3000 = hl::DW3000::new(spi, cs)
        .init()
        .expect("Failed init.")
        .config(Config::default())
        .expect("Failed config.");

    let mut delay = Delay::new(cp.SYST, clocks);
    delay.delay_ms(500u16);

    defmt::println!("Init OK");

    loop {
        /****************************************************** */
        /*********     INITIATE WITH A TRANSMITION     ******** */
        /****************************************************** */
        defmt::println!("FIRST STEP : Requesting new measurement...");

        let mut sending = dw3000
            .send(&[0], hl::SendTime::Now, Config::default())
            .expect("Failed configure transmitter");
        let result = block!(sending.s_wait()).expect("Error Sending");
        let t1: u64 = result.value();
        dw3000 = sending.finish_sending().expect("Failed to finish sending");

        /****************************************************** */
        /*********          WAITING T2 AND T3          ******** */
        /****************************************************** */
        defmt::println!("SECOND STEP : Wainting answer...");

        let mut receiving = dw3000
            .receive(Config::default())
            .expect("Failed configure receiver.");
        let mut buffer = [0; 1024];
        let result = block!(receiving.r_wait(&mut buffer)).expect("error during the reception");

        dw3000 = receiving
            .finish_receiving()
            .expect("Failed to finish receiving");

        let x = result.frame.payload;
        let t2: u64 = ((x[0] as u64) << (8 * 4))
            + ((x[1] as u64) << (8 * 3))
            + ((x[2] as u64) << (8 * 2))
            + ((x[3] as u64) << 8)
            + (x[4] as u64);
        let t3: u64 = ((x[5] as u64) << (8 * 4))
            + ((x[6] as u64) << (8 * 3))
            + ((x[7] as u64) << (8 * 2))
            + ((x[8] as u64) << 8)
            + (x[9] as u64);
        let t4: u64 = result.rx_time.value();

        defmt::println!("Distance = {} m", calc_distance_simple(t1, t2, t3, t4));
        defmt::println!("--- RTT FINISHED ---\n");
        delay.delay_ms(500u16);
    }
}

fn calc_distance_simple(t1: u64, t2: u64, t3: u64, t4: u64) -> f64 {
    let f: f64 = 1.0 / 499200000.0 / 128.0; // DW3000 frequency (Hz)
    let s_light: f64 = 299792458.0; // light speed 10⁹m/s

    let tround: f64 = (t4 - t1) as f64;
    let treply: f64 = (t3 - t2) as f64;

    let tof_tick: f64 = (tround - treply) / 2_f64;

    let tof_sec: f64 = tof_tick * f;

    let distance: f64 = s_light * tof_sec;
    distance
}
