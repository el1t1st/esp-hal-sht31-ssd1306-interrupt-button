#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    // the self enables the use of gpio
    gpio::{self, Event, Input, Io, Level, Output, Pull},
    macros::ram,
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};
use esp_println::println;

static BUTTON: Mutex<RefCell<Option<Input<gpio::Gpio1>>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<Output<gpio::Gpio2>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let _delay = Delay::new(&clocks);

    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    io.set_interrupt_handler(handler);

    // set up a button
    let mut button = Input::new(io.pins.gpio1, Pull::Up);

    // set up critical section for the button
    critical_section::with(|cs| {
        button.listen(Event::FallingEdge);
        BUTTON.borrow_ref_mut(cs).replace(button)
    });

    // set up a led
    let led = Output::new(io.pins.gpio2, Level::Low);
    critical_section::with(|cs| LED.borrow_ref_mut(cs).replace(led));

    // set up an i2c

    loop {
        // println!("Privet Mir!");
        // delay.delay(500.millis());
    }
}

#[handler]
#[ram]
fn handler() {
    println!("fired up the interrupt handler GPIO");

    critical_section::with(|cs| {
        // since the LEd is a static variable it is accessible
        LED.borrow_ref_mut(cs).as_mut().unwrap().toggle();
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt()
    });
}
