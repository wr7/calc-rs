#![no_std]
#![no_main]

#[no_mangle]
static mut FOO: u32 = 0;

mod bare;

fn main() -> ! {
    loop {
        unsafe {
            FOO += 1;
        }
    }
}
