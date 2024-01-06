#![no_main]
#![no_std]

use core::fmt::Write;
use cortex_m_rt::entry;
use lsm303agr::Lsm303agr;
use microbit::hal::{prelude::*, twim, uarte};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

mod serial_setup;

use crate::serial_setup::UartePort;

// const ACCELEROMETER_ADDR: u8 = 0b0011001;
// const MAGNETOMETER_ADDR: u8 = 0b0011110;

// const ACCELEROMETER_ID_REG: u8 = 0x0f;
// const MAGNETOMETER_ID_REG: u8 = 0x4f;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            uarte::Parity::EXCLUDED,
            uarte::Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    write!(serial, "Microbit is online!!!\r\n");

    let mut i2c = {
        twim::Twim::new(
            board.TWIM0,
            board.i2c_internal.into(),
            twim::Frequency::K100,
        )
    };

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor
        .set_accel_odr(lsm303agr::AccelOutputDataRate::Hz10)
        .unwrap();
    sensor
        .set_mag_odr(lsm303agr::MagOutputDataRate::Hz10)
        .unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    loop {
        if sensor.accel_status().unwrap().xyz_new_data | sensor.mag_status().unwrap().xyz_new_data {
            let acc_data = sensor.accel_data().unwrap();
            let mag_data = sensor.mag_data().unwrap();
            rprintln!("Acceleration: x {} y {} z {}", acc_data.x, acc_data.y, acc_data.z);
            rprintln!("Magnetometer: x {} y {} z {}", mag_data.x, mag_data.y, mag_data.z);
            write!(
                serial,
                "Acceleration: x {} y {} z {} \r\n",
                acc_data.x, acc_data.y, acc_data.z
            );
            write!(
                serial,
                "Magnetometer: x {} y {} z {} \r\n",
                mag_data.x, mag_data.y, mag_data.z
            );
        }
        nb::block!(serial.flush()).unwrap();
    }
}
