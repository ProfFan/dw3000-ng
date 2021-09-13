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

use dw3000::{hl};

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

    let mut dw3000 = hl::DW1000::new(spi, cs);
    rprintln!("dm3000 = {:?}", dw3000);

    /* // BLOQUE !!!!!
    let mut dw3000 = hl::DW1000::new(spi, cs).init()
        .expect("Failed to initialize DW1000");
    rprintln!("dm3000 = {:?}", dw3000);
    */

    let gen_cfg_aes = dw3000.ll().gen_cfg_aes().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("gen_cfg_aes = {:?}", gen_cfg_aes);


    // test du registre dev_id
    let dev_id = dw3000.ll().dev_id().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("dev-id = {:?}", dev_id);

    // test du registre EUI
    let eui = dw3000.ll().eui().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("eui = {:?}", eui);

    // test du registre PANADR
    let panadr = dw3000.ll().panadr().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("panadr = {:?}", panadr);

    // test du registre SYS_CFG
    let sys_cfg = dw3000.ll().sys_cfg().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("SYS_CFG = {:?}", sys_cfg);

    // test du registre FF_CFG
    let ff_cfg = dw3000.ll().ff_cfg().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("FF_CFG = {:?}", ff_cfg);
   



    // test du registre DIG_DIAG
    let dig_dial = dw3000.ll().dig_dial().read()
        .expect("Failed to read DIG_DIAG register");
    rprintln!("DIG_DIAG = {:?}", dig_dial);
/*
    // test du registre PMSC_CTRL
    let pmsc_ctrl = dw3000.ll().pmsc_ctrl().read()
        .expect("Failed to read PMSC_CTRL register");
    rprintln!("PMSC_CTRL = {:?}", pmsc_ctrl);

    // test du registre SOFT_RST
    let soft_rst = dw3000.ll().soft_rst().read()
        .expect("Failed to read SOFT_RST register");
    rprintln!("SOFT_RST = {:?}", soft_rst);

    // test du registre CLK_CTRL
    let clk_ctrl = dw3000.ll().clk_ctrl().read()
        .expect("Failed to read CLK_CTRL register");
    rprintln!("CLK_CTRL = {:?}", clk_ctrl);

    // test du registre SEQ_CTRL
    let seq_ctrl = dw3000.ll().seq_ctrl().read()
        .expect("Failed to read SEQ_CTRL register");
    rprintln!("SEQ_CTRL = {:?}", seq_ctrl);

    // test du registre TXFSEQ
    let txfseq = dw3000.ll().txfseq().read()
        .expect("Failed to read TXFSEQ register");
    rprintln!("TXFSEQ = {:?}", txfseq);

    // test du registre LED_CTRL
    let led_ctrl = dw3000.ll().led_ctrl().read()
        .expect("Failed to read LED_CTRL register");
    rprintln!("LED_CTRL = {:?}", led_ctrl);

    // test du registre RX_SNIFF
    let rx_sniff = dw3000.ll().rx_sniff().read()
        .expect("Failed to read RX_SNIFF register");
    rprintln!("RX_SNIFF = {:?}", rx_sniff);

    // test du registre BIAS_CTRL
    let bias_ctrl = dw3000.ll().bias_ctrl().read()
        .expect("Failed to read BIAS_CTRL register");
    rprintln!("BIAS_CTRL = {:?}", bias_ctrl);

    // test du registre ACC_MEM
    let acc_mem = dw3000.ll().acc_mem().read()
        .expect("Failed to read ACC_MEM register");
    //    rprintln!("yes");
    rprintln!("ACC_MEM = {:?}", acc_mem);

    // test du registre SCRATCH_RAM
    let scratch_ram = dw3000.ll().scratch_ram().read()
        .expect("Failed to read SCRATCH_RAM register");
    rprintln!("SCRATCH_RAM = {:?}", scratch_ram);

    // test du registre AES_KEY_RAM
    let aes_key_ram = dw3000.ll().aes_key_ram().read()
        .expect("Failed to read AES_KEY_RAM register");
    rprintln!("AES_KEY_RAM = {:?}", aes_key_ram);

    // test du registre DB_DIAG
    let db_diag = dw3000.ll().db_diag().read()
        .expect("Failed to read DB_DIAG register");
    rprintln!("DB_DIAG = {:?}", db_diag);

    // test du registre DB_DIAG_SET1
    let db_diag_set1 = dw3000.ll().db_diag_set1().read()
        .expect("Failed to read DB_DIAG_SET1 register");
    rprintln!("DB_DIAG_SET1 = {:?}", db_diag_set1);

    // test du registre DB_DIAG_SET2
    let db_diag_set2 = dw3000.ll().db_diag_set2().read()
        .expect("Failed to read DB_DIAG_SET2 register");
    rprintln!("DB_DIAG_SET2 = {:?}", db_diag_set2);

    // test du registre INDIRECT_PTR_A
    let indirect_ptr_a = dw3000.ll().indirect_ptr_a().read()
        .expect("Failed to read INDIRECT_PTR_A register");
    rprintln!("INDIRECT_PTR_A = {:?}", indirect_ptr_a);

    // test du registre INDIRECT_PTR_B
    let indirect_ptr_b = dw3000.ll().indirect_ptr_b().read()
        .expect("Failed to read INDIRECT_PTR_B register");
    rprintln!("INDIRECT_PTR_B = {:?}", indirect_ptr_b);

    // test du registre IN_PTR_CFG
    let in_ptr_cfg = dw3000.ll().in_ptr_cfg().read()
        .expect("Failed to read IN_PTR_CFG register");
    rprintln!("IN_PTR_CFG = {:?}", in_ptr_cfg);

    // test du registre FINT_STAT 
    let fint_stat = dw3000.ll().fint_stat().read()
        .expect("Failed to read FINT_STAT register");
    rprintln!("FINT_STAT = {:?}", fint_stat);

    // test du registre PTR_ADDR_A
    let ptr_addr_a = dw3000.ll().ptr_addr_a().read()
        .expect("Failed to read PTR_ADDR_A register");
    rprintln!("PTR_ADDR_A = {:?}", ptr_addr_a);

    // test du registre PTR_OFFSET_A
    let ptr_offset_a = dw3000.ll().ptr_offset_a().read()
        .expect("Failed to read PTR_OFFSET_A register");
    rprintln!("PTR_OFFSET_A = {:?}", ptr_offset_a);

    // test du registre PTR_ADDR_B
    let ptr_addr_b = dw3000.ll().ptr_addr_b().read()
        .expect("Failed to read PTR_ADDR_B register");
    rprintln!("PTR_ADDR_B = {:?}", ptr_addr_b);

    // test du registre PTR_OFFSET_B
    let ptr_offset_b = dw3000.ll().ptr_offset_b().read()
        .expect("Failed to read PTR_OFFSET_B register");
    rprintln!("PTR_OFFSET_B = {:?}", ptr_offset_b);
*/
    loop {
        
    }    

}
