#![no_std]
#![no_main]

// TODO: make a delay between i2c packets so that the MCU doesn't pull SCL low indefinitely

use core::{
    panic::PanicInfo,
    ptr::{self, addr_of_mut},
};

use bitmap32::BitMap;
use calc::CalculatorState;
use calc_alloc::B64Allocator;
use calc_common::Character;
use calc_keyboard::ButtonEvent;
use cortex_m_rt::entry;
use stm_util::{
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

#[entry]
fn main() -> ! {
    const LED_PIN: GPIOPin = GPIOPin::new(GPIOPort::A, 15);

    unsafe {
        ptr::write_volatile(addr_of_mut!(COUNTER), 0);
    }

    GPIOPort::A.enable();

    LED_PIN.configure(GPIOConfiguration::output(
        PinType::PushPull,
        PinSpeed::Low,
        PullUpPullDownResistor::None,
    ));

    for keyboard_pin in 0..10 {
        let pin = GPIOPin::new(GPIOPort::A, keyboard_pin);
        pin.configure(GPIOConfiguration::input(PullUpPullDownResistor::PullDown));
    }

    LED_PIN.set(false);

    let mut i2c2 = unsafe { i2c::initialize_i2c2() };

    psuedo_wait(); // The screen needs time to turn on

    let mut err = false;

    let init_packet = [
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

    err |= !i2c2.send_data(0x3c, init_packet);

    let mut state = CalculatorState::default();
    err |= !update_screen(&mut i2c2, &state.screen);
    psuedo_wait();

    if !err {
        LED_PIN.set(true);
    }

    let mut old_state = (0, 0);
    loop {
        let new_state = get_row_column_state();

        let keyboard_event = ButtonEvent::from_keyboard_state(old_state, new_state);
        state.msg.clear();
        for key_state in [old_state, new_state] {
            for rc in [key_state.0, key_state.1] {
                for bit_index in 0..5 {
                    let bit = rc & (1 << bit_index) != 0;
                    if bit {
                        state.msg.push(Character::One);
                    } else {
                        state.msg.push(Character::Zero);
                    }
                }
                state.msg.push(Character::Slash);
            }
            state.msg.pop();
            state.msg.push(Character::Dot);
        }
        state.msg.pop();
        old_state = new_state;

        if let Some(event) = keyboard_event {
            if event.button_down {
                state.on_button_press(event.button);
                update_screen(&mut i2c2, &state.screen);
            }
        }
    }
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

pub fn get_row_column_state() -> (u8, u8) {
    let port_a_state = GPIOPort::A.get_all_pins();

    let column_state = port_a_state & ((1 << 5) - 1);
    let row_state = (port_a_state >> 5) & ((1 << 5) - 1);

    (row_state as u8, column_state as u8)
}
