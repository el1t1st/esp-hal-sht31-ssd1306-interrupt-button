#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;
use embedded_graphics::{
    geometry::Dimensions,
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::Point,
    text::{Alignment, Text},
    Drawable,
};
use ssd1306::{
    prelude::*, rotation::DisplayRotation, size::DisplaySize128x64, I2CDisplayInterface, Ssd1306,
};

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    // the self enables the use of gpio
    gpio::{self, Event, Input, Io, Level, Output, Pull},
    i2c::I2C,
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

    // Connect the ssd1306 display
    let sda = io.pins.gpio4;
    let scl = io.pins.gpio5;

    // set up an i2c connection for the display
    let i2c_display = I2C::new(peripherals.I2C0, sda, scl, 100.kHz(), &clocks, None);

    let interface = I2CDisplayInterface::new(i2c_display);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    // Specify text style
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_alignment(
        "INTERRUPT EXPERIMENT",
        display.bounding_box().center() + Point::new(0, 14),
        text_style,
        Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();

    display.flush().unwrap();

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
