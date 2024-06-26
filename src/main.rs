#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::IO,
    interrupt::Priority,
    pcnt::{channel::{self, PcntSource}, unit, PCNT},
    peripherals::Peripherals,
    prelude::*,
};
use log::info;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    esp_println::logger::init_logger_from_env();
    log::info!("Hello world!");

    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio2.into_push_pull_output();
    let mut clk = io.pins.gpio12.into_pull_up_input();
    let mut dt = io.pins.gpio13.into_pull_up_input();

    let pcnt = PCNT::new(peripherals.PCNT, Some(interrupt_handler));
    let mut u0 = pcnt.get_unit(unit::Number::Unit1);
    u0.configure(unit::Config {
        low_limit: -100,
        high_limit: 100,
        filter: Some(min(10u16 * 80, 1023u16)),
        ..Default::default()
    })
    .unwrap();

    info!("setup channel 0");
    let mut ch0 = u0.get_channel(channel::Number::Channel0);

    ch0.configure(
        PcntSource::from_pin(&mut clk),
        PcntSource::from_pin(&mut dt),
        channel::Config {
            lctrl_mode: channel::CtrlMode::Reverse,
            hctrl_mode: channel::CtrlMode::Keep,
            pos_edge: channel::EdgeMode::Decrement,
            neg_edge: channel::EdgeMode::Increment,
            invert_ctrl: false,
            invert_sig: false,
        },
    );

    info!("setup channel 1");
    let mut ch1 = u0.get_channel(channel::Number::Channel1);
    ch1.configure(
        PcntSource::from_pin(&mut dt),
        PcntSource::from_pin(&mut clk),
        channel::Config {
            lctrl_mode: channel::CtrlMode::Reverse,
            hctrl_mode: channel::CtrlMode::Keep,
            pos_edge: channel::EdgeMode::Increment,
            neg_edge: channel::EdgeMode::Decrement,
            invert_ctrl: false,
            invert_sig: false,
        },
    );
    info!("subscribing to events");
    u0.events(unit::Events {
        low_limit: true,
        high_limit: true,
        thresh0: false,
        thresh1: false,
        zero: false,
    });

    info!("enabling interrupts");
    u0.listen();
    info!("resume pulse counter unit 0");
    u0.resume();

    critical_section::with(|cs| UNIT0.borrow_ref_mut(cs).replace(u0));

    let mut last_value: i32 = 0;
    loop {
        critical_section::with(|cs| {
            let mut u0 = UNIT0.borrow_ref_mut(cs);
            let u0 = u0.as_mut().unwrap();
            let value: i32 = u0.get_value() as i32 + VALUE.load(Ordering::SeqCst);
            if value != last_value {
                info!("value: {value}");
                last_value = value;
            }
            if value % 8 > 3 {
                led.set_high();
            } else {
                led.set_low();
            }
        });
    }
}

//% CHIPS: esp32 esp32c6 esp32h2 esp32s2 esp32s3

// #![no_std]
// #![no_main]

use core::{cell::RefCell, cmp::min, sync::atomic::Ordering};

use critical_section::Mutex;
// use esp_backtrace as _;
// use esp_hal::{
//     gpio::{Io, Pull},
//     interrupt::Priority,
//     pcnt::{
//         channel::{self, PcntInputConfig, PcntSource},
//         unit,
//         Pcnt,
//     },
//     peripherals::Peripherals,
//     prelude::*,
// };
// use esp_println::println;
use portable_atomic::AtomicI32;

static UNIT0: Mutex<RefCell<Option<unit::Unit>>> = Mutex::new(RefCell::new(None));
static VALUE: AtomicI32 = AtomicI32::new(0);

// #[entry]
// fn main() -> ! {
//     let peripherals = Peripherals::take();

//     let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

//     // Set up a pulse counter:
//     println!("setup pulse counter unit 0");
//     let pcnt = Pcnt::new(peripherals.PCNT, Some(interrupt_handler));
//     let mut u0 = pcnt.get_unit(unit::Number::Unit1);
//     u0.configure(unit::Config {
//         low_limit: -100,
//         high_limit: 100,
//         filter: Some(min(10u16 * 80, 1023u16)),
//         ..Default::default()
//     })
//     .unwrap();

//     println!("setup channel 0");
//     let mut ch0 = u0.get_channel(channel::Number::Channel0);
//     let mut pin_a = io.pins.gpio4;
//     let mut pin_b = io.pins.gpio5;

//     ch0.configure(
//         PcntSource::from_pin(&mut pin_a, PcntInputConfig { pull: Pull::Up }),
//         PcntSource::from_pin(&mut pin_b, PcntInputConfig { pull: Pull::Up }),
//         channel::Config {
//             lctrl_mode: channel::CtrlMode::Reverse,
//             hctrl_mode: channel::CtrlMode::Keep,
//             pos_edge: channel::EdgeMode::Decrement,
//             neg_edge: channel::EdgeMode::Increment,
//             invert_ctrl: false,
//             invert_sig: false,
//         },
//     );

//     println!("setup channel 1");
//     let mut ch1 = u0.get_channel(channel::Number::Channel1);
//     ch1.configure(
//         PcntSource::from_pin(&mut pin_b, PcntInputConfig { pull: Pull::Up }),
//         PcntSource::from_pin(&mut pin_a, PcntInputConfig { pull: Pull::Up }),
//         channel::Config {
//             lctrl_mode: channel::CtrlMode::Reverse,
//             hctrl_mode: channel::CtrlMode::Keep,
//             pos_edge: channel::EdgeMode::Increment,
//             neg_edge: channel::EdgeMode::Decrement,
//             invert_ctrl: false,
//             invert_sig: false,
//         },
//     );
//     println!("subscribing to events");
//     u0.events(unit::Events {
//         low_limit: true,
//         high_limit: true,
//         thresh0: false,
//         thresh1: false,
//         zero: false,
//     });

//     println!("enabling interrupts");
//     u0.listen();
//     println!("resume pulse counter unit 0");
//     u0.resume();

//     critical_section::with(|cs| UNIT0.borrow_ref_mut(cs).replace(u0));

//     let mut last_value: i32 = 0;
//     loop {
//         critical_section::with(|cs| {
//             let mut u0 = UNIT0.borrow_ref_mut(cs);
//             let u0 = u0.as_mut().unwrap();
//             let value: i32 = u0.get_value() as i32 + VALUE.load(Ordering::SeqCst);
//             if value != last_value {
//                 println!("value: {value}");
//                 last_value = value;
//             }
//         });
//     }
// }

#[handler(priority = Priority::Priority2)]
fn interrupt_handler() {
    critical_section::with(|cs| {
        let mut u0 = UNIT0.borrow_ref_mut(cs);
        let u0 = u0.as_mut().unwrap();
        if u0.interrupt_set() {
            let events = u0.get_events();
            if events.high_limit {
                VALUE.fetch_add(100, Ordering::SeqCst);
            } else if events.low_limit {
                VALUE.fetch_add(-100, Ordering::SeqCst);
            }
            u0.reset_interrupt();
        }
    });
}
