#![no_std]
#![no_main]

// TODO: make a delay between i2c packets so that the MCU doesn't pull SCL low indefinitely

use core::{
    iter,
    panic::PanicInfo,
    ptr::{self, addr_of_mut},
};

use bitmap32::BitMap;
use calc::CalculatorState;
use calc_alloc::B64Allocator;
use calc_common::{Character, MetaButton};
use cortex_m_rt::entry;
use stm_util::{
    address,
    gpio::{GPIOConfiguration, GPIOPin, GPIOPort, PinSpeed, PinType, PullUpPullDownResistor},
    i2c::{self, I2C2},
};

#[global_allocator]
static ALLOCATOR: B64Allocator<32> = B64Allocator::new();

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

    let alt_init_packet = [
        0x00, // command stream mode
        0xa8, 0x3f, 0xd3, 0x00, // set display offset
        0x40, // set start line
        0xa0, // set segment remap
        0xc0, // set com output scan direction
        0xda, 0x12, // set com pin hardware configuration
        0x81, 0x7f, // set contrast
        0xa4, // read contents from ram
        0xd5, 0x80, // set oscillator frequency
        0x8d, 0x14, // enable charge pump
        0xaf, // turn on display
    ];

    err |= !i2c2.send_data(0x3c, alt_init_packet);

    let mut state = CalculatorState::default();
    err |= !update_screen(&mut i2c2, &state.screen);
    psuedo_wait();

    let buttons = [
        Character::Nine.into(),
        Character::Plus.into(),
        Character::Five.into(),
        Character::Multiply.into(),
        Character::OpenParen.into(),
        Character::Three.into(),
        Character::Plus.into(),
        Character::Two.into(),
        Character::CloseParen.into(),
        MetaButton::Enter.into(),
    ];

    for button in buttons {
        state.on_button_press(button);
        err |= !update_screen(&mut i2c2, &state.screen);
        // psuedo_wait();
    }

    // state.on_button_press(Character::Nine.into());
    // err |= !update_screen(&mut i2c2, &state.screen);
    // psuedo_wait();

    // state.on_button_press(Character::Plus.into());
    // err |= !update_screen(&mut i2c2, &state.screen);
    // psuedo_wait();

    // state.on_button_press(Character::Five.into());
    // err |= !update_screen(&mut i2c2, &state.screen);
    // psuedo_wait();

    // state.on_button_press(Character::Zero.into());
    // err |= !update_screen(&mut i2c2, &state.screen);
    // psuedo_wait();

    // state.on_button_press(calc::Button::Enter);
    // err |= !update_screen(&mut i2c2, &state.screen);
    // psuedo_wait();

    // err |= !update_screen(&mut i2c2, &stest::test_map);

    if !err {
        LED_PIN.set(true);
    } else {
        // LED_PIN.set(true);
    }

    loop {}
}

pub fn update_screen(i2c2: &mut I2C2, buffer: &BitMap<u32, 256>) -> bool {
    let mut err = false;

    for page in 0..8 {
        err |= !i2c2.send_data(0x3c, [0x00, 0xb0 | page as u8, 0x00, 0x10]);
        if let Some(mut frame) = i2c2.start_frame(0x3c, 0x40) {
            err |= !frame.transmit(0);
            err |= !frame.transmit(0);

            for column in 0..128 {
                let u32_index = column * 2 + page / 4;
                let byte_index = page % 4;

                frame.transmit(buffer.0[u32_index].to_be_bytes()[byte_index].reverse_bits());
            }

            frame.stop();
        } else {
            err = true;
        }
    }

    !err
}
