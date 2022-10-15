#![no_main]
#![no_std]

use nucleo_l053r8 as _; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    nucleo_l053r8::exit()
}
