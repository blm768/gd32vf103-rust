#![no_std]
#![no_main]

extern crate panic_halt;

use embedded_hal::digital::v2::OutputPin;
use gd32vf103xx_hal as hal;
use hal::pac;
use hal::prelude::*;
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut rcu = dp.RCU.configure().freeze();
    let gpioa = dp.GPIOA.split(&mut rcu);
    let mut pa8 = gpioa.pa8.into_push_pull_output();
    pa8.set_high().unwrap();
    loop {}
}
