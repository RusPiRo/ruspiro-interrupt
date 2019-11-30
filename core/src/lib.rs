/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-interrupt-core/0.3.0")]
#![no_std]
#![feature(asm)]

//! # Interrupt Core functions
//! 
//! Core functions to enable/disable interupts globally. This is splitted from the
//! [``ruspiro-interrupt``](https://crates.io/crates/ruspiro-interrupt) crate to remove circular dependencies between
//! the interrupt crate others (e.g. ``ruspiro-singleton``) crate.

use core::sync::atomic::{AtomicBool, Ordering};

// simple state to track whether we are currently running inside an IRQ
// this is usually set and cleared by the interrupt handler [interrupt_handler]
static IRQ_HANDLER_ACTIVE: AtomicBool = AtomicBool::new(false);

// last IRQ state before globally disabling interrupts
static IRQ_STATE: AtomicBool = AtomicBool::new(false);
// last FAULT/FIQ state before globally disabling fast interrupts
static FAULT_STATE: AtomicBool = AtomicBool::new(false);

pub fn entering_interrupt_handler() {
    IRQ_HANDLER_ACTIVE.store(true, Ordering::SeqCst);
}

pub fn leaving_interrupt_handler() {
    IRQ_HANDLER_ACTIVE.store(false, Ordering::SeqCst);
}

/// globally enabling interrupts (IRQ/FIQ) to be triggered
pub fn enable_interrupts() {
    enable_irq();
    enable_fiq();
}

/// globally disabling interrupts (IRQ/FIQ) from beeing triggered
pub fn disable_interrupts() {
    // in aarch64 mode the interrupts are disabled by default on entering
    // no need to disable
    #[cfg(target_arch="aarch64")]
    {
        if IRQ_HANDLER_ACTIVE.load(Ordering::SeqCst) { return; }
    }
    disable_irq();
    disable_fiq();
}

/// globally re-enabling interrupts (IRQ/FIQ) to be triggered. This is done based on the global state
/// that was set before the interrupts were disable using the [``disable_interrupts``] function.
pub fn re_enable_interrupts() {
    // in aarch64 mode the interrupts are disabled by default on entering
    // no need to re-enable when running inside interrupt handler
    #[cfg(target_arch="aarch64")]
    {
        if IRQ_HANDLER_ACTIVE.load(Ordering::SeqCst) { return; }
    }
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
    #[cfg(target_arch="aarch64")]
    unsafe { 
        asm!("msr daifclr, #2
              isb") // as per ARM spec the ISB ensures triggering pending interrupts
    };
}

/// globally re-enable ``IRQ`` interrupts to be triggered based on the global state that was set before disabling IRQ
/// interrupts wihin the [``disable_irq``] function.
fn re_enable_irq() {
    // re-enable interrupts if they have been enabled prior to disabling
    let state = IRQ_STATE.load(Ordering::SeqCst);

    if state {
        #[cfg(target_arch="arm")]
        unsafe { 
            asm!("cpsie i
                  isb") // as per ARM spec the ISB ensures triggering pending interrupts
        };
        #[cfg(target_arch="aarch64")]
        unsafe { 
            asm!("msr daifclr, #2
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
    #[cfg(target_arch="aarch64")]
    unsafe { 
        asm!("msr daifclr, #1
              isb") // as per ARM spec the ISB ensures triggering pending interrupts
    };
}

/// globally re-enable ``FIQ`` interrupts to be triggered based on the global state that was set before disabling FIQ
/// interrupts wihin the [``disable_fiq``] function.
fn re_enable_fiq() {
    // re-enable interrupts if they have been enabled prior to disabling
    let state = FAULT_STATE.load(Ordering::SeqCst);

    if state {
        #[cfg(target_arch="arm")]
        unsafe { 
            asm!("cpsie f
                isb") // as per ARM spec the ISB ensures triggering pending interrupts
        };
        #[cfg(target_arch="aarch64")]
        unsafe { 
            asm!("msr daifclr, #1
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
    #[cfg(target_arch="aarch64")]
    unsafe { asm!("msr daifset, #2") };

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
    #[cfg(target_arch="aarch64")]
    unsafe { asm!("msr daifset, #1") };

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
    #[cfg(target_arch="aarch64")]
    unsafe {
        let state: u32;
        asm!("MRS $0, DAIF":"=r"(state):::"volatile");
        // irq enabled if mask bit was not set
        return !((state >> 6) & 0x2);
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
    #[cfg(target_arch="aarch64")]
    unsafe {
        let state: u32;
        asm!("MRS $0, DAIF":"=r"(state):::"volatile");
        // fiq enabled if mask bit was not set
        return !((state >> 6) & 0x1);
    }
    // for non ARM targets there is nothing implemented to get current IRQ state, so return 0
    0
}