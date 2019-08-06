/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-interrupt/0.1.0")]

//! # Interrupt Macros
//! 
//! This crate provides the custom attribute ``#[IrqHandler(<interrupt type>)]`` to be used when implementing an 
//! interrupt handler.
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
    match &*irq_id_s {
        /*"GpioBank0" | "GpioBank1" | "GpioBank2" => {
            let valid_signature = valid_common_signature
                && func.decl.inputs.len() == 1;
                /* the following checks if the given parameter is a imutable reference
                   but we needed to check for a u32 parameter which seem not to be possible so keep it as 
                   a check for exactly one parameter is given...
                && match func.decl.inputs[0] {
                    FnArg::Captured(ref arg) => match arg.ty {
                        Type::Reference(ref r) => r.lifetime.is_none() && r.mutability.is_none(),
                        _ => false,
                    },
                    _ => false,
                };*/
            
            if !valid_signature {
                return syn::Error::new(syn::export::Span::call_site(), "interrupt handler must have signature `[unsafe] fn(u32)`")
                .to_compile_error()
                .into()
            }    
        },*/

        _ => {
            let valid_signature = valid_common_signature 
                && func.decl.inputs.is_empty();

            if !valid_signature {
                return syn::Error::new(syn::export::Span::call_site(), "interrupt handler must have signature `[unsafe] fn()`")
                .to_compile_error()
                .into()
            }
        },
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
        #[no_mangle]
        pub unsafe fn #ident() {
            // force compiler error if the irq_name does not appear in the Interrupt enum that need to be
            // referred to in the crate using this attribute
            crate::irqtypes::Interrupt::#irq_name;

            #(#stmts)*
        }
    ).into()
}