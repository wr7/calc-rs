#![no_std]
#![no_main]

// TODO: make a delay between i2c packets so that the MCU doesn't pull SCL low indefinitely

use core::{
    iter,
    panic::PanicInfo,
    ptr::{self, addr_of_mut},
};

use cortex_m_rt::entry;
use stm_util::{
    address,
    gpio::{GPIOConfiguration, GPIOPin, GPIOPort, PinSpeed, PinType, PullUpPullDownResistor},
    i2c,
};

#[no_mangle]
#[link_section = ".bss"]
static mut COUNTER: u32 = 0;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    const LED_PIN: GPIOPin = unsafe { GPIOPin::new_unchecked(GPIOPort::A, 7) };
    LED_PIN.set(true);
    loop {}
}

fn psuedo_wait() {
    unsafe {
        let counter_ptr = addr_of_mut!(COUNTER);
        let mut counter_val = 0;

        while counter_val < 1_000_000 {
            counter_ptr.write_volatile(counter_val);
            counter_val += 1;
        }
    }
}

// #[allow(unused)]
// mod consts {
//     pub const SETCONTRAST: u8 = 0x81;
//     pub const DISPLAYALLON_RESUME: u8 = 0xA4;
//     pub const DISPLAYALLON: u8 = 0xA5;
//     pub const NORMALDISPLAY: u8 = 0xA6;
//     pub const INVERTDISPLAY: u8 = 0xA7;
//     pub const DISPLAYOFF: u8 = 0xAE;
//     pub const DISPLAYON: u8 = 0xAF;
//     pub const SETDISPLAYOFFSET: u8 = 0xD3;
//     pub const SETCOMPINS: u8 = 0xDA;
//     pub const SETVCOMDETECT: u8 = 0xDB;
//     pub const SETDISPLAYCLOCKDIV: u8 = 0xD5;
//     pub const SETPRECHARGE: u8 = 0xD9;
//     pub const SETMULTIPLEX: u8 = 0xA8;
//     pub const SETLOWCOLUMN: u8 = 0x00;
//     pub const SETHIGHCOLUMN: u8 = 0x10;
//     pub const SETSTARTLINE: u8 = 0x40;
//     pub const MEMORYMODE: u8 = 0x20;
//     pub const COLUMNADDR: u8 = 0x21;
//     pub const PAGEADDR: u8 = 0x22;
//     pub const COMSCANINC: u8 = 0xC0;
//     pub const COMSCANDEC: u8 = 0xC8;
//     pub const SEGREMAP: u8 = 0xA0;
//     pub const CHARGEPUMP: u8 = 0x8D;
//     pub const SWITCHCAPVCC: u8 = 0x2;
//     // Scrolling #defines
//     pub const ACTIVATE_SCROLL: u8 = 0x2F;
//     pub const DEACTIVATE_SCROLL: u8 = 0x2E;
//     pub const SET_VERTICAL_SCROLL_AREA: u8 = 0xA3;
//     pub const RIGHT_HORIZONTAL_SCROLL: u8 = 0x26;
//     pub const LEFT_HORIZONTAL_SCROLL: u8 = 0x27;
//     pub const VERTICAL_AND_RIGHT_HORIZONTAL_SCROLL: u8 = 0x29;
//     pub const VERTICAL_AND_LEFT_HORIZONTAL_SCROLL: u8 = 0x2A;
// }

#[entry]
fn main() -> ! {
    const LED_PIN: GPIOPin = unsafe { GPIOPin::new_unchecked(GPIOPort::A, 7) };

    unsafe {
        ptr::write_volatile(addr_of_mut!(COUNTER), 0);
    }

    GPIOPort::A.enable();

    LED_PIN.configure(GPIOConfiguration::output(
        PinType::PushPull,
        PinSpeed::Low,
        PullUpPullDownResistor::None,
    ));

    LED_PIN.set(false);

    let mut i2c2 = unsafe { i2c::initialize_i2c2() };

    // i2c::initialize_i2c2();

    psuedo_wait(); // The screen needs time to turn on

    let mut err = false;

    const I2C2_ISR: *mut u32 = (address::I2C2 + 0x18) as _;

    let test_packet = [0x00, 0xae, 0xa5, 0x8d, 0x14, 0xaf];

    err |= !i2c2.send_data(0x3c, test_packet);

    if !err {
        LED_PIN.set(true);
    } else {
        // LED_PIN.set(true);
    }

    loop {}
}
