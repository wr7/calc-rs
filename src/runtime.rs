//! Includes basic things needed to run rust code. This includes:
//!  - The interrupt vector for the MCU
//!  - The panic handler
//!  - The memory allocator (only because we're using allocation)

use core::panic::PanicInfo;

use calc_alloc::B64Allocator;
use stm_util::gpio::{GPIOPin, GPIOPort};

#[global_allocator]
static ALLOCATOR: B64Allocator<32> = B64Allocator::new();

extern "C" {
    fn WindowWatchdogInterrupt();
    fn RTCInterrupt();
    fn FlashInterrupt();
    fn RCCInterrupt();
    fn EXTI0_1Interrupt();
    fn EXTI2_3Interrupt();
    fn EXTI4_15Interrupt();
    fn DMACh1Interrupt();
    fn DMACh2_3Interrupt();
    fn DMACh4_5Interrupt();
    fn ADCInterrupt();
    fn Tim1BrkUpTrgCOMInterrupt();
    fn Tim1CCInterrupt();
    fn Tim3Interrupt();
    fn Tim6Interrupt();
    fn Tim14Interrupt();
    fn Tim15Interrupt();
    fn Tim16Interrupt();
    fn Tim17Interrupt();
    fn I2C1Interrupt();
    fn I2C2Interrupt();
    fn SPI1Interrupt();
    fn SPI2Interrupt();
    fn USART1Interrupt();
    fn USART2Interrupt();
    fn USART3_4_5_6Interrupt();
    fn USBInterrupt();
}

#[link_section = ".vector_table.interrupts"]
#[no_mangle]
pub static __INTERRUPTS: [Option<unsafe extern "C" fn()>; 32] = [
    Some(WindowWatchdogInterrupt),  // 0
    None,                           // RESERVED // 1
    Some(RTCInterrupt),             // 2
    Some(FlashInterrupt),           // 3
    Some(RCCInterrupt),             // 4
    Some(EXTI0_1Interrupt),         // 5
    Some(EXTI2_3Interrupt),         // 6
    Some(EXTI4_15Interrupt),        // 7
    None,                           // RESERVED // 8
    Some(DMACh1Interrupt),          // 9
    Some(DMACh2_3Interrupt),        // 10
    Some(DMACh4_5Interrupt),        // 11
    Some(ADCInterrupt),             // 12
    Some(Tim1BrkUpTrgCOMInterrupt), // 13
    Some(Tim1CCInterrupt),          // 14
    None,                           // RESERVED // 15
    Some(Tim3Interrupt),            // 16
    Some(Tim6Interrupt),            // 17
    None,                           // RESERVED // 18
    Some(Tim14Interrupt),           // 19
    Some(Tim15Interrupt),           // 20
    Some(Tim16Interrupt),           // 21
    Some(Tim17Interrupt),           // 22
    Some(I2C1Interrupt),            // 23
    Some(I2C2Interrupt),            // 24
    Some(SPI1Interrupt),            // 25
    Some(SPI2Interrupt),            // 26
    Some(USART1Interrupt),          // 27
    Some(USART2Interrupt),          // 28
    Some(USART3_4_5_6Interrupt),    // 29
    None,                           // RESERVED // 30
    Some(USBInterrupt),             // 31
];

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    const LED_PIN: GPIOPin = unsafe { GPIOPin::new_unchecked(GPIOPort::A, 15) };
    LED_PIN.set(false);

    loop {}
}
