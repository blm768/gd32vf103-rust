#![no_std]
#![no_main]

extern crate panic_halt;

use core::convert::Infallible;

use atat::atat_derive::{AtatCmd, AtatResp};
use atat::{AtatClient, ClientBuilder, ComQueue, NoopUrcMatcher, Queues, ResQueue, UrcQueue};

use embedded_hal::serial::{Read, Write};

use gd32vf103xx_hal as hal;
use hal::pac;
use hal::pac::USART1;
use hal::prelude::*;
use hal::serial::{Config, Rx, Serial};
use hal::timer::Timer;

use heapless::consts;
use heapless::spsc::Queue;

use nb::block;

use riscv_rt::entry;

use gd32vf103_test::{PinMode, StatusLed};

fn tx_bytes<T: Write<u8>>(cmd: impl IntoIterator<Item = u8>, tx: &mut T) -> Result<(), T::Error> {
    for byte in cmd.into_iter() {
        block!(tx.write(byte))?;
    }
    Ok(())
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+RST", ResetResponse)]
pub struct Reset;

#[derive(Clone, AtatResp)]
pub struct ResetResponse;

static mut INGRESS: Option<atat::IngressManager> = None;
static mut RX: Option<Rx<USART1>> = None;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut rcu = dp.RCU.configure().freeze();
    let gpioa = dp.GPIOA.split(&mut rcu);
    let pa8 = gpioa.pa8.into_push_pull_output();
    let mut status = StatusLed::new(pa8);

    let mut afio = dp.AFIO.constrain(&mut rcu);
    let dbg_tx = gpioa.pa9;
    let dbg_rx = gpioa.pa10;
    let dbg_serial = Serial::new(
        dp.USART0,
        (dbg_tx, dbg_rx),
        Config::default().baudrate(115200.bps()),
        &mut afio,
        &mut rcu,
    );
    let (mut dbg_tx, _) = dbg_serial.split();

    //tx_bytes("Initialized serial\r\n".bytes(), &mut dbg_tx)
    //.unwrap_or_else(|_: Infallible| unreachable!());

    let ser_tx = gpioa.pa2;
    let ser_rx = gpioa.pa3;
    let serial = Serial::new(
        dp.USART1,
        (ser_tx, ser_rx),
        Config::default().baudrate(115200.bps()),
        &mut afio,
        &mut rcu,
    );

    static mut RES_QUEUE: ResQueue<consts::U256, consts::U5> = Queue(heapless::i::Queue::u8());
    static mut URC_QUEUE: UrcQueue<consts::U256, consts::U10> = Queue(heapless::i::Queue::u8());
    static mut COM_QUEUE: ComQueue<consts::U3> = Queue(heapless::i::Queue::u8());

    let queues = Queues {
        res_queue: unsafe { RES_QUEUE.split() },
        urc_queue: unsafe { URC_QUEUE.split() },
        com_queue: unsafe { COM_QUEUE.split() },
    };

    let (mut tx, mut rx) = serial.split();

    let timer = Timer::timer1(dp.TIMER1, 1.hz(), &mut rcu);

    tx_bytes("Initialized serial\r\n".bytes(), &mut dbg_tx)
        .unwrap_or_else(|_: Infallible| unreachable!());

    let (mut client, ingress) = ClientBuilder::new(
        tx,
        timer,
        |t| t.khz().into(),
        atat::Config::new(atat::Mode::Timeout),
    )
    .with_custom_urc_matcher(NoopUrcMatcher {})
    .build(queues);

    unsafe { INGRESS = Some(ingress) };
    unsafe { RX = Some(rx) };

    client.send(&Reset).unwrap();

    status.set_mode(PinMode::High);

    loop {}
}

/*
#[interrupt]
fn TIM7() {
    let ingress = unsafe { INGRESS.as_mut().unwrap() };
    ingress.parse_at();
}

#[interrupt]
fn USART2() {
    let ingress = unsafe { INGRESS.as_mut().unwrap() };
    let rx = unsafe { RX.as_mut().unwrap() };
    if let Ok(d) = nb::block!(rx.read()) {
        ingress.write(&[d]);
    }
}
*/
