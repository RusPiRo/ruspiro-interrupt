/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-interrupt-macro/0.0.0")]

//! # Interrupt Macros
//! 
//! This crate provides the custom attribute ``#[IrqHandler(<interrupt type>)]`` to be used when implementing an interrupt handler.
//! 
//! # Usage
//! 
//! ```
//! #[IrqHandler(ArmTimer)]
//! unsafe fn MyTimerHandler() {
//!     // implement the interrupt handling here and do not forget
//!     // to acknowledge the interrupt in the interrupt specific registers
//!     // in case of the timer interrupt it would be done like so
//! 
//!     TIMERIRQ::Register.set(1);
//! }
//! 
//! define_registers! ( TIMERIRQ: WriteOnly<u32> @ 0x3F00_B40C => [] );
//! ```
//! 

extern crate proc_macro;
extern crate syn;
extern crate quote;

use proc_macro::*;
use syn::*;
use quote::quote;

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn IrqHandler(attr: TokenStream, item: TokenStream) -> TokenStream {
    // indicate usage of this macro in the compiler output
    println!("implement handler for IRQ: \"{}\"", attr.to_string());
    //let func: ItemFn = syn::parse(item).expect("`#IrqHandler` must be applied to a function");
    //let args: AttributeArgs = syn::parse(attr).expect("This attribute requires 1 parameter");

    let func = parse_macro_input!(item as ItemFn);
    let args:AttributeArgs = parse_macro_input!(attr as AttributeArgs);
    let irq_name = match args.get(0) {
        Some(NestedMeta::Meta(Meta::Word(meta))) => meta,
        _ => return syn::Error::new(syn::export::Span::call_site(), "interrupt identifier missing in `#[IrqHandler(identifier)`")
                .to_compile_error()
                .into(),
    };

    let ident = func.ident; // original function identifier
    let attrs = func.attrs; // function attributes #[...]
    let block = func.block; // function block
    let stmts = block.stmts; // function statements

    let irq_name_s = format!("__irq_handler__{}", irq_name.to_string());

    quote!(
        // use a fixed export name to ensure the same irq handler is not implemented twice
        #[allow(non_snake_case)]
        #[export_name = #irq_name_s]
        #(#attrs)*
        pub unsafe extern "C" fn #ident() {
            // force compiler error if the irq_name does not appear in the Interrupt enum that need to be
            // referred to in the crate using this one 
            crate::irqtypes::Interrupt::#irq_name;

            #(#stmts)*
        }
    ).into()
}