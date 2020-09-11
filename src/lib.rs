/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-interrupt/0.3.1")]
#![no_std]
#![feature(llvm_asm)]
#![feature(linkage)]

//! # Interrupt handler for Raspberry Pi
//!
//! This crates provides functions and macros (custom attribute) to conviniently implement interrupt handler for
//! Raspberry Pi 3. The possible interrupts a handler can be implemented for are available as enum [irqtypes::Interrupt]
//!
//! # Usage
//!
//! ```no_run
//! extern crate ruspiro_interrupt; // <- this kind of usage is VERY IMPORTANT to ensure linking works as expected!
//! use ruspiro_interrupt::*;
//!
//! #[IrqHandler(ArmTimer)]
//! fn timer_handler() {
//!     // IMPORTANT: acknowledge the irq !
//!
//!     // implement stuff that shall be executed if the interrupt is raised...
//!     // be careful when this code uses spinlocks as this might lead to dead-locks if the
//!     // executing code interrupted currently helds a lock the code inside this handler tries to aquire the same one
//! }
//!
//! fn main() {
//!     // as we have an interrupt handler defined we need to enable interrupt handling globally as well
//!     // as the specific interrupt we have a handler implemented for
//!     IRQ_MANAGER.take_for(|irq_mgr| {
//!         irq_mgr.initialize();
//!         irq_mgr.activate(Interrupt::ArmTimer);
//!         irq_mgr.activate(Interrupt::Aux);
//!     });
//!
//!     enable_interrupts();
//! }
//! ```
//!
//! In some cases the interrupt type/line is shared between different sources. In those cases a handler need to be
//! implemented for the specific interrupt source. The source is given in the custom attribute like this:
//!
//! ```no_run
//! extern crate ruspiro_interrupt; // <- this kind of usage is VERY IMPORTANT to ensure linking works as expected!
//! use ruspiro_interrupt::*;
//!
//! #[IrqHandler(Aux, Uart1)]
//! fn aux_uart1_handler() {
//!     // implement Uart1 interrupt handler here
//! }
//!
//! # fn main() {
//! # }
//! ```
//!
//! # Limitations
//!
//! However, only a limited ammount of shared interrupts is available with the current version - which is only the **Aux**
//! interrupt at the moment.
//!

extern crate alloc;
extern crate paste;

pub use ruspiro_interrupt_core::*;
pub use ruspiro_interrupt_macros::*;
pub mod irqtypes;
pub use irqtypes::*;

use ruspiro_singleton::Singleton;

mod auxhandler;
mod interface;

use alloc::vec::*;

/// The singleton accessor to the interrupt manager
pub static IRQ_MANAGER: Singleton<InterruptManager> = Singleton::new(InterruptManager::new());

/// The interrupt manager representation
pub struct InterruptManager {
    enabled: [u32; 3], // store the current active irq's of the different banks (shadowing the register value)
}

impl InterruptManager {
    pub(crate) const fn new() -> Self {
        InterruptManager { enabled: [0; 3] }
    }

    /// One time interrupt manager initialization. This performs the initial configuration and deactivates all IRQs
    pub fn initialize(&mut self) {
        interface::initialize();
    }

    /// activate a specific interrupt to be raised and handled (id a handler is implemented)
    /// if there is no handler implemented for this interrupt it may lead to an endless interrupt
    /// loop as the interrupt never gets acknowledged by the handler.
    /// The is unfortunately no generic way of acknowledgement implementation possible as the acknowledge
    /// register and process differs for the individual interrupts.
    pub fn activate(&mut self, irq: Interrupt) {
        let irq_num = irq as u32;
        let irq_bank = irq_num >> 5;

        self.enabled[irq_bank as usize] |= 1 << (irq_num & 0x1F);

        interface::activate(irq_bank, irq_num);
        //println!("enabled Irq's: {:X}, {:X}, {:X}", self.enabled[0], self.enabled[1], self.enabled[2]);
        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        unsafe {
            llvm_asm!("dmb sy")
        };
    }

    /// deactivate a specific interrupt from beeing raised. This ensures the handler will also not getting called any
    /// longer
    pub fn deactivate(&mut self, irq: Interrupt) {
        let irq_num = irq as u32;
        let irq_bank = irq_num >> 5;

        interface::deactivate(irq_bank, irq_num);

        self.enabled[irq_bank as usize] &= !(1 << (irq_num & 0x1F));
        #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
        unsafe {
            llvm_asm!("dmb sy")
        };
    }
}

/********************************************************************************************
 * Functions that need to be exported, this seem not to work if they are part of of a child
 * module, so define them here
 ********************************************************************************************/
#[allow(dead_code)]
#[repr(u32)]
enum ExceptionType {
    CurrentElSp0Sync = 0x01,
    CurrentElSp0Irq = 0x02,
    CurrentElSp0Fiq = 0x03,
    CurrentElSp0SErr = 0x04,

    CurrentElSpxSync = 0x11,
    CurrentElSpxIrq = 0x12,
    CurrentElSpxFiq = 0x13,
    CurrentElSpxSErr = 0x14,

    LowerEl64SpxSync = 0x21,
    LowerEl64SpxIrq = 0x22,
    LowerEl64SpxFiq = 0x23,
    LowerEl64SpxSErr = 0x24,

    LowerEl32SpxSync = 0x31,
    LowerEl32SpxIrq = 0x32,
    LowerEl32SpxFiq = 0x33,
    LowerEl32SpxSErr = 0x34,

    A32UndefInstruction = 0x50,
    A32SoftwareInterrupt = 0x51,
    A32PrefetchAbort = 0x52,
    A32DataAbort = 0x53,
    A32Irq = 0x54,
    A32Fiq = 0x55,
}

/// The default exception handler.
/// This is the entry point for any exception taken at any core. The type gives the hint on what
/// the exyception is about, sync, irq, etc. This entry point is called from the ``ruspiro-boot``
/// crate when this is used for bootstrapping. Otherwise the custom bootstrapping need to properly
/// setup the exception table and call this entry point with the required input
///
#[cfg(target_arch = "aarch64")]
#[no_mangle]
unsafe fn __exception_handler_default(
    exception: ExceptionType,
    _esr: u64,
    _spsr: u64,
    _far: u64,
    _elr: u64,
) {
    match exception {
        ExceptionType::CurrentElSp0Irq => interrupt_handler(),
        ExceptionType::CurrentElSp0Fiq => interrupt_handler(),
        ExceptionType::CurrentElSpxIrq => interrupt_handler(),
        ExceptionType::CurrentElSpxFiq => interrupt_handler(),
        _ => (),
    }
}

#[cfg(target_arch = "arm")]
#[no_mangle]
unsafe fn __exception_handler_default(
    exception: ExceptionType,
    _spsr: u32,
    _sp_irq: u32,
    _lr_irq: u32,
) {
    match exception {
        ExceptionType::A32Irq => interrupt_handler(),
        ExceptionType::A32Fiq => interrupt_handler(),
        _ => (),
    }
}

/// Entry point for interrupt handling. This function dispatches the detected interrupt to the
/// elsewhere implemented dedicated handlers. Those handlers should be tagged with the ``IrqHandler``
/// attribute
#[allow(dead_code)]
fn interrupt_handler() {
    // first globally store that we are inside an interrupt handler
    entering_interrupt_handler();

    // now retrieve the pending interrupts
    let pendings = interface::get_pending_irqs();

    // from the pending interrupts filter the active ones
    let active = IRQ_MANAGER.use_for(|mgr| {
        [
            mgr.enabled[0] & pendings[0],
            mgr.enabled[1] & pendings[1],
            mgr.enabled[2] & pendings[2],
        ]
    });

    // now that we have the active interrupts we can dispatch to the dedicated handlers
    if active[0] != 0 {
        // IRQ Bank 1
        if active[0] & (1 << 1) != 0 {
            __irq_handler__SystemTimer1()
        }
        if active[0] & (1 << 3) != 0 {
            __irq_handler__SystemTimer3()
        }
        if active[0] & (1 << 8) != 0 {
            __irq_handler__Isp()
        }
        if active[0] & (1 << 9) != 0 {
            __irq_handler__Usb()
        }
        if active[0] & (1 << 12) != 0 {
            __irq_handler__CoreSync0()
        }
        if active[0] & (1 << 13) != 0 {
            __irq_handler__CoreSync1()
        }
        if active[0] & (1 << 14) != 0 {
            __irq_handler__CoreSync2()
        }
        if active[0] & (1 << 15) != 0 {
            __irq_handler__CoreSync3()
        }
        if active[0] & (1 << 29) != 0 {
            auxhandler::aux_handler()
        }
        if active[0] & (1 << 30) != 0 {
            __irq_handler__Arm()
        }
        if active[0] & (1 << 31) != 0 {
            __irq_handler__GpuDma()
        }
    }

    if active[1] != 0 {
        // IRQ Bank 2
        if active[1] & (1 << (49 - 32)) != 0 {
            __irq_handler__GpioBank0()
        }
        if active[1] & (1 << (50 - 32)) != 0 {
            __irq_handler__GpioBank1()
        }
        if active[1] & (1 << (51 - 32)) != 0 {
            __irq_handler__GpioBank2()
        }
        if active[1] & (1 << (52 - 32)) != 0 {
            __irq_handler__GpioBank3()
        }
        if active[1] & (1 << (53 - 32)) != 0 {
            __irq_handler__I2c()
        }
        if active[1] & (1 << (54 - 32)) != 0 {
            __irq_handler__Spi()
        }
        if active[1] & (1 << (55 - 32)) != 0 {
            __irq_handler__I2sPcm()
        }
        if active[1] & (1 << (56 - 32)) != 0 {
            __irq_handler__Sdio()
        }
        if active[1] & (1 << (57 - 32)) != 0 {
            __irq_handler__Pl011()
        }
    }

    if active[2] != 0 {
        // IRQ Bank Basic
        if active[2] & (1 << (64 - 64)) != 0 {
            __irq_handler__ArmTimer()
        }
        if active[2] & (1 << (65 - 64)) != 0 {
            __irq_handler__ArmMailbox()
        }
        if active[2] & (1 << (66 - 64)) != 0 {
            __irq_handler__ArmDoorbell0()
        }
        if active[2] & (1 << (67 - 64)) != 0 {
            __irq_handler__ArmDoorbell1()
        }
        if active[2] & (1 << (68 - 64)) != 0 {
            __irq_handler__ArmGpu0Halted()
        }
        if active[2] & (1 << (69 - 64)) != 0 {
            __irq_handler__ArmGpu1Halted()
        }
        if active[2] & (1 << (70 - 64)) != 0 {
            __irq_handler__ArmIllegalType1()
        }
        if active[2] & (1 << (71 - 64)) != 0 {
            __irq_handler__ArmIllegalType0()
        }
        if active[2] & (1 << (72 - 64)) != 0 {
            __irq_handler__ArmPending1()
        }
        if active[2] & (1 << (73 - 64)) != 0 {
            __irq_handler__ArmPending2()
        }
    }

    // when we are done store that we are leaving an interrupt handler
    leaving_interrupt_handler();
}

/// The IRQ handling entry point. This entrypoint is maintained by the ``rusprio-boot`` crate and points to
/// an empty implementation using a linker script "magic". Once interrupts are globally enabled and specific
/// interrupts has been activated this entry point will be called wheneever an interrupt occurs. The
/// function determines the interrupt source and dispatches the call to the corresponding handler implementations
/// done somewhere else
#[no_mangle]
unsafe fn __interrupt_h(_core: u32) {
    entering_interrupt_handler();
    IRQ_MANAGER.use_for(|mgr| {
        // check wheter the interrupt in a pending register really has been activiated
        let pendings: [u32; 3] = interface::get_pending_irqs();
        //info!("irq raised. pending: {:X}, {:X}, {:X}", pendings[0], pendings[1], pendings[2]);

        // build a list of interrupt id's based on the bit's set in the 3 irq enable banks
        let active_irqs: Vec<u8> = (0..).zip(&pendings).fold(Vec::new(), |acc, p| {
            acc.into_iter()
                .chain(set_bits_to_vec(
                    (*p.1) & mgr.enabled[p.0 as usize],
                    p.0 * 32,
                ))
                .collect()
        });

        for id in active_irqs {
            match id {
                1 => __irq_handler__SystemTimer1(),
                3 => __irq_handler__SystemTimer3(),
                8 => __irq_handler__Isp(),
                9 => __irq_handler__Usb(),
                12 => __irq_handler__CoreSync0(),
                13 => __irq_handler__CoreSync1(),
                14 => __irq_handler__CoreSync2(),
                15 => __irq_handler__CoreSync3(),
                29 => auxhandler::aux_handler(),
                30 => __irq_handler__Arm(),
                31 => __irq_handler__GpuDma(),
                49 => __irq_handler__GpioBank0(),
                50 => __irq_handler__GpioBank1(),
                51 => __irq_handler__GpioBank2(),
                52 => __irq_handler__GpioBank3(),
                53 => __irq_handler__I2c(),
                54 => __irq_handler__Spi(),
                55 => __irq_handler__I2sPcm(),
                56 => __irq_handler__Sdio(),
                57 => __irq_handler__Pl011(),
                64 => __irq_handler__ArmTimer(),
                65 => __irq_handler__ArmMailbox(),
                66 => __irq_handler__ArmDoorbell0(),
                67 => __irq_handler__ArmDoorbell1(),
                68 => __irq_handler__ArmGpu0Halted(),
                69 => __irq_handler__ArmGpu1Halted(),
                70 => __irq_handler__ArmIllegalType1(),
                71 => __irq_handler__ArmIllegalType0(),
                72 => __irq_handler__ArmPending1(),
                73 => __irq_handler__ArmPending2(),

                _ => __irq_handler_Default(),
            }
        }
    });
    leaving_interrupt_handler();
}

fn set_bits_to_vec(value: u32, base: u8) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::with_capacity(32);
    let mut check = value;
    for idx in 0..32 {
        if check == 0 {
            break;
        }
        if (check & 0x1) != 0 {
            v.push(idx + base);
        }
        check >>= 1;
    }

    v
}

macro_rules! default_handler_impl {
    ($($name:ident),*) => {$(
        paste::item!{
            #[allow(non_snake_case)]
            #[linkage="weak"]
            #[no_mangle]
            extern "C" fn [<__irq_handler__ $name>](){
                //__irq_handler_Default();
            }
        }
    )*};
}

default_handler_impl![
    SystemTimer1,
    SystemTimer3,
    Isp,
    Usb,
    CoreSync0,
    CoreSync1,
    CoreSync2,
    CoreSync3,
    Aux_Uart1,
    Aux_Spi1,
    Aux_Spi2,
    Arm,
    GpuDma,
    GpioBank0,
    GpioBank1,
    GpioBank2,
    GpioBank3,
    I2c,
    Spi,
    I2sPcm,
    Sdio,
    Pl011,
    ArmTimer,
    ArmMailbox,
    ArmDoorbell0,
    ArmDoorbell1,
    ArmGpu0Halted,
    ArmGpu1Halted,
    ArmIllegalType1,
    ArmIllegalType0,
    ArmPending1,
    ArmPending2
];

#[allow(non_snake_case)]
#[no_mangle]
fn __irq_handler_Default() {}
