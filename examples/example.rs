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
    /*****************              CONFIGURATION DU RESET              *********************/
    /****************************************************************************************/

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);

    /****************************************************************************************/
    /*****************              CONFIGURATION du DW3000               *******************/
    /****************************************************************************************/

    let mut dw3000 = hl::DW1000::new(spi, cs).init()
                        .expect("Failed init.");
    rprintln!("dm3000 = {:?}", dw3000);


    // rst_n.set_low().unwrap();
    // rst_n.set_high().unwrap();

    delay.delay_ms(2u8);

    let ainit2idle = dw3000.ll().seq_ctrl().read().unwrap().ainit2idle();
    let onw_go2idle = dw3000.ll().aon_dig_cfg().read().unwrap().onw_go2idle();
    rprintln!("ainit2idle = {:?}", ainit2idle);
    rprintln!("onw_go2idle = {:?}", onw_go2idle);

    dw3000.ll().seq_ctrl().write(|w| w.ainit2idle(1)).unwrap();
    dw3000.ll().aon_dig_cfg().write(|w| w.onw_go2idle(1)).unwrap();
    
    while dw3000.ll().sys_status().read().unwrap().rcinit() == 0 {
        delay.delay_ms(2u8);
    };

    let ainit2idle = dw3000.ll().seq_ctrl().read().unwrap().ainit2idle();
    let onw_go2idle = dw3000.ll().aon_dig_cfg().read().unwrap().onw_go2idle();
    rprintln!("ainit2idle = {:?}", ainit2idle);
    rprintln!("onw_go2idle = {:?}", onw_go2idle);
    if (ainit2idle == 0) || (onw_go2idle == 0) {
        rprintln!("Init failed. Impossible to reach INIT_PLL.");
    } 
    delay.delay_ms(2u8);

    rprintln!("Is in Ready / IDLE_PLL.");


    dw3000.set_address( mac::PanId(0x0d57),
                        mac::ShortAddress(50))
            .expect("Failed to set address.");


    loop {
        
    }    

}
