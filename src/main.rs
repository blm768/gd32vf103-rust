#![no_std]
#![no_main]

extern crate panic_halt;

use gd32vf103_hal as hal;
use hal::pac;
use hal::prelude::*;
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut rcu = dp.RCU.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcu.apb2);
    let mut pa8 = gpioa.pa8.into_push_pull_output(&mut gpioa.ctl1);
    pa8.set_high().unwrap();
    loop {}
}
