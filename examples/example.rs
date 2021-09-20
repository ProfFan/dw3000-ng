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
    /************              CONFIGURATION DU RESET du DW3000             *****************/
    /****************************************************************************************/

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);
    rst_n.set_low().unwrap();
    rst_n.set_high().unwrap();

    /****************************************************************************************/
    /*****************              CONFIGURATION du DW3000               *******************/
    /****************************************************************************************/

    let mut dw3000 = hl::DW1000::new(spi, cs).init()
                        .expect("Failed init.");
    rprintln!("dm3000 = {:?}", dw3000);

    rprintln!("cal_mode = {:#x?}",dw3000.ll().rx_cal().read().unwrap().cal_mode());
    dw3000.ll().rx_cal().write(|w| w.cal_mode(1)).unwrap();
    rprintln!("cal_mode = {:#x?}",dw3000.ll().rx_cal().read().unwrap().cal_mode());

    dw3000.ll().fast_command(2).unwrap();

/*
    dw3000.set_address( mac::PanId(0x0d57),
                        mac::ShortAddress(50))
            .expect("Failed to set address.");
*/


/*
    // ONW_PGFCAL : activate RX calibration on wake-up
    let onw_pgfcal = dw3000.ll().aon_dig_cfg().read().unwrap().onw_pgfcal();
    rprintln!("onw_pgfcal = {:?}", onw_pgfcal);

    dw3000.ll().aon_dig_cfg().write(|w|
                w.onw_pgfcal(1)); // activate RX config in wakeup

    rst_n.set_low().unwrap();
    rst_n.set_high().unwrap();

    delay.delay_ms(10u8);

    let onw_pgfcal = dw3000.ll().aon_dig_cfg().read().unwrap().onw_pgfcal();
    rprintln!("onw_pgfcal = {:?}", onw_pgfcal);


    // CAL_MODE : 0 for normal mode
    let cal_mode = dw3000.ll().rx_cal().read().unwrap().cal_mode();
    rprintln!("cal_mode = {:?}", cal_mode);
    // CAL_EN : calibration enable, RX_CAL_STS set when finish
    let cal_en = dw3000.ll().rx_cal().read().unwrap().cal_en();
    rprintln!("cal_en = {:?}", cal_en);
    let rx_cal_sts = dw3000.ll().rx_cal_sts().read().unwrap().value();
    rprintln!("rx_cal_sts = {:?}", rx_cal_sts);
    // COMP_DLY : doit etre mis à 0x2 pour optimisation
    let comp_dly = dw3000.ll().rx_cal().read().unwrap().comp_dly();
    rprintln!("comp_dly = {:?}", comp_dly);
*/


    loop {
        
    }    

}
