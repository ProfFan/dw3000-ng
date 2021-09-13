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

    // test du registre SPI_RD_CRC
    let spi_rd_crc = dw3000.ll().spi_rd_crc().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("SPI_RD_CRC = {:?}", spi_rd_crc);

    // test du registre SYS_TIME
    let sys_time = dw3000.ll().sys_time().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("SYS_TIME = {:?}", sys_time);

    // test du registre TX_FCTRL
    let tx_fctrl = dw3000.ll().tx_fctrl().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("TX_FCTRL = {:?}", tx_fctrl);

    // test du registre DX_TIME
    let dx_time = dw3000.ll().dx_time().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("DX_TIME = {:?}", dx_time);

    // test du registre DREF_TIME
    let dref_time = dw3000.ll().dref_time().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("DREF_TIME = {:?}", dref_time);

    // test du registre RX_FWTO
    let rx_fwto = dw3000.ll().rx_fwto().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("RX_FWTO = {:?}", rx_fwto);

    // test du registre SYS_CTRL
    let sys_ctrl = dw3000.ll().sys_ctrl().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("SYS_CTRL = {:?}", sys_ctrl);

    // test du registre SYS_ENABLE
    let sys_enable = dw3000.ll().sys_enable().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("SYS_ENABLE = {:?}", sys_enable);

    // test du registre SYS_STATUS
    let sys_status = dw3000.ll().sys_status().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("SYS_STATUS = {:?}", sys_status);

    // test du registre RX_FINFO
    let rx_finfo = dw3000.ll().rx_finfo().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("RX_FINFO = {:?}", rx_finfo);

    // test du registre RX_TIME
    let rx_time = dw3000.ll().rx_time().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("RX_TIME = {:?}", rx_time);

    // test du registre TX_TIME
    let tx_time = dw3000.ll().tx_time().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("TX_TIME = {:?}", tx_time);




    // test du registre TX_RAWST
    let tx_rawst = dw3000.ll().tx_rawst().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("TX_RAWST = {:?}", tx_rawst);

    // test du registre TX_ANTD
    let tx_antd = dw3000.ll().tx_antd().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("TX_ANTD = {:?}", tx_antd);

    // test du registre ACK_RESP
    let ack_resp = dw3000.ll().ack_resp().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("ACK_RESP = {:?}", ack_resp);

    // test du registre TX_POWER
    let tx_power = dw3000.ll().tx_power().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("TX_POWER = {:?}", tx_power);

    // test du registre CHAN_CTRL
    let chan_ctrl = dw3000.ll().chan_ctrl().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("CHAN_CTRL = {:?}", chan_ctrl);

    // test du registre LE_PEND_01
    let le_pend_01 = dw3000.ll().le_pend_01().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("LE_PEND_01 = {:?}", le_pend_01);

    // test du registre LE_PEND_23
    let le_pend_23 = dw3000.ll().le_pend_23().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("LE_PEND_23 = {:?}", le_pend_23);

    // test du registre SPI_COLLISION
    let spi_collision = dw3000.ll().spi_collision().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("SPI_COLLISION = {:?}", spi_collision);

    // test du registre RDB_STATUS
    let rdb_status = dw3000.ll().rdb_status().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("RDB_STATUS = {:?}", rdb_status);

    // test du registre RDB_DIAG
    let rdb_diag = dw3000.ll().rdb_diag().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("RDB_DIAG = {:?}", rdb_diag);

    // test du registre AES_CFG
    let aes_cfg = dw3000.ll().aes_cfg().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("AES_CFG = {:?}", aes_cfg);

    // test du registre AES_IV0
    let aes_iv0 = dw3000.ll().aes_iv0().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("AES_IV0 = {:?}", aes_iv0);

    // test du registre AES_IV1
    let aes_iv1 = dw3000.ll().aes_iv1().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("AES_IV1 = {:?}", aes_iv1);

    // test du registre AES_IV2
    let aes_iv2 = dw3000.ll().aes_iv2().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("AES_IV2 = {:?}", aes_iv2);

    // test du registre AES_IV3
    let aes_iv3 = dw3000.ll().aes_iv3().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("AES_IV3 = {:?}", aes_iv3);

    // test du registre AES_IV4
    let aes_iv4 = dw3000.ll().aes_iv4().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("AES_IV4 = {:?}", aes_iv4);

    // test du registre DMA_CFG
    let dma_cfg = dw3000.ll().dma_cfg().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("DMA_CFG = {:?}", dma_cfg);

        // test du registre AES_START
    let aes_start = dw3000.ll().aes_start().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("AES_START = {:?}", aes_start);

        // test du registre AES_STS
    let aes_sts = dw3000.ll().aes_sts().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("AES_STS = {:?}", aes_sts);

        // test du registre AES_KEY
    let aes_key = dw3000.ll().aes_key().read()
        .expect("Failed to read DEV_ID register");
    rprintln!("AES_KEY = {:?}", aes_key);


    loop {
        
    }    

}
