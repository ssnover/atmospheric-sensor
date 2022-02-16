#![no_main]
#![no_std]

use core::convert::TryInto;
use hal::prelude::*;
use scd30::scd30::Scd30;
use stm32f3xx_hal as hal;

use infrastructure as _;
use serial_protocol::ReportCO2Data;

#[cortex_m_rt::entry]
fn main() -> ! {
    let board = hal::pac::Peripherals::take().unwrap();
    let mut rcc = board.RCC.constrain();
    let mut gpiob = board.GPIOB.split(&mut rcc.ahb);
    let mut ld2 = gpiob
        .pb13
        .into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    ld2.set_low().unwrap();

    let mut flash = board.FLASH.constrain();
    let clocks = rcc.cfgr.sysclk(8.MHz()).freeze(&mut flash.acr);
    let mut delay = hal::delay::Delay::new(cortex_m::Peripherals::take().unwrap().SYST, clocks);

    let mut i2c_scl =
        gpiob
            .pb8
            .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrh);
    let mut i2c_sda =
        gpiob
            .pb9
            .into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrh);
    i2c_scl.internal_pull_up(&mut gpiob.pupdr, true);
    i2c_sda.internal_pull_up(&mut gpiob.pupdr, true);
    let i2c = hal::i2c::I2c::new(
        board.I2C1,
        (i2c_scl, i2c_sda),
        100.kHz().try_into().unwrap(),
        clocks,
        &mut rcc.apb1,
    );

    let mut gpioa = board.GPIOA.split(&mut rcc.ahb);
    let serial_tx =
        gpioa
            .pa2
            .into_af7_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    let mut serial_rx =
        gpioa
            .pa3
            .into_af7_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl);
    serial_rx.internal_pull_up(&mut gpioa.pupdr, true);
    let mut serial = hal::serial::Serial::new(
        board.USART2,
        (serial_tx, serial_rx),
        115200.Bd(),
        clocks,
        &mut rcc.apb1,
    );

    let mut sensor = Scd30::new_with_address(i2c, 0x61);
    sensor.start_measuring_with_mbar(30).unwrap();

    let mut body_buffer = [0u8; 32];

    loop {
        for _ in 0..10 {
            ld2.toggle().unwrap();
            delay.delay_ms(1000_u16);
        }

        if let Ok(Some(measurement)) = sensor.read() {
            let msg = ReportCO2Data {
                measurement: measurement.co2,
            };
            let msg = serial_protocol::Message {
                hdr: serial_protocol::Header {
                    version: 0x00,
                    id: 0x00,
                    msg_type: serial_protocol::MessageType::ReportCO2Data,
                },
                msg: &serial_protocol::encode_body(&msg, &mut body_buffer).unwrap(),
            };
            let mut buf = [0u8; 32];
            let bytes_used = serial_protocol::encode(&msg, &mut buf).unwrap();
            serial.bwrite_all(&bytes_used).unwrap();
        } else {
            serial.write(b'0').unwrap();
        }
    }
}
