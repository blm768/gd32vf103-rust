#![no_std]
#![no_main]

extern crate panic_halt;

use gd32vf103xx_hal as hal;
use hal::pac;
use hal::prelude::*;

use riscv_rt::entry;

use gd32vf103_test::{PinMode, StatusLed};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut rcu = dp.RCU.configure().freeze();
    let gpioa = dp.GPIOA.split(&mut rcu);
    let pa8 = gpioa.pa8.into_push_pull_output();
    let mut status = StatusLed::new(pa8);
    status.set_mode(PinMode::High);
    loop {}
}
