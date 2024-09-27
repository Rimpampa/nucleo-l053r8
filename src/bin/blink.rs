#![no_main]
#![no_std]

use nucleo_l053r8 as _;

use core::cell::Cell;

use cortex_m::asm;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use stm32l0xx_hal::{
    gpio::*,
    pac::{self, interrupt, Interrupt},
    prelude::*,
    rcc::Config,
    timer::Timer,
};

static LED: Mutex<Cell<Option<gpioa::PA5<Output<PushPull>>>>> = Mutex::new(Cell::new(None));
static TIMER: Mutex<Cell<Option<Timer<pac::TIM2>>>> = Mutex::new(Cell::new(None));

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Configure the clock.
    let mut rcc = dp.RCC.freeze(Config::hsi16());

    // Acquire the GPIOA peripheral. This also enables the clock for GPIOA in
    // the RCC register.
    let gpioa = dp.GPIOA.split(&mut rcc);

    // Configure PA5 as output.
    let led = gpioa.pa5.into_push_pull_output();

    // Configure the timer.
    let mut timer = dp.TIM2.timer(1.Hz(), &mut rcc);
    timer.listen();

    // Store the LED and timer in mutex refcells to make them available from the
    // timer interrupt.
    cortex_m::interrupt::free(|cs| {
        LED.borrow(cs).set(Some(led));
        TIMER.borrow(cs).set(Some(timer));
    });

    // Enable the timer interrupt in the NVIC.
    unsafe {
        NVIC::unmask(Interrupt::TIM2);
    }

    loop {
        asm::wfi();
    }
}

#[interrupt]
#[allow(non_snake_case)]
fn TIM2() {
    // Keep a state to blink the LED.
    static mut LED_STATE: bool = false;

    cortex_m::interrupt::free(|cs| {
        let mut timer = TIMER.borrow(cs).replace(None)?;
        timer.clear_irq();
        TIMER.borrow(cs).set(Some(timer));

        let mut led = LED.borrow(cs).replace(None)?;
        LED_STATE
            .then(|| led.set_low())
            .unwrap_or_else(|| led.set_high())
            .unwrap();
        *LED_STATE = !*LED_STATE;
        LED.borrow(cs).set(Some(led));

        Some(())
    })
    .unwrap()
}
