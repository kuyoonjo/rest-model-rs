extern crate proc_macro;
use core::panic;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

/// Procedural macro to create a new struct with optional fields and copied derives
#[proc_macro_attribute]
pub fn rest_model(args: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input token stream as a struct
    let input = parse_macro_input!(item as DeriveInput);
    let original = input.clone();
    let struct_name = input.ident;

    let mut get = false;
    let mut get_with_id = false;
    let mut put = false;
    let mut patch = false;
    let mut delete = false;

    // Parse args
    let mut db: Option<Ident> = None;
    let mut db_name: Option<Ident> = None;
    let mut table_name: Option<Ident> = None;
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("db") {
            let mut i = 0;
            meta.parse_nested_meta(|meta| {
                let ident = meta.path.get_ident().unwrap();
                if i == 0 {
                    db = Some(ident.clone());
                } else if i == 1 {
                    db_name = Some(ident.clone());
                } else if i == 2 {
                    table_name = Some(ident.clone());
                } else {
                    return Err(meta.error("rest_model db only support 3 params"));
                }
                i += 1;
                Ok(())
            })?;
            if i != 1 || i != 3 {
                return Err(meta.error("rest_model db only support 1 or 3 params"));
            }
            Ok(())
        } else if meta.path.is_ident("with") {
            meta.parse_nested_meta(|meta| {
                if meta.path.is_ident("get") {
                    get = true;
                    Ok(())
                } else if meta.path.is_ident("get_with_id") {
                    get_with_id = true;
                    Ok(())
                } else if meta.path.is_ident("put") {
                    put = true;
                    Ok(())
                } else if meta.path.is_ident("patch") {
                    patch = true;
                    Ok(())
                } else if meta.path.is_ident("delete") {
                    delete = true;
                    Ok(())
                } else if meta.path.is_ident("all") {
                    get = true;
                    get_with_id = true;
                    put = true;
                    patch = true;
                    delete = true;
                    Ok(())
                } else {
                    Err(meta.error("unsupported rest_model with property"))
                }
            })
        } else {
            Err(meta.error(format!(
                "unsupported rest_model property `{}`",
                meta.path.get_ident().unwrap().to_string()
            )))
        }
    });

    parse_macro_input!(args with parser);

    if db.is_none() {
        panic!("Db must be specified");
    }

    // Generate CRUD methods based on the configuration
    let mut methods = quote! {
        impl rest_model::method::Init<#struct_name, #db> for #struct_name {}
    };

    if db_name.is_some() && table_name.is_some() {
        methods.extend(quote! {
            impl rest_model::RestModel for #struct_name {
                fn get_db_name() -> &'static str {
                    #db_name
                }
                fn get_table_name() -> &'static str {
                    #table_name
                }
            }
        });
    }

    if get_with_id {
        methods.extend(quote! {
            impl rest_model::method::GetWithId<#struct_name, #db> for #struct_name {}
        });
    }
    if get {
        methods.extend(quote! {
            impl rest_model::method::Get<#struct_name, #db> for #struct_name {}
        });
    }
    if put {
        methods.extend(quote! {
            impl rest_model::method::Put<#struct_name, #db> for #struct_name {}
        });
    }
    if patch {
        methods.extend(quote! {
            impl rest_model::method::Patch<#struct_name, #db> for #struct_name {}
        });
    }
    if delete {
        methods.extend(quote! {
            impl rest_model::method::Delete<#struct_name, #db> for #struct_name {}
        });
    }

    // Generate the new struct with optional fields and the copied derives
    let expanded = quote! {
        #original
        #methods
    };

    // Convert the expanded code into a TokenStream
    TokenStream::from(expanded)
}
