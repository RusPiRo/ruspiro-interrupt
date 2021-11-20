//! # Interrupt Handler Usage Example
//!
//! This minimal example uses the ARM system timer to trigger an interrupt.
//! The implementation demonstrates how the corresponding interrupt handler
//! could be written and how the ISR channel can be used to notify the 
//! normal processing that the interrupt has happened.
//!
//! When this example is build and deployed to the Raspberry Pi it will blink an
//! LED connected to GPIO pin 21.
//!

#![no_std]
#![no_main]

extern crate alloc;
extern crate ruspiro_allocator;
extern crate ruspiro_boot;
extern crate ruspiro_interrupt;

use alloc::boxed::Box;
use ruspiro_boot::{come_alive_with, run_with};
use ruspiro_interrupt::{self as irq, IrqHandler, isr_channel};
use ruspiro_mmio_register::define_mmio_register;
use ruspiro_mmu as mmu;

come_alive_with!(alive);
run_with!(running);

fn alive(core: u32) {
    // do one-time initialization here
    // configure the mmu as we will deal with atomic operations (within the memory
    // allocator that is used by the isr channel under the hood to store the data
    // within the HEAP)
    // use some arbitrary values for VideoCore memory start and size. This is fine
    // as we will use a small lower amount of the ARM memory only.
    unsafe { mmu::initialize(core, 0x3000_0000, 0x001_000) };

    // initialize interrupt handling
    irq::initialize();

    // configure the timer
    SYS_TIMERCS::Register.write_value(SYS_TIMERCS::M1::MATCH);
    // set the match value to the current free-running counter + some delta
    // the delta is one tick per micro second
    let current = SYS_TIMERCLO::Register.get();
    // set the match value to 1s after now
    SYS_TIMERC1::Register.set(current + 1_000_000);

    // globally enable interrupts
    irq::enable_interrupts();

    // now create the ISR channel and register the same with the interrupt
    let (timer_tx, timer_rx) = isr_channel();
    irq::activate(irq::Interrupt::SystemTimer1, Some(timer_tx.clone()));

    loop {
      // wait for the interrupt to send stuff through the channel and lit a led
      // on GPIO 21 to indicate this
      while timer_rx.recv().is_err() {};
      unsafe { lit_debug_led(21) };

      // wait for the interrupt to send stuff through the channel and clear a led
      // on GPIO 21 to indicate this
      while timer_rx.recv().is_err() {};
      unsafe { clear_debug_led(21) };
    }
}

fn running(_core: u32) -> ! {
    // do any processing here and ensure you never return from this function
    loop { }
}


// provide the interrupt handler implementation for a specific interrupt
#[IrqHandler(SystemTimer1)]
fn isr_system_timer(channel: Option<IsrSender<Box<dyn Any>>>) {
  // as soon as the interupt was raised we need to acknowledge the same
  // as we could configure up to 4 compare match values we need to check
  // which one actually raised this IRQ. We only deal with match value 1 here
  if SYS_TIMERCS::Register.read(SYS_TIMERCS::M1) == 1 {
    SYS_TIMERCS::Register.write_value(SYS_TIMERCS::M1::MATCH);
    // in case of a channel being present just send an empty message
    channel.map(|tx| tx.send(Box::new(())));
    // once we have received the timer interrupt update the match value
    // to trigger the interrupt again
    // set the match value to the current free-running counter + some delta
    // the delta is one tick per micro second
    let current = SYS_TIMERCLO::Register.get();
    // set the match value to 1s after now
    SYS_TIMERC1::Register.set(current + 1_000_000);
  }
}

// Define some MMIO registers required for accessing the timer device
const PERIPHERAL_BASE: usize = 0x3F00_0000;
// Base address of system timer MMIO register
const SYS_TIMER_BASE: usize = PERIPHERAL_BASE + 0x3000;

define_mmio_register![
    /// system timer control register, keep in mind that actually only timer 1 and 3 are free on RPi
    pub SYS_TIMERCS<ReadWrite<u32>@(SYS_TIMER_BASE)> {
        /// system timer 0 match flag
        M0 OFFSET(0) [
            MATCH = 1,
            CLEAR = 0
        ],
        /// system timer 1 match flag
        M1 OFFSET(1) [
            MATCH = 1,
            CLEAR = 0
        ],
        /// system timer 2 match flag
        M2 OFFSET(2) [
            MATCH = 1,
            CLEAR = 0
        ],
        /// system timer 3 match flag
        M3 OFFSET(3) [
            MATCH = 1,
            CLEAR = 0
        ]
    },
    /// system timer free running counter lower 32Bit value
    pub SYS_TIMERCLO<ReadOnly<u32>@(SYS_TIMER_BASE + 0x04)>,
    /// system timer compare value register
    pub SYS_TIMERC1<ReadWrite<u32>@(SYS_TIMER_BASE + 0x10)>
];

// This minimal example should not have a big dependency tree just to provide a simplified version of how the interrupt
// crate is intended to be used. But to be able to see any feedback once this example is deployed to the Raspberry Pi
// we use an unsafe and direct way to manipulate a GPIO pin to lit a LED to visualize the interrupt has been raised
use core::ptr::{read_volatile, write_volatile};
/// Lit a LED connected to the given GPIO number
///
/// # Safety
/// This access is unsafe as it directly writes to MMIO registers.
unsafe fn lit_debug_led(num: u32) {
    let fsel_num = num / 10;
    let fsel_shift = (num % 10) * 3;
    let fsel_addr = PERIPHERAL_BASE as u32 + 0x0020_0000 + 4 * fsel_num;
    let set_addr = PERIPHERAL_BASE as u32 + 0x0020_001c + num / 32;
    let mut fsel: u32 = read_volatile(fsel_addr as *const u32);
    fsel &= !(7 << fsel_shift);
    fsel |= 1 << fsel_shift;
    write_volatile(fsel_addr as *mut u32, fsel);

    let set: u32 = 1 << (num & 0x1F);
    write_volatile(set_addr as *mut u32, set);
}

/// Clear a LED connected to the given GPIO number
///
/// # Safety
/// This access is unsafe as it directly writes to MMIO registers.
unsafe fn clear_debug_led(num: u32) {
  let fsel_num = num / 10;
  let fsel_shift = (num % 10) * 3;
  let fsel_addr = PERIPHERAL_BASE as u32 + 0x0020_0000 + 4 * fsel_num;
  let clr_addr = PERIPHERAL_BASE as u32 + 0x0020_0028 + num / 32;
  let mut fsel: u32 = read_volatile(fsel_addr as *const u32);
  fsel &= !(7 << fsel_shift);
  fsel |= 1 << fsel_shift;
  write_volatile(fsel_addr as *mut u32, fsel);

  let clear: u32 = 1 << (num & 0x1F);
  write_volatile(clr_addr as *mut u32, clear);
}