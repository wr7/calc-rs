#![no_std]
#![no_main]

use core::{
    iter,
    panic::PanicInfo,
    ptr::{self, addr_of_mut},
};

use cortex_m_rt::entry;
use stm_util::{
    gpio::{GPIOConfiguration, GPIOPin, GPIOPort, PinSpeed, PinType, PullUpPullDownResistor},
    i2c,
};

#[no_mangle]
#[link_section = ".bss"]
#[export_name = "counter"]
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

    psuedo_wait(); // The little dinky screen needs time for it to power on

    i2c::initialize_i2c2();
    // i2c::i2c_send(LED_PIN, 0b0111100, [0b11000000, 0b11111111]);

    let mut err = false;

    // err |= !i2c::i2c_send(0b0111100, [0, 0xe3, 0xae, 0xa8, 31, 0xa5, 0xaf, 0xe3]);
    // err |= !i2c::i2c_send(
    //     0b0111100,
    //     [
    //         0,
    //         consts::DISPLAYOFF,
    //         consts::SETDISPLAYCLOCKDIV,
    //         0x80,
    //         consts::SETMULTIPLEX,
    //         31,
    //         consts::SETDISPLAYOFFSET,
    //         consts::SETSTARTLINE,
    //         consts::CHARGEPUMP,
    //         0x14,
    //         consts::SETVCOMDETECT,
    //         0x40,
    //         consts::DISPLAYALLON,
    //         consts::NORMALDISPLAY,
    //         consts::DEACTIVATE_SCROLL,
    //         consts::DISPLAYON,
    //     ],
    // );

    // err |= !i2c::i2c_send(
    //     0b0111100,
    //     [
    //         0,    // command stream mode
    //         0xae, // reset display
    //         0xa8, 31,   // set screen height to 31px
    //         0xa5, // entire display on
    //         0x8d, 0x14, // enable charge pump`
    //         0xaf, // turn on display
    //     ],
    // );

    // let commands = [
    //     0xae, // reset display
    //     0xa8, 31, // set mux
    //     0xd3, 0,    // set display offset
    //     0x40, // set start line
    //     0xa0, // set segment remap
    //     0xc0, // set COM scan direction
    //     0xda, 0x02, // set COM hardware configuration
    //     0x81, 0x7f, // set contrast
    //     0xa5, // set screen to on
    //     0xd5, 0x80, // set oscillator frequency
    //     0x8d, 0x14, // enable charge pump
    //     0xaf, // turn on screen
    // ];

    // for command in commands {
    //     err |= !i2c::i2c_send(0b0111100, [0, command]);
    // }

    // err |= !i2c::i2c_send(
    //     0b0111100,
    //     iter::once(0x40).chain(.take(129),
    // );

    // err |= !i2c::i2c_send(
    //     0b0111100,
    //     [
    //         0,    // command stream mode
    //         0xae, // reset display
    //         0xa8, 31, // set mux
    //         0xd3, 0,    // set display offset
    //         0x40, // set start line
    //         0xa0, // set segment remap
    //         0xc0, // set COM scan direction
    //         0xda, 0x02, // set COM hardware configuration
    //         0x81, 0x7f, // set contrast
    //         0xa5, // set screen to on
    //         0xd5, 0x80, // set oscillator frequency
    //         0x8d, 0x14, // enable charge pump
    //         0xaf, // turn on screen
    //     ],
    // );

    err |= !i2c::i2c_send(0b0111100, [0, 0xae, 0xe3]);
    err |= !i2c::i2c_send(0b0111100, [0, 0xe3]);

    // err |= !i2c::i2c_send(0b0111100, [0b01000000, 0xFF, 0xFF, 0xFF, 0xFF]);
    // err |= !i2c::i2c_send(0b0111100, [0b10000000, 0b10100100]);

    if !err {
        LED_PIN.set(true);
    }

    // LED_PIN.set(true);

    // i2c::i2c_send(LED_PIN, 0b0111100, [0b10000000, 0b10100100]);

    // i2c::i2c_send(LED_PIN, 0b0101101, [0b10000000, 0b10100101]);

    // LED_PIN.set(true);

    // let mut counter_val = 0;
    // let mut state = true;

    // loop {
    //     unsafe {
    //         if counter_val == 0 {
    //             LED_PIN.set(state);
    //             state = !state;
    //         }

    //         let counter_addr = addr_of_mut!(COUNTER);
    //         counter_val += 1;

    //         ptr::write_volatile(counter_addr, counter_val);

    //         if counter_val == 1_000_000 {
    //             counter_val = 0;
    //         }
    //     }
    // }
    loop {}
}
