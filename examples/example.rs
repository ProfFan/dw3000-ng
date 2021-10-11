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
	spi::{Mode, Phase, Polarity, Spi},
};
use embedded_hal::digital::v2::OutputPin;
use dw3000::hl;
// use dw3000::time::{TIME_MAX,Instant,};
use dw3000::RxConfig;

#[entry]
fn main() -> ! {
	rtt_init_print!();
	rprintln!("Coucou copain !");

	/******************************************************* */
	/************       CONFIGURATION DE BASE     ********** */
	/******************************************************* */

	// Get access to the device specific peripherals from the peripheral access crate
	let dp = pac::Peripherals::take().unwrap();
	let cp = cortex_m::Peripherals::take().unwrap();

	// Take ownership over the raw flash and rcc devices and convert them into the corresponding
	// HAL structs
	let mut flash = dp.FLASH.constrain();
	let mut rcc = dp.RCC.constrain();
	let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

	let clocks = rcc
		.cfgr
		.use_hse(8.mhz())
		.sysclk(36.mhz())
		.freeze(&mut flash.acr);

	let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
	let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

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
		&mut rcc.apb2,
	);
	
	/****************************************************** */
	/*****       CONFIGURATION DU RESET du DW3000   ******* */
	/****************************************************** */

	let mut delay = Delay::new(cp.SYST, clocks);

	let mut rst_n = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);

	// UWB module reset
	rst_n.set_low().unwrap();
	rst_n.set_high().unwrap();


	/****************************************************** */
	/*********       CONFIGURATION du DW3000       ******** */
	/****************************************************** */

	let mut dw3000 = hl::DW1000::new(spi, cs);
	delay.delay_ms(1000u16);

	// variable pour recuperer l'etat du module
	rprintln!(
		"Etat après un new et une attente de 1sec = {:#x?}",
		dw3000.state()
	);

	// activation de la calibration auto
	dw3000
		.ll()
		.aon_dig_cfg()
		.write(|w| w.onw_pgfcal(1))
		.expect("Write to onw_pgfcal failed.");

	rprintln!(
		"On est dans l'état IDLE_RC -> SPIRDY = {:#x?}",
		dw3000.idle_rc_passed()
	);

	// On initialise le module pour passer à l'état IDLE
	rprintln!("On fait maintenant un init !");
	let mut dw3000 = dw3000.init(&mut delay).expect("Failed init.");
	delay.delay_ms(1000u16);
	rprintln!("Après l'init, l'état est = {:#x?}", dw3000.state());

	// après ces états, on peux vérifier l'etat du systeme avec les reg:
	// SPIRDY -> indique qu'on a finit les config d'allumage (IDLE_RC)
	rprintln!(
		"Est ce qu'on est dans l'état IDLE_RC ? = {:#x?}",
		dw3000.idle_rc_passed()
	);

	// CPLOCK -> indique si l'horloge PLL est bloquée (IDLE_PLL)
	rprintln!(
		"Est ce qu'on est dans l'état IDLE_PLL ? = {:#x?}",
		dw3000.idle_pll_passed()
	);

	// PLL_HILO -> indique un probleme sur la conf de PLL
	rprintln!(
		"Probleme pour lock la PLL ? = {:#x?}",
		dw3000.ll().sys_status().read().unwrap().pll_hilo()
	);

	delay.delay_ms(5000u16);

	// let valid_instant   = Instant::new(TI*ME_MAX);

	// ON PASSE EN MODE RECEVEUR
	let mut receiving = dw3000
		.receive(RxConfig {
			frame_filtering: false,
			..RxConfig::default()
		})
		.expect("Failed configure receiver.");
	rprintln!("On passe en mode reception = {:#x?}", receiving.state());

	// ETAPE 1 : recherche du preamble
	// fait automatiquement, on desactive le time-out pre-toc (default)

	// ETAPE 2 : Accumulation preamble and await SFD

	// ETAPE 3 :

	delay.delay_ms(1000u16);
	rprintln!("\nOn regarde ou en est le receveur\n");
	rprintln!("Etat ? : {:#x?}", receiving.rx_state());

	loop {
		delay.delay_ms(2000u16);
		rprintln!("Etat ? : {:#x?}", receiving.rx_state());
		//frame_ready = receiving.ll().sys_status().read().unwrap().rxfr();

		rprintln!(
			"tested value = {:#x?}",
			receiving.ll().ldo_rload().read().unwrap().value()
		);
	}
}
