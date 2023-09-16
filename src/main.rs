#![no_std]
#![no_main]

extern crate alloc;

// TODO: make a delay between i2c packets so that the MCU doesn't pull SCL low indefinitely

use core::ptr::{self, addr_of_mut};

use alloc::vec::Vec;
use bitmap32::BitMap;
use calc::CalculatorState;

use calc_keyboard::ButtonEvent;
use cortex_m_rt::entry;
use stm_util::{
    enable_interrupt, enable_interrupts,
    gpio::{GPIOConfiguration, GPIOPin, GPIOPort, PinSpeed, PinType, PullUpPullDownResistor},
    i2c::{self, I2C2},
    timer::Timer6,
    CriticalSection,
};

mod runtime;

#[no_mangle]
#[link_section = ".bss"]
static mut COUNTER: u32 = 0;

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

static mut PENDING_BUTTONS: Vec<ButtonEvent> = Vec::new();

#[no_mangle]
unsafe extern "C" fn Tim6Interrupt() {
    Timer6::clear_interrupt();

    static mut OLD_KEYBOARD_STATE: (u8, u8) = (0, 0);

    let new_state = get_row_column_state();

    let event = ButtonEvent::from_keyboard_state(OLD_KEYBOARD_STATE, new_state);
    OLD_KEYBOARD_STATE = new_state;

    if let Some(event) = event {
        PENDING_BUTTONS.insert(0, event);
    }
}

fn pop_button_event() -> Option<ButtonEvent> {
    let critical_section = CriticalSection::start();

    // SAFE: interrupts are disabled, so Tim6Interrupt cannot have this reference at the same time as this function
    let return_value = unsafe { &mut PENDING_BUTTONS }.pop();

    critical_section.end();

    return_value
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

    if !err {
        LED_PIN.set(true);
    }

    stm_util::timer::Timer6::initialize();

    unsafe { enable_interrupt(17) }; // Enables the TIM6 interrupt

    loop {
        stm_util::disable_interrupts();

        if let Some(event) = pop_button_event() {
            enable_interrupts();

            if event.button_down {
                state.on_button_press(event.button);
                update_screen(&mut i2c2, &state.screen);
            }
        } else {
            stm_util::wait_for_interrupt();
            enable_interrupts();
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
