#![no_main]
#![no_std]

use cortex_m::asm;
use embedded_time::rate::Hertz;
use nucleo_l053r8 as _;

use cortex_m_rt::entry;
use stm32l0xx_hal::{
    gpio::*,
    pac,
    prelude::*,
    rcc::{AHBPrescaler, ClockSrc, Config, HSI16Div, PLLDiv, PLLMul, PLLSource, APBPrescaler},
    spi,
};

/*
    [DS11896 - Rev 10, 4.10 Digital interface specification]:
    The SPI interface works at a maximum frequency of 10MHz

    [DB3512 - Rev 3, figure 1. STEVAL-FKI868V2 circuit schematic]:
    The pins SPI pins are connected to the CN5 connector, in this way:
    - CSN = 3
    - SDI/MOSI = 4
    - SDO/MISO = 5
    - SCLK = 6

    [UM1724 - Rev 14, Figure 20. NUCLEO-L053R8]:
    Those pins are connected to those GPIO ports:
    - SCLK = A5
    - SDO/MISO = A6
    - SDI/MOSI = A7
    - CSN = B6

    [DS10152 - Rev 9, Table 16. Alternate function port A]:
    The A5, A6 and A7 GPIO pins, to work as intended, have to be configured
    in alternate function mode 0 (AF0)

    [DS11896 - Rev 10, 9.1 Serial peripheral interface]:
    CPOL and CPHA have to be both 0 (at the start atleast)

    The NUCLEO specs has a note about changing CPOL and CPHA:
    > Prior to changing the CPOL/CPHA bits the SPI must be disabled by resetting the SPE bit.
    The LSBFIRST bit can be used to selecte which bit is sent first, if MSb or LSb
    The SPI_CR1::DFF can be used to select whether the data frames are 8 or 16 bits

*/

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Configure the clock.
    // let mut rcc = dp.RCC.freeze(
    //     Config::pll(PLLSource::HSI16(HSI16Div::Div1), PLLMul::Mul4, PLLDiv::Div2)
    //         .ahb_pre(AHBPrescaler::NotDivided)
    //         .apb1_pre(APBPrescaler::NotDivided)
    //         .apb2_pre(APBPrescaler::NotDivided)
    // );
    let mut rcc = dp.RCC.freeze(
        Config::hsi16()
    );

    let pre = rcc.clocks.apb2_clk().0 / Hertz::<u32>::try_from(8.MHz()).unwrap().0;
    defmt::println!("PRE: {}", pre);

    // Acquire the GPIOA peripheral.
    // This also enables the clock for GPIOA in the RCC register.
    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    let sck = gpiob.pb3; // gpioa.pa5;
    let miso = gpioa.pa6;
    let mosi = gpioa.pa7;
    let mut sdn = gpioa.pa8.into_push_pull_output();
    let mut csn = gpioa
        .pa1 /* gpiob.pb6 */
        .into_push_pull_output();

    sdn.set_high().unwrap();
    defmt::println!("SDN high");

    csn.set_high().unwrap();
    defmt::println!("CSN high");

    let mut spi = dp.SPI1.spi(
        (sck, miso, mosi),
        spi::MODE_0,
        Hertz::try_from(8.MHz()).unwrap(),
        &mut rcc,
    );
    defmt::println!("SPI setup");

    sdn.set_low().unwrap();
    defmt::println!("SDN low");

    csn.set_low().unwrap();
    defmt::println!("CSN low");

    nb::block!(spi.send(0b00000001)).unwrap();
    defmt::println!("SPI send header");

    let status0 = nb::block!(spi.read()).unwrap();
    defmt::println!("status0 = {}", status0);

    nb::block!(spi.send(0x00)).unwrap();
    defmt::println!("SPI send address");

    let status1 = nb::block!(spi.read()).unwrap();
    defmt::println!("status1 = {}", status1);

    nb::block!(spi.send(0)).unwrap();
    defmt::println!("send 0");

    let partnum = nb::block!(spi.read()).unwrap();
    defmt::println!("gpio0conf = {:b}", partnum);

    nb::block!(spi.send(0)).unwrap();
    defmt::println!("send 0");

    let version = nb::block!(spi.read()).unwrap();
    defmt::println!("gpio1conf = {:b}", version);

    csn.set_high().unwrap();
    defmt::println!("CSN high");

    loop {}
}
