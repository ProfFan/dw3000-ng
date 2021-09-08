#![no_main]
#![no_std]

// crates de gestion des messages de debug
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

//use stm32f1::stm32f103;
//use stm32f1::stm32f103::interrupt;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    pac,
    prelude::*,
    spi::{Spi, Mode, Phase, Polarity},
    //delay::Delay,
};

use dw1000::{configs, hl, RxConfig};

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
    // ne permet pas de 
    dw1000.configure_leds(true, true, true, true, 15)
        .expect("Failed to initialize LEDS");;

    
    //let mut dw1000 = dw1000.get_address();
    //rprintln!("dm1000 = {:?}", dw1000);

    //dw1000.set_adress();

    //let adresse = dw1000.get_address().unwrap();
    //rprintln!("adresse = {:?}", adresse);

    let dev_id = dw1000.ll().dev_id().read()
        .expect("Failed to read DEV_ID register");

    rprintln!("info ? = {:?}", dev_id);

    let mut dw1000 = dw1000.receive(RxConfig::default())
        .expect("Failed to set DW1000 as a receiver");;
    rprintln!("dm1000 = {:?}", dw1000);

    loop {
        
    }    

}
