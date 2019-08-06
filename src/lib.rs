/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-interrupt/0.1.0")]
#![no_std]
#![feature(asm)]
#![feature(linkage)]

//! # Raspberry Pi Interrupt handler
//! 
//! This crates provides functions and macros (custom attribute) to conviniently implement interrupt handler for 
//! Raspberry Pi 3. The possible interrupts a handler can be implemented for are available as enum [irqtypes::Interrupt]
//! 
//! # Usage
//! 
//! ```
//! use ruspiro_interrupt::*;
//! 
//! #[IrqHandler(ArmTimer)]
//! fn timer_handler() {
//!     // TODO: acknowledge the irq
//! 
//!     // implement stuff that shall be executed if the interrupt is raised...
//!     // be careful when this code uses spinlocks as this might lead to dead-locks if the 
//!     // executing code interrupted currently helds a lock the code inside this handler tries to aquire the same one
//!     println('timer interrupt raised');
//! }
//! 
//! fn demo() {
//!     // as we have an interrupt handler defined we need to enable interrupt handling globally as well
//!     // as the specific interrupt we have a handler implemented for
//!     IRQ_MANAGER.take_for(|mgr| {
//!         mgr.enable();
//!         mgr.activate(Interrupt.ArmTimer);
//!     });
//! }
//! ```
//! 

extern crate alloc;

pub use ruspiro_interrupt_macros::*;
pub mod irqtypes;
pub use irqtypes::*;

use ruspiro_singleton::Singleton;
mod interface;

use alloc::vec::*;

/// The singleton accessor to the interrupt manager
pub static IRQ_MANAGER: Singleton<InterruptManager> = Singleton::new(InterruptManager::new());

/// The interrupt manager representation
pub struct InterruptManager {
    enabled: [u32; 3], // store the current active irq's of the different banks (shadowing the register value)
}

impl InterruptManager {
    pub(crate) const fn new () -> Self {
        InterruptManager {
            enabled: [0; 3],
        }
    }

    /// One time interrupt manager initialization. This performs the initial configuration and deactivates all IRQs
    pub fn initialize(&mut self) {
        interface::initialize();
    }

    /// globally enable interrupts
    pub fn enable(&self) {
        interface::enable_i();
        interface::enable_f();
    }

    /// globally disable interrupts
    pub fn disable(&self) {
        interface::disable_i();
        interface::disable_f();
    }

    /// activate a specific interrupt to be raised and handled (id a handler is implemented)
    /// if there is no handler implemented for this interrupt it may lead to an endless interrupt
    /// loop as the interrupt never gets acknowledged by the handler.
    /// The is unfortunately no generic way of acknowledgement implementation possible as the acknowledge
    /// register and process differs for the individual interrupts.
    pub fn activate(&mut self, irq: Interrupt) {
        let irq_num = irq as u32;
        let irq_bank = irq_num >> 5;
        
        interface::activate(irq_bank, irq_num);
        
        self.enabled[irq_bank as usize] |= 1 << (irq_num & 0x1F);
        unsafe{ asm!("dmb") };
    }

    /// deactivate a specific interrupt from beeing raised. This ensures the handler will also not getting called any
    /// longer
    pub fn deactivate(&mut self, irq: Interrupt) {
        let irq_num = irq as u32;
        let irq_bank = irq_num >> 5;

        interface::deactivate(irq_bank, irq_num);
        
        self.enabled[irq_bank as usize] &= !(1 << (irq_num & 0x1F));
        unsafe{ asm!("dmb") };
    }
}

/********************************************************************************************
 * Functions that need to be exported, this seem not to work if they are part of of a child
 * module, so define them here
 ********************************************************************************************/
/// The IRQ handling entry point. This entrypoint is maintained by the ``rusprio-boot`` crate and points to
/// an empty implementation using a linker script "magic". Once interrupts are globally enabled and specific
/// interrupts has been activated this entry point will be called wheneever an interrupt occurs. The
/// function determines the interrupt source and dispatches the call to the corresponding handler implementations
/// done somewhere else
#[no_mangle]
unsafe fn __interrupt_h(_core: u32) {
    IRQ_MANAGER.use_weak_for(|mgr| {
        // check wheter the interrupt in a pending register really has been activiated
        let pendings: [u32; 3] = interface::get_pending_irqs();

        // build a list of interrupt id's based on the bit's set in the 3 irq enable banks
        let active_irqs: Vec<u8> = (0..).zip(&pendings).fold(Vec::new(), |acc, p| {
            acc
                .into_iter()
                .chain(
                    set_bits_to_vec(
                        (*p.1) & mgr.enabled[p.0 as usize], p.0*32)
                    )
                .collect()
        });

        for id in active_irqs {
            match id {
                1   => __irq_handler__SystemTimer1(),
                3   => __irq_handler__SystemTimer3(),
                8   => __irq_handler__Isp(),
                9   => __irq_handler__Usb(),
                12  => __irq_handler__CoreSync0(),
                13  => __irq_handler__CoreSync1(),
                14  => __irq_handler__CoreSync2(),
                15  => __irq_handler__CoreSync3(),
                29  => __irq_handler__Aux(),
                30  => __irq_handler__Arm(),
                31  => __irq_handler__GpuDma(),
                49  => __irq_handler__GpioBank0(),
                50  => __irq_handler__GpioBank1(),
                51  => __irq_handler__GpioBank2(),
                52  => __irq_handler__GpioBank3(),
                53  => __irq_handler__I2c(),
                54  => __irq_handler__Spi(),
                55  =>__irq_handler__I2sPcm(),
                56  => __irq_handler__Sdio(),
                57  => __irq_handler__Pl011(),
                64  => __irq_handler__ArmTimer(),
                65  => __irq_handler__ArmMailbox(),
                66  => __irq_handler__ArmDoorbell0(),
                67  => __irq_handler__ArmDoorbell1(),
                68  => __irq_handler__ArmGpu0Halted(),
                69  => __irq_handler__ArmGpu1Halted(),
                70  => __irq_handler__ArmIllegalType1(),
                71  => __irq_handler__ArmIllegalType0(),
                72  => __irq_handler__ArmPending1(),
                73  => __irq_handler__ArmPending2(),

                _ => __irq_handler_Default()
            }
        }
    });
}

fn set_bits_to_vec(value: u32, base: u8) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::with_capacity(32);
    let mut check = value;
    for idx in 0..32 {
        if check == 0 { break; }
        if (check & 0x1) != 0 { v.push(idx + base); }
        check >>= 1;
    }

    v
}


#[allow(non_snake_case)]
#[no_mangle]
fn __irq_handler_Default() {

}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__SystemTimer1(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__SystemTimer3(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__Isp(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__Usb(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__CoreSync0(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__CoreSync1(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__CoreSync2(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__CoreSync3(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__Aux(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__Arm(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__GpuDma(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__GpioBank0(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__GpioBank1(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__GpioBank2(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__GpioBank3(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__I2c(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__Spi(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__I2sPcm(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__Sdio(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__Pl011(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__ArmTimer(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__ArmMailbox(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__ArmDoorbell0(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__ArmDoorbell1(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__ArmGpu0Halted(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__ArmGpu1Halted(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__ArmIllegalType1(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__ArmIllegalType0(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__ArmPending1(){
    __irq_handler_Default();
}

#[allow(non_snake_case)]
#[linkage="weak"]
#[no_mangle]
extern "C" fn __irq_handler__ArmPending2(){
    __irq_handler_Default();
}
