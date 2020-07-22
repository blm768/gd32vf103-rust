#![no_std]
#![no_main]

extern crate panic_halt;

use core::convert::Infallible;

use embedded_hal::serial::{Read, Write};

use gd32vf103xx_hal as hal;
use hal::pac;
use hal::prelude::*;
use hal::serial::{Config, Serial};

use nb::block;

use riscv_rt::entry;

use gd32vf103_test::{PinMode, StatusLed};

fn tx_bytes<T: Write<u8>>(cmd: impl IntoIterator<Item = u8>, tx: &mut T) -> Result<(), T::Error> {
    for byte in cmd.into_iter() {
        block!(tx.write(byte))?;
    }
    Ok(())
}

fn tx_at_cmd<T: Write<u8>>(cmd: impl IntoIterator<Item = u8>, tx: &mut T) -> Result<(), T::Error> {
    tx_bytes(cmd, tx)?;
    block!(tx.write(b'\r'))?;
    block!(tx.write(b'\n'))?;
    block!(tx.flush())
}

fn rx_at_cmd<T: Read<u8>>(buf: &mut [u8], rx: &mut T) -> Result<usize, RxAtCmdError<T::Error>> {
    // TOOD: why aren't we getting these?
    /*
    let byte = block!(rx.read())?;
    if byte != b'\r' {
        panic!();
        return Err(RxAtCmdError::InvalidDelimiter(byte));
    }
    let byte = block!(rx.read())?;
    if byte != b'\n' {
        return Err(RxAtCmdError::InvalidDelimiter(byte));
    }
    */
    let mut count = 0;
    loop {
        let byte = block!(rx.read())?;
        match byte {
            // TODO: how are inline \r characters handled in AT commands?
            b'\r' => {
                let byte = b'\n';
                // TODO: Why not getting this?
                //let byte = block!(rx.read())?;
                match byte {
                    b'\n' => break,
                    _ => return Err(RxAtCmdError::InvalidDelimiter(byte)),
                }
            }
            _ => {
                if count >= buf.len() {
                    return Err(RxAtCmdError::TooLong);
                }
                buf[count] = byte;
                count += 1;
            }
        }
    }

    Ok(count)
}

enum RxAtCmdError<E> {
    InvalidDelimiter(u8),
    TooLong,
    Other(E),
}

impl<E> From<E> for RxAtCmdError<E> {
    fn from(e: E) -> Self {
        Self::Other(e)
    }
}

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

    tx_bytes("Initialized serial\r\n".bytes(), &mut dbg_tx)
        .unwrap_or_else(|_: Infallible| unreachable!());

    let ser_tx = gpioa.pa2;
    let ser_rx = gpioa.pa3;
    let serial = Serial::new(
        dp.USART1,
        (ser_tx, ser_rx),
        Config::default().baudrate(115200.bps()),
        &mut afio,
        &mut rcu,
    );

    let (mut tx, mut rx) = serial.split();
    tx_at_cmd("AT+RST".bytes(), &mut tx).unwrap_or_else(|_: Infallible| unreachable!());

    let mut recv_buf = [0u8; 64];
    let recv_count = rx_at_cmd(&mut recv_buf, &mut rx)
        .map_err(|_| ())
        .expect("Failed to reset modem");
    let rx_msg = &recv_buf[0..recv_count];

    tx_bytes(rx_msg.iter().copied(), &mut dbg_tx).unwrap();
    if rx_msg == b"OK" {
        status.set_mode(PinMode::High);
    }

    loop {}
}
