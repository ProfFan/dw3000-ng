#![no_main]
#![no_std]

use stm32f1 as _; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    stm32f1::exit()
}
