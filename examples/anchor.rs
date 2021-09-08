#![no_main]
#![no_std]

// crates de gestion des messages de debug
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use nb::block;

//use stm32f1::stm32f103;
//use stm32f1::stm32f103::interrupt;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    pac,
    prelude::*,
    spi::{Spi, Mode, Phase, Polarity},
    timer::Timer,
    //delay::Delay,
};

use dw1000::{hl, TxConfig, mac, ranging};

#[entry]
fn main() -> ! {

    rtt_init_print!();
    rprintln!("Coucou copain !");

    /****************************************************************************************/
    /*****************              CONFIGURATION DE BASE               *********************/
    /****************************************************************************************/

    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();
    //let cp = cortex_m::Peripherals::take().unwrap();

    // Take ownership over the raw flash and rcc devices and convert them into the corresponding
    // HAL structs
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(36.mhz()).freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    /****************************************************************************************/
    /*****************              CONFIGURATION DU SPI                *********************/
    /****************************************************************************************/

    let pins = (
        gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl),
        gpioa.pa6.into_floating_input(&mut gpioa.crl),
        gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl),
    );

    let cs = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);

    let spi_mode = Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };
    let spi = Spi::spi1(dp.SPI1, pins, &mut afio.mapr, spi_mode, 100.khz(), clocks, &mut rcc.apb2);

    //let mut delay = Delay::new(cp.SYST, clocks);

    /****************************************************************************************/
    /*****************              CONFIGURATION du DW3000               *******************/
    /****************************************************************************************/


    let mut dw1000 = hl::DW1000::new(spi, cs).init()
        .expect("Failed to initialize DW1000");
    rprintln!("dm1000 = {:?}", dw1000);

    // permet de visualiser les messages envoyés et recus
    dw1000.configure_leds(true, true, true, true, 15)
        .expect("Failed to initialize LEDS");
    /*
    dw1000.enable_tx_interrupts()
        .expect("Failed to enable TX interrupts");
    dw1000.enable_rx_interrupts()
        .expect("Failed to enable RX interrupts");
    */
    // [1] https://github.com/Decawave/dwm1001-examples
    //dw1000.set_antenna_delay(16456, 16300)
    //    .expect("Failed to set antenna delay");

    // Set network address
    /*
    dw1000
        .set_address(
            mac::PanId(0x0d57),    // hardcoded network id
            mac::ShortAddress(50), // pas random device address
        )
        .expect("Failed to set address"); 
    */



    //let mut task_timer = Timer::tim2(dp.TIM2, &clocks, &mut rcc.apb1);
    //let mut timeout_timer = Timer::tim3(dp.TIM3, &clocks, &mut rcc.apb1);
    //task_timer.start_count_down(1.hz());

    








    loop {
        let mut sending = dw1000
            .send(
                b"ping",
                mac::Address::broadcast(&mac::AddressMode::Short),
                None,
                TxConfig::default(),
            )
            .expect("Failed to start receiver");
        
        block!(sending.wait())
            .expect("Failed to send data");

        dw1000 = sending.finish_sending()
            .expect("Failed to finish sending");

        rprintln!(".");
    }    

}
