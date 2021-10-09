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

use dw3000::hl;
use dw3000::RxConfig;



#[entry]
fn main() -> ! {

    rtt_init_print!();
    rprintln!("Coucou copain !\n\n");

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

    // NEW
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);

    // UWB module reset 
    rst_n.set_low().unwrap();
    rst_n.set_high().unwrap();

    /****************************************************************************************/
    /*****************              CONFIGURATION du DW3000               *******************/
    /****************************************************************************************/

    let mut dw3000 = hl::DW1000::new(spi, cs);

    // variables pour recuperer l'etat du module
    let mut state = dw3000.ll().sys_state().read().unwrap().pmsc_state();
    let mut rcinit = dw3000.ll().sys_status().read().unwrap().rcinit();
    let mut spirdy = dw3000.ll().sys_status().read().unwrap().spirdy();
    let mut cplock = dw3000.ll().sys_status().read().unwrap().cplock();
    if rcinit == 0x01 {
        rprintln!("Après la fonction new, on est dans l'état INIT_RC (rcinit = 1)");
    }
    if spirdy == 0x01 {
        rprintln!("Après la fonction new, on est dans l'état IDLE_RC (spirdy = 1)");
    }
    if cplock == 0x01 {
        rprintln!("Après la fonction new, on est dans l'état IDLE_PLL (cpclock = 1)");
    }
    rprintln!("Après la fonction new, on est dans l'état {:#x?}\n\n", state);


    delay.delay_ms(1000u16);
    state = dw3000.ll().sys_state().read().unwrap().pmsc_state();
    rcinit = dw3000.ll().sys_status().read().unwrap().rcinit();
    spirdy = dw3000.ll().sys_status().read().unwrap().spirdy();
    cplock = dw3000.ll().sys_status().read().unwrap().cplock();
    if rcinit == 0x01 {
        rprintln!("Après un delay, on est dans l'état INIT_RC (rcinit = 1)");
    }
    if spirdy == 0x01 {
        rprintln!("Après un delay, on est dans l'état IDLE_RC (spirdy = 1)");
    }
    if cplock == 0x01 {
        rprintln!("Après un delay, on est dans l'état IDLE_PLL (cpclock = 1)");
    }
    rprintln!("Après un delay, on est dans l'état {:#x?}\n\n", state);


    // activation de la calibration auto
    // dw3000.ll().aon_dig_cfg().write(|w| w.onw_pgfcal(1));

    // INIT
    let mut dw3000 = dw3000.init(&mut delay).expect("Failed init.");
    state = dw3000.ll().sys_state().read().unwrap().pmsc_state();
    rcinit = dw3000.ll().sys_status().read().unwrap().rcinit();
    spirdy = dw3000.ll().sys_status().read().unwrap().spirdy();
    cplock = dw3000.ll().sys_status().read().unwrap().cplock();
    if rcinit == 0x01 {
        rprintln!("Après un init, on est dans l'état INIT_RC (rcinit = 1)");
    }
    if spirdy == 0x01 {
        rprintln!("Après un init, on est dans l'état IDLE_RC (spirdy = 1)");
    }
    if cplock == 0x01 {
        rprintln!("Après un init, on est dans l'état IDLE_PLL (cpclock = 1)");
    }
    rprintln!("Après la fonction init, on est dans l'état {:#x?}\n\n", state);


// CONF DE LA PLL pour passer en mode IDLE_PLL
    
    // set CAL_EN in PLL_CAL register
    dw3000.ll().pll_cal().write(|w| w.cal_en(1))
            .expect("Write to PLL_CAL failed");
    //clear CP_LOCK

    // In CLK_CTRL sub register, the 2 bits of SYS_CLK are set to AUTO

    // set AINIT2IDLE 
    dw3000.ll().seq_ctrl().write(|w| w.ainit2idle(1))
            .expect("Write to AINIT2IDLE failed");
    // wait for CP_LOCK = 1



    // set PLL_CFG (4 bytes) / PLL8CFG8CH
    dw3000.ll().pll_cfg().write(|w| w.value(0x1F3C))
            .expect("Write 0x1F3C to PLL_CFG failed");
    // if channel 9, blabla
    // setting rf_tx_ctrl_1
    dw3000.ll().rf_tx_ctrl_1().write(|w| w.value(0x0E))
            .expect("Write 0x0E to rf_tx_ctrl_1 failed");
    // setting pll_cal
    dw3000.ll().pll_cal().modify(|_,w| 
        w
            .pll_cfg_ld(0x81)
    )
    .expect("Write 0x81 to pll_cfg_ld failed");
    // clearing cp_lock
    dw3000.ll().sys_status().write(|w| w.cplock(1))
            .expect("Write to cp_lock failed");
    delay.delay_ms(1000u16);
    cplock = dw3000.ll().sys_status().read().unwrap().cplock();
        rprintln!("la pll est elle lock ? = {:#x?}", cplock);

    // PLL
    // CLK_CTRL, SYS_CLK to auto
    dw3000.ll().clk_ctrl().modify(|_,w| w.sys_clk(0))
            .expect("Write to SYS_CLK failed");
    // set ainit2idle
    dw3000.ll().seq_ctrl().modify(|_,w| w.ainit2idle(1))
            .expect("Write to ainit2idle failed");
    // check CPLOCK

    delay.delay_ms(1000u16);
    //dw3000.ll().fast_command(0);




    // ON PASSE EN MODE RECEVEUR 
    let mut receiving = dw3000.receive(RxConfig {
                frame_filtering: false,
                .. RxConfig::default()
            })
            .expect("Failed configure receiver.");
    state = receiving.ll().sys_state().read().unwrap().pmsc_state();
    rprintln!("On passe en mode reception = {:#x?}", state);




    delay.delay_ms(1000u16);
    let rx_state = receiving.ll().sys_state().read().unwrap().rx_state();
    rprintln!("\nOn regarde ou en est le receveur\n" );
    rprintln!("Etat ? : {}", rx_state);



    loop {
        
        delay.delay_ms(10000u16);

    }    

}
