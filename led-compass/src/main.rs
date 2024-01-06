#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::f32::consts::PI;

use cortex_m_rt::entry;
use libm::{atan2f, sqrtf};
use lsm303agr::Lsm303agr;
use microbit::{
    display::blocking::Display,
    hal::{twim, Timer},
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use crate::{
    calibration::{calc_calibration, calibrated_measurement},
    led::{direction_to_led, Direction},
};

mod calibration;
mod led;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let i2c = {
        twim::Twim::new(
            board.TWIM0,
            board.i2c_internal.into(),
            twim::Frequency::K100,
        )
    };
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor
        .set_mag_odr(lsm303agr::MagOutputDataRate::Hz10)
        .unwrap();
    sensor
        .set_accel_odr(lsm303agr::AccelOutputDataRate::Hz10)
        .unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    let calibration = calc_calibration(&mut sensor, &mut display, &mut timer);
    rprintln!("Calibration: {:?}", calibration);
    rprintln!("Calibration done, entering busy loop");

    loop {
        while !sensor.mag_status().unwrap().xyz_new_data {}
        let mut data = sensor.mag_data().unwrap();
        data = calibrated_measurement(data, &calibration);

        let x = data.x as f32;
        let y = data.y as f32;
        let z = data.z as f32;
        let magnitude = sqrtf(x * x + y * y + z * z);
        rprintln!("{} nT, {} mG", magnitude, magnitude / 100.0);

        let theta = atan2f(data.y as f32, data.x as f32);
        let dir = if theta < -7. * PI / 8. {
            Direction::West
        } else if theta < -5. * PI / 8. {
            Direction::SouthWest
        } else if theta < -3. * PI / 8. {
            Direction::South
        } else if theta < -PI / 8. {
            Direction::SouthEast
        } else if theta < PI / 8. {
            Direction::East
        } else if theta < 3. * PI / 8. {
            Direction::NorthEast
        } else if theta < 5. * PI / 8. {
            Direction::North
        } else if theta < 7. * PI / 8. {
            Direction::NorthWest
        } else {
            Direction::West
        };

        display.show(&mut timer, direction_to_led(dir), 100);
    }
}
