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
    spi::{Spi, Mode, Phase, Polarity},
};

use embedded_hal::digital::v2::OutputPin;

use dw3000::{hl, mac};

use nb::block;

use dw3000::time::{TIME_MAX,Instant,};



#[entry]
fn main() -> ! {

    rtt_init_print!();
    rprintln!("Coucou copain !");

    /****************************************************************************************/
    /*****************              CONFIGURATION DE BASE               *********************/
    /****************************************************************************************/

    // Get access to the device specific peripherals from the peripheral access crate
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

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

    /****************************************************************************************/
    /************              CONFIGURATION DU RESET du DW3000             *****************/
    /****************************************************************************************/

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);

    // UWB module reset 
    rst_n.set_low().unwrap();
    rst_n.set_high().unwrap();

    /****************************************************************************************/
    /*****************              CONFIGURATION du DW3000               *******************/
    /****************************************************************************************/


    rprintln!("On initialise le module : new + init en meme temps");
    let mut dw3000 = hl::DW1000::new(spi, cs).init()
                        .expect("Failed init.");
    rprintln!("dm3000 = {:?}", dw3000);

    delay.delay_ms(3000u16);
    let mut state = dw3000.ll().sys_state().read().unwrap().pmsc_state();
    rprintln!("l'état devrait etre en IDLE = {:#x?}", state);

    dw3000.ll().aon_dig_cfg().write(|w| w.onw_pgfcal(1));

    delay.delay_ms(1000u16);


    loop {
        let mut receiving = dw3000.receive()
                        .expect("Failed configure receiver.");

        rprintln!("receiver = {:?}", receiving);


        let cmd_status = receiving.ll().fcmd_stat().read().unwrap().value();
        rprintln!("cmd_status = {:#x?}", cmd_status);
        let state = receiving.ll().sys_state().read().unwrap().pmsc_state();
        rprintln!("state = {:#x?}", state);

        // on cré un buffer pour stoquer le resultat message du receveur
        let mut buffer = [0; 1024];
        delay.delay_ms(10u8);

        // on recupère un message avec une fonction bloquante
        rprintln!("on commence une fonction qui bloque !");
        let result = block!(receiving.wait(&mut buffer));
        rprintln!("on est sorti de la fonction qui bloque !");

        // on affiche le resultat
        rprintln!("result = {:?}", result);

        dw3000 = receiving.finish_receiving()
                .expect("Failed to finish receiving");
    }    

}
