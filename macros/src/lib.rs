/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-interrupt-macros/||VERSION||")]

//! # Interrupt Macros
//!
//! This crate provides the custom attribute ``#[IrqHandler(<interrupt type>[, <source>])]`` to be used when
//! implementing an interrupt handler. Detailed documentation can be found in the `ruspiro-interrupt` crate.
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
    Some(NestedMeta::Meta(Meta::Path(meta))) => &meta.segments.first().unwrap().ident,
    _ => {
      return quote! {
          compile_error!("interrupt identifier missing in `#[IrqHandler(identifier)`");
      }
      .into();
    }
  };

  // verify the signature of the function given to the handler
  // the required signature may differ depending on the interrupt that shall be handled
  // first check the basic ones
  let valid_common_signature = func.sig.constness.is_none()  // no "fn const"
        && func.vis == Visibility::Inherited        // inherited in the current crate?
        && func.sig.abi.is_none()                       // no "extern" is used
        && func.sig.generics.params.is_empty()     // no generics like fn handler<T>
        && func.sig.generics.where_clause.is_none() // no generics in a where clause like fn handler(a: T) where T: FnOnce
        && func.sig.variadic.is_none()             // no variadic parameters like fn handler(...)
        && match func.sig.output {                 // only default return types allowed
            ReturnType::Default => true,
            _ => false,
        };

  let irq_id_s = irq_name.to_string();
  let irq_func_suffix = match &*irq_id_s {
    "Aux" => {
      // Aux IrqHandler tag signature is: IrqHandler(Aux,Uart1)
      let aux_source = match args.get(1) {
                Some(NestedMeta::Meta(Meta::Path(meta))) => meta,
                _=> {
                    return quote! {
                        compile_error!("`Aux` interrupt source missing in `#[IrqHandler(Aux, <SOURCE>)`. <SOURCE> could be one of: `Uart1` | `Spi1` | `Spi2`.");
                    }.into()
                }
            };
      let aux_source_s = aux_source.segments.first().unwrap().ident.to_string();
      // check for valid Aux types
      if &*aux_source_s != "Uart1" && &*aux_source_s != "Spi1" && &*aux_source_s != "Spi2" {
        return quote! { compile_error!("Wrong source for `Aux` interrupt in `#[IrqHandler(Aux, <SOURCE>)`. <SOURCE> could be one of: `Uart1` | `Spi1` | `Spi2`.");
                }.into();
      }
      let valid_signature = valid_common_signature; // && func.decl.inputs.is_empty();

      if !valid_signature {
        return quote! {
                    compile_error!("interrupt handler must have signature `[unsafe] fn(tx: IsrSender(Box<dyn Any>))`");
                }.into();
      }

      format!("{}_{}", irq_name.to_string(), aux_source_s)
    }
    _ => {
      let valid_signature = valid_common_signature; // && func.decl.inputs.is_empty();

      if !valid_signature {
        return quote! {
                    compile_error!("interrupt handler must have signature `[unsafe] fn(tx: IsrSender(Box<dyn Any>)))`");
                }.into();
      }

      irq_name.to_string()
    }
  };

  let ident = func.sig.ident; // original function identifier
  let attrs = func.attrs; // function attributes #[...]
  let block = func.block; // function block
  let stmts = block.stmts; // function statements

  let irq_name_s = format!("__irq_handler__{}", irq_func_suffix);
  return quote!(
    // use a fixed export name to ensure the same irq handler is not implemented twice
    #[allow(non_snake_case)]
    #[export_name = #irq_name_s]
    #(#attrs)*
    #[no_mangle]
    pub unsafe extern "C" fn #ident(
      channel: Option<ruspiro_interrupt::IsrSender<crate::alloc::boxed::Box<dyn core::any::Any>>>
    ) {
      // force compiler error if the irq_name does not appear in the Interrupt enum that need to be
      // referred to in the crate using this attribute
      ruspiro_interrupt::Interrupt::#irq_name;

      #(#stmts)*
    }
  )
  .into();
}
