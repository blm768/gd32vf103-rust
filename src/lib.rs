#![no_std]

use embedded_hal::digital::v2::OutputPin;

use gd32vf103xx_hal as hal;
use hal::gpio::gpioa::PA8;
use hal::gpio::{Output, PushPull};

pub enum PinMode {
    Low,
    High,
}

pub struct StatusLed {
    pin: PA8<Output<PushPull>>,
}

impl StatusLed {
    pub fn new(pin: PA8<Output<PushPull>>) -> Self {
        Self { pin }
    }

    pub fn set_mode(&mut self, mode: PinMode) {
        match mode {
            PinMode::Low => self.pin.set_low().unwrap(),
            PinMode::High => self.pin.set_high().unwrap(),
        }
    }
}
