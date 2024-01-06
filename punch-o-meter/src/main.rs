#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use embedded_hal::{timer::CountDown, blocking::delay::DelayMs};
use lsm303agr::Lsm303agr;
use microbit::hal::{twim, Timer};
use nb::Error;
use panic_rtt_target as _;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;

#[entry]
fn main() -> ! {
    const THRESHOLD: f32 = 0.5;

    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let i2c = {
        twim::Twim::new(
            board.TWIM0,
            board.i2c_internal.into(),
            twim::Frequency::K100,
        )
    };

    let mut countdown = Timer::new(board.TIMER0);
    let mut delay = Timer::new(board.TIMER1);
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor
        .set_accel_odr(lsm303agr::AccelOutputDataRate::Hz50)
        .unwrap();
    sensor.set_accel_scale(lsm303agr::AccelScale::G16).unwrap();

    let mut max_g: f32 = 0.0;
    let mut measuring = false;

    loop {
        while !sensor.accel_status().unwrap().xyz_new_data {}
        let g_x = sensor.accel_data().unwrap().x as f32 / 1000.0;

        if measuring {
            match countdown.wait() {
                Err(Error::WouldBlock) => {
                    if g_x > max_g {
                        max_g = g_x;
                    }
                }
                Ok(_) => {
                    rprintln!("Max acceleration: {}g", max_g);

                    max_g = 0.0;
                    measuring = false
                }
                Err(Error::Other(_)) => {
                    unreachable!()
                }
            }
        } else {
            if g_x > THRESHOLD {
                rprintln!("START!");

                measuring = true;
                max_g = g_x;
                countdown.start(1_000_000u32);
            }
        }
        delay.delay_ms(20u8)
    }
}
