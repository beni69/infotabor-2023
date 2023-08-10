#![no_std]
#![no_main]

use rp_pico as bsp;

use bsp::{
    entry,
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        pac,
        sio::Sio,
        usb::UsbBus,
        watchdog::Watchdog,
        Adc, Timer,
    },
    Pins,
};
use core::convert::Infallible;
use core::fmt::Write;
use cortex_m::{delay::Delay, prelude::_embedded_hal_adc_OneShot};
use defmt_rtt as _;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use heapless::String;
use panic_halt as _;
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

const ANALOG_ROUND: u16 = 5;
const SERVO_MIN: u32 = 2750;
const SERVO_MAX: u32 = 4100;
const SERVO_ROUND: u16 = 10;

#[entry]
fn entry() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let adc = Adc::new(pac.ADC, &mut pac.RESETS);

    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    let serial = SerialPort::new(&usb_bus);
    let usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Karesz Klub")
        .product("TvRemote Pico Relay")
        .serial_number("C.U.M-2")
        .device_class(2)
        .build();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);

    while timer.get_counter().ticks() < 2_000_00 {}

    main(pins, delay, adc, serial, usb_dev, timer).unwrap();
    loop {}
}

fn main(
    pins: Pins,
    mut delay: Delay,
    mut adc: Adc,
    mut serial: SerialPort<UsbBus>,
    mut usb_dev: UsbDevice<UsbBus>,
    timer: Timer,
) -> Result<(), Infallible> {
    let mut led = pins.led.into_push_pull_output();
    led.set_high()?;

    let mut led0 = pins.gpio4.into_push_pull_output();
    let mut led1 = pins.gpio3.into_push_pull_output();
    let mut led2 = pins.gpio2.into_push_pull_output();
    let mut led3 = pins.gpio1.into_push_pull_output();
    let mut led4 = pins.gpio0.into_push_pull_output();

    let mut speed_pin = pins.gpio26.into_floating_input();
    let mut servo_pin = pins.gpio27.into_floating_input();

    let btn_w = pins.gpio14.into_pull_up_input();
    let btn_a = pins.gpio16.into_pull_up_input();
    let btn_s = pins.gpio17.into_pull_up_input();
    let btn_d = pins.gpio15.into_pull_up_input();
    let btn_buzzer = pins.gpio6.into_pull_up_input();

    led0.set_high()?;
    delay.delay_ms(200);
    led1.set_high()?;
    delay.delay_ms(200);
    led2.set_high()?;
    delay.delay_ms(200);
    led3.set_high()?;
    delay.delay_ms(200);
    led4.set_high()?;
    delay.delay_ms(800);

    led0.set_low()?;
    led1.set_low()?;
    led2.set_low()?;
    led3.set_low()?;
    led4.set_low()?;

    while timer.get_counter().ticks() < 2_000_000 {
        usb_dev.poll(&mut [&mut serial]);
    }
    let _ = serial.write(b"serial these nuts.\r\n");

    let mut text: String<64> = String::new();
    let mut speed: u16 = 0;
    let mut servo: u16 = 0;
    let mut btn = Buttons::default();

    loop {
        usb_dev.poll(&mut [&mut serial]);

        let value: u16 = adc.read(&mut speed_pin).unwrap();
        let value = (100u16).saturating_sub(value / (4000 / 100)); // 0..4000 -> 0.100
        let _speed = value - (value % ANALOG_ROUND);
        if _speed != speed {
            speed = _speed;

            led0.set_high()?;
            if speed >= 20 {
                led1.set_high()?;
            } else {
                led1.set_low()?;
            }
            if speed >= 40 {
                led2.set_high()?;
            } else {
                led2.set_low()?;
            }
            if speed >= 60 {
                led3.set_high()?;
            } else {
                led3.set_low()?;
            }
            if speed >= 80 {
                led4.set_high()?;
            } else {
                led4.set_low()?;
            }

            text.clear();
            writeln!(&mut text, "speed {speed}").unwrap();
            let _ = serial.write(text.as_bytes());
        }

        let value: u16 = adc.read(&mut servo_pin).unwrap();
        let _servo = linear_conv(value as u32, SERVO_MIN, SERVO_MAX, 0, 95) as u16;
        let _servo = _servo - (_servo % SERVO_ROUND);
        if _servo != servo {
            servo = _servo;
            text.clear();
            writeln!(&mut text, "servo {servo}").unwrap();
            let _ = serial.write(text.as_bytes());
        }

        let w = btn_w.is_low()?;
        if w != btn.w {
            btn.w = w;
            text.clear();
            writeln!(&mut text, "w {}", w as u8).unwrap();
            let _ = serial.write(text.as_bytes());
        }
        let a = btn_a.is_low()?;
        if a != btn.a {
            btn.a = a;
            text.clear();
            writeln!(&mut text, "a {}", a as u8).unwrap();
            let _ = serial.write(text.as_bytes());
        }
        let s = btn_s.is_low()?;
        if s != btn.s {
            btn.s = s;
            text.clear();
            writeln!(&mut text, "s {}", s as u8).unwrap();
            let _ = serial.write(text.as_bytes());
        }
        let d = btn_d.is_low()?;
        if d != btn.d {
            btn.d = d;
            text.clear();
            writeln!(&mut text, "d {}", d as u8).unwrap();
            let _ = serial.write(text.as_bytes());
        }
        let buzzer = btn_buzzer.is_low()?;
        if buzzer != btn.buzzer {
            btn.buzzer = buzzer;
            text.clear();
            writeln!(&mut text, "buzzer {}", buzzer as u8).unwrap();
            let _ = serial.write(text.as_bytes());
        }

        delay.delay_ms(5);
    }
}

fn linear_conv(n: u32, xmin: u32, xmax: u32, ymin: u32, ymax: u32) -> u32 {
    let xrange = xmax - xmin;
    let yrange = ymax - ymin;
    n.saturating_sub(xmin)
        .saturating_mul(yrange)
        .saturating_div(xrange)
        .saturating_add(ymin)
}

#[derive(Default)]
struct Buttons {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
    buzzer: bool,
}
