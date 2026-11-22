use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    AttrStyle, Attribute, DeriveInput, Fields, Ident, Meta, Path, Token, parse_macro_input,
    punctuated::Punctuated,
};

fn find_error(attrs: Vec<Attribute>) -> (Path, TokenStream) {
    let mut error = None;

    for attr in attrs {
        let AttrStyle::Outer = attr.style else {
            continue;
        };

        let Meta::List(meta_list) = attr.meta else {
            continue;
        };

        if !meta_list.path.is_ident("error") {
            continue;
        }

        if error.is_some() {
            panic!("You can only use the error attribute once on an enum");
        }

        let mut paths = meta_list
            .parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
            .expect("Failed to parse attribute arguments")
            .into_iter();

        let error_type = paths.next().expect("Error type is missing");
        let error_variant = match paths.next() {
            Some(variant) => {
                if paths.next().is_some() {
                    panic!("Expected at most 2 attribute arguments");
                }

                quote! { #error_type::#variant }
            }
            None => quote! { #error_type },
        };

        error = Some((error_type, error_variant))
    }

    error.expect("An error attribute is necessary")
}

fn to_snake_case(s: String) -> String {
    s.chars()
        .enumerate()
        .fold(String::new(), |mut s, (idx, ch)| {
            if ch.is_uppercase() {
                if idx != 0 {
                    s.push('_');
                }

                s.extend(ch.to_lowercase());
            } else {
                s.push(ch);
            }

            s
        })
}

#[proc_macro_derive(EnumTryAs, attributes(error))]
pub fn enum_try_as(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (error_type, error_variant) = find_error(input.attrs);

    let methods = match input.data {
        syn::Data::Enum(data_enum) => data_enum.variants,
        _ => unimplemented!("Enum try as works only for enums"),
    }
    .into_iter()
    .map(|variant| {
        let variant_name = variant.ident;

        let variant_snake_name = Ident::new(
            &to_snake_case(variant_name.to_string()),
            variant_name.span(),
        );

        let variant_spaced_name = Ident::new(
            &variant_name.to_string().replace('_', " "),
            variant_name.span(),
        );

        let method = format_ident!("try_as_{}", variant_snake_name);

        let (val, body) = match variant.fields {
            Fields::Named(fields_named) => {
                let variables = fields_named
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().expect("Failed to get name of field"));

                let return_values = variables.clone();

                let values = fields_named.named.iter().map(|f| &f.ty);

                let val = quote! {
                    ( #( & #values ),* )
                };

                let body = quote! {
                    match self {
                        #name::#variant_name { #( #variables ),* } => Ok(( #( #return_values ),* )),
                        _ => Err(#error_variant(stringify!(#variant_spaced_name))),
                    }
                };

                (val, body)
            }
            Fields::Unnamed(fields_unnamed) => {
                // #fields_unnamed
                let variables =
                    fields_unnamed
                        .unnamed
                        .iter()
                        .enumerate()
                        .fold(vec![], |mut vec, (i, _)| {
                            vec.push(
                                TokenStream::from_str(&format!("_{i}"))
                                    .expect("Failed to convert string to token stream"),
                            );
                            vec
                        });

                let return_values = variables.clone();

                let types = fields_unnamed.unnamed.iter().map(|x| &x.ty);
                let val = quote! { ( #( &#types ),* ) };

                let body = quote! {
                    match self {
                        #name::#variant_name ( #( #variables ),* ) => Ok(( #( #return_values ),* )),
                        _ => Err(#error_variant(stringify!(#variant_spaced_name))),
                    }
                };

                (val, body)
            }
            Fields::Unit => (
                quote! { () },
                quote! {
                    match self {
                        #name::#variant_name => Ok(()),
                        _ => Err(#error_variant(stringify!(#variant_spaced_name)))
                    }
                },
            ),
        };

        quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                pub fn #method(&self) -> ::std::result::Result<#val, #error_type> {
                    #body
                }
            }
        }
    });

    let expanded = quote! {
        #( #methods )*
    };

    proc_macro::TokenStream::from(expanded)
}
