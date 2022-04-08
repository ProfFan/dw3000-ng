#![no_main]
#![no_std]

use example_stm32f1 as _; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    example_stm32f1::exit()
}