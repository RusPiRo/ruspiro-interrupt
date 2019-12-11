/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-interrupt-macros/0.2.1")]

//! # Interrupt Macros
//!
//! This crate provides the custom attribute ``#[IrqHandler(<interrupt type>[, <source>])]`` to be used when implementing an
//! interrupt handler.
//!
//! # Usage
//!
//! ```no_run
//! use ruspiro_interrupt_macros::*;
//! 
//! #[IrqHandler(ArmTimer)]
//! unsafe fn my_timer_handler() {
//!     // implement the interrupt handling here and do not forget
//!     // to acknowledge the interrupt in the interrupt specific registers
//! }
//! 
//! # fn main() { }
//! ```
//!
//! In some rare cases the interrupt line is shared between specific interrupt sources. In this case the source of
//! the interrupt need to be passed as well as an identifier.
//! ```no_run
//! use ruspiro_interrupt_macros::*;
//! 
//! #[IrqHandler(Aux, Uart1)]
//! unsafe fn my_aux_uart1_handler() {
//!     // handle Uart1 interrupt here - this usually has no "acknowledge" register...
//! }
//! 
//! # fn main() { }
//! ```
//!
//!

extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::*;
use quote::quote;
use syn::*;

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn IrqHandler(attr: TokenStream, item: TokenStream) -> TokenStream {
    // indicate usage of this macro in the compiler output
    println!("implement handler for IRQ: \"{}\"", attr.to_string());

    let func = parse_macro_input!(item as ItemFn);
    let args: AttributeArgs = parse_macro_input!(attr as AttributeArgs);
    let irq_name = match args.get(0) {
        Some(NestedMeta::Meta(Meta::Word(meta))) => meta,
        _ => {
            return syn::Error::new(
                syn::export::Span::call_site(),
                "interrupt identifier missing in `#[IrqHandler(identifier)`",
            )
            .to_compile_error()
            .into()
        }
    };

    // verify the signature of the function given to the handler
    // the required signature may differ depending on the interrupt that shall be handled
    // first check the basic ones
    let valid_common_signature = func.constness.is_none()  // no "fn const"
        && func.vis == Visibility::Inherited        // inherited in the current crate?
        && func.abi.is_none()                       // no "extern" is used
        && func.decl.generics.params.is_empty()     // no generics like fn handler<T>
        && func.decl.generics.where_clause.is_none() // no generics in a where clause like fn handler(a: T) where T: FnOnce
        && func.decl.variadic.is_none()             // no variadic parameters like fn handler(...)
        && match func.decl.output {                 // only default return types allowed
            ReturnType::Default => true,
            _ => false,
        };

    let irq_id_s = irq_name.to_string();
    let irq_func_suffix = match &*irq_id_s {
        "Aux" => {
            // Aux IrqHandler tag signature is: IrqHandler(Aux,Uart1)
            let aux_source = match args.get(1) {
                Some(NestedMeta::Meta(Meta::Word(meta))) => meta,
                _=> return syn::Error::new(syn::export::Span::call_site(), "`Aux` interrupt source missing in `#[IrqHandler(Aux, <SOURCE>)`. <SOURCE> could be one of: `Uart1` | `Spi1` | `Spi2`.")
                            .to_compile_error()
                            .into()
            };
            let aux_source_s = aux_source.to_string();
            // check for valid Aux types
            if &*aux_source_s != "Uart1" && &*aux_source_s != "Spi1" && &*aux_source_s != "Spi2" {
                return syn::Error::new(syn::export::Span::call_site(), "Wrong source for `Aux` interrupt in `#[IrqHandler(Aux, <SOURCE>)`. <SOURCE> could be one of: `Uart1` | `Spi1` | `Spi2`.")
                        .to_compile_error()
                        .into();
            }
            let valid_signature = valid_common_signature && func.decl.inputs.is_empty();

            if !valid_signature {
                return syn::Error::new(
                    syn::export::Span::call_site(),
                    "interrupt handler must have signature `[unsafe] fn()`",
                )
                .to_compile_error()
                .into();
            }

            format!("{}_{}", irq_name.to_string(), aux_source.to_string())
        }
        _ => {
            let valid_signature = valid_common_signature && func.decl.inputs.is_empty();

            if !valid_signature {
                return syn::Error::new(
                    syn::export::Span::call_site(),
                    "interrupt handler must have signature `[unsafe] fn()`",
                )
                .to_compile_error()
                .into();
            }

            irq_name.to_string()
        }
    };

    let ident = func.ident; // original function identifier
    let attrs = func.attrs; // function attributes #[...]
    let block = func.block; // function block
    let stmts = block.stmts; // function statements

    let irq_name_s = format!("__irq_handler__{}", irq_func_suffix);
    quote!(
        // use a fixed export name to ensure the same irq handler is not implemented twice
        #[allow(non_snake_case)]
        #[export_name = #irq_name_s]
        #(#attrs)*
        #[no_mangle]
        pub unsafe fn #ident() {
            // force compiler error if the irq_name does not appear in the Interrupt enum that need to be
            // referred to in the crate using this attribute
            self::irqtypes::Interrupt::#irq_name;

            #(#stmts)*
        }
    )
    .into()
}
