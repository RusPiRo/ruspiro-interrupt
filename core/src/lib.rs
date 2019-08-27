/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-interrupt-core/0.2.0")]
#![no_std]
#![feature(asm)]

//! # Interrupt Core functions
//! 
//! Core functions to enable/disable interupts globally. This is splitted from the
//! [``ruspiro-interrupt``](https://crates.io/crates/ruspiro-interrupt) crate to remove circular dependencies between
//! the interrupt crate others (e.g. ``ruspiro-singleton``) crate.

use core::sync::atomic::{AtomicBool, Ordering};

// last IRQ state before globally disabling interrupts
static IRQ_STATE: AtomicBool = AtomicBool::new(false);
// last FAULT/FIQ state before globally disabling fast interrupts
static FAULT_STATE: AtomicBool = AtomicBool::new(false);

/// globally enabling interrupts (IRQ/FIQ) to be triggered
pub fn enable_interrupts() {
    enable_irq();
    enable_fiq();
}

/// globally disabling interrupts (IRQ/FIQ) from beeing triggered
pub fn disable_interrupts() {
    disable_irq();
    disable_fiq();
}

/// globally re-enabling interrupts (IRQ/FIQ) to be triggered. This is done based on the global state
/// that was set before the interrupts were disable using the [``disable_interrupts``] function.
pub fn re_enable_interrupts() {
    re_enable_irq();
    re_enable_fiq();
}


/// globally enable ``IRQ`` interrupts to be triggered
pub fn enable_irq() {
    #[cfg(target_arch="arm")]
    unsafe { 
        asm!("cpsie i
              isb") // as per ARM spec the ISB ensures triggering pending interrupts
    };
}

/// globally re-enabe ``IRQ`` interrupts to be triggered based on the global state that was set before disabling IRQ
/// interrupts wihin the [``disable_irq``] function.
pub fn re_enable_irq() {
    // re-enable interrupts if they have been enabled prior to disabling
    let state = IRQ_STATE.load(Ordering::SeqCst);

    if state {
        #[cfg(target_arch="arm")]
        unsafe { 
            asm!("cpsie i
                isb") // as per ARM spec the ISB ensures triggering pending interrupts
        }; 
    }
}

/// globally enable ``FIQ`` interrupts to be triggered
pub fn enable_fiq() {
    #[cfg(target_arch="arm")]
    unsafe { 
        asm!("cpsie f
              isb") // as per ARM spec the ISB ensures triggering pending interrupts
    };
}

/// globally re-enabe ``FIQ`` interrupts to be triggered based on the global state that was set before disabling FIQ
/// interrupts wihin the [``disable_fiq``] function.
pub fn re_enable_fiq() {
    // re-enable interrupts if they have been enabled prior to disabling
    let state = FAULT_STATE.load(Ordering::SeqCst);

    if state {
        #[cfg(target_arch="arm")]
        unsafe { 
            asm!("cpsie f
                isb") // as per ARM spec the ISB ensures triggering pending interrupts
        }; 
    }
}

/// globally disable ``IRQ`` interrupts from beeing triggered. This function stores the state of the current enabling/disabling
/// of interrupts. If ``disable`` is called multiple times after each other this will than ultimately store "disabled" as
/// last state. In this case a previous enabled state (before the multiple calls) is not able to recover with a call to [``re_enable_irq``].
pub fn disable_irq() {
    // remember the last IRQ state
    let state = get_interrupt_state();

    #[cfg(target_arch="arm")]
    unsafe { asm!("cpsid i") };

    // store the last interrupt state after interrupts have been
    // disabled to ensure interrupt free atomic operation
    IRQ_STATE.store(state != 0, Ordering::SeqCst);
}

/// globally disable ``FIQ`` interrupts from beeing triggered. This function stores the state of the current enabling/disabling
/// of interrupts. If ``disable`` is called multiple times after each other this will than ultimately store "disabled" as
/// last state. In this case a previous enabled state (before the multiple calls) is not able to recover with a call to [``re_enable_fiq``].
pub fn disable_fiq() {
    // remember the last FIQ state
    let state = get_fault_state();

    #[cfg(target_arch="arm")]
    unsafe { asm!("cpsid f") };

    // store the last interrupt state after interrupts have been
    // disabled to ensure interrupt free atomic operation
    FAULT_STATE.store(state != 0, Ordering::SeqCst);
}

#[allow(unreachable_code)]
fn get_interrupt_state() -> u32 {
    #[cfg(target_arch="arm")]
    unsafe {
        let state: u32;
        asm!("MRS $0, CPSR":"=r"(state):::"volatile");
        return state & 0x80;
    }
    // for non ARM targets there is nothing implemented to get current IRQ state, so return 0
    0
}

#[allow(unreachable_code)]
fn get_fault_state() -> u32 {    
    #[cfg(target_arch="arm")]
    unsafe {
        let state: u32;
        asm!("MRS $0, CPSR":"=r"(state):::"volatile");
        return state & 0x40;
    }
    // for non ARM targets there is nothing implemented to get current IRQ state, so return 0
    0
}