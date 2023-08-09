#![no_std]
#![no_main]

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
use embedded_hal::digital::v2::OutputPin;
use heapless::String;
use panic_halt as _;
use rp_pico as bsp;
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

const ANALOG_ROUND: u16 = 5;

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

    loop {
        usb_dev.poll(&mut [&mut serial]);

        let value: u16 = adc.read(&mut speed_pin).unwrap();
        let value = (100u16).saturating_sub(value / 40);
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

        delay.delay_ms(5);
    }
}
