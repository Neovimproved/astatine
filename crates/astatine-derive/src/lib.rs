use std::{fmt::Debug, str::FromStr};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    AttrStyle, Attribute, Data, DeriveInput, Expr, Fields, Ident, Lit, Meta, MetaList, Path, Token,
    Variant, parse_macro_input, punctuated::Punctuated,
};

fn set_err_with_meta_list(error: &mut Option<(Path, TokenStream)>, meta_list: MetaList) {
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

    *error = Some((error_type, error_variant));
}

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

        set_err_with_meta_list(&mut error, meta_list);
    }

    error.expect("An error attribute is necessary")
}

fn to_snake_case(s: &str) -> String {
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

fn try_as_variant(variant: Variant, error_type: &Path, error_variant: &TokenStream) -> TokenStream {
    let ident = variant.ident;
    let variant_string = ident.to_string();

    let snake_name = to_snake_case(&variant_string);
    let spaced_name = Ident::new(&variant_string.replace('_', " "), ident.span());

    let method = format_ident!("try_as_{}", snake_name);

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
                    Self::#ident { #( #variables ),* } => Ok(( #( #return_values ),* )),
                    _ => Err(#error_variant(stringify!(#spaced_name))),
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
                if let self = Self::#ident ( #( #variables ),* ) {
                    Ok(( #( #return_values ),* ))
                } else {
                    Err(#error_variant(stringify!(#spaced_name)))
                }
            };

            (val, body)
        }
        Fields::Unit => (
            quote! { () },
            quote! {
                if let self = Self::#ident else {
                    Ok(())
                } else {
                    _ => Err(#error_variant(stringify!(#spaced_name)))
                }
            },
        ),
    };

    quote! {
        pub fn #method(&self) -> ::std::result::Result<#val, #error_type> {
            #body
        }
    }
}

#[proc_macro_derive(EnumTryAs, attributes(error))]
pub fn enum_try_as(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    let variants = match input.data {
        Data::Enum(data_enum) => data_enum.variants,
        _ => unimplemented!("Enum try as works only for enums"),
    };

    let (error_type, error_variant) = find_error(input.attrs);

    let recurse = variants
        .into_iter()
        .map(|variant| try_as_variant(variant, &error_type, &error_variant));

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #( #recurse )*
        }
    };

    proc_macro::TokenStream::from(expanded)
}

#[allow(unused)]
fn dbg_thing(thing: impl Debug) -> String {
    let s = format!("{thing:#?}");

    let mut accumulated = s.lines().fold(String::new(), |mut acc, line| {
        let whitespace_end = line.chars().take_while(|ch| *ch == ' ').count();
        acc.push_str(&line[whitespace_end / 2..]);
        acc.push('\n');
        acc
    });

    accumulated.pop();

    accumulated
}

#[proc_macro_derive(PoweredEnum, attributes(power))]
pub fn powered_enum(token_stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(token_stream as DeriveInput);

    let data = match input.data {
        Data::Enum(data_enum) => data_enum,
        _ => unimplemented!("You must use an enum for the PoweredEnum macro"),
    };

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let variants = data.variants.into_iter().map(|var| {
        let variant_name = var.ident;
        let attrs = var.attrs;
        let mut power: Option<usize> = None;

        for attr in attrs {
            let AttrStyle::Outer = attr.style else {
                continue;
            };

            let Meta::NameValue(name_value) = attr.meta else {
                continue;
            };

            let mut segments = name_value.path.segments.into_iter();

            let Some(name) = segments.next() else {
                continue;
            };

            if name.ident != "power" {
                continue;
            }

            let Expr::Lit(literal) = name_value.value else {
                panic!(
                    "Invalid power value for {variant_name}. You must enter an unsigned integer"
                );
            };

            let Lit::Int(value) = literal.lit else {
                panic!(
                    "Invalid power value for {variant_name}. You must enter an unsigned integer"
                );
            };

            power = Some(
                value
                    .base10_parse::<usize>()
                    .expect("Failed to parse power"),
            );
        }

        if power.is_none() {
            panic!("{variant_name} must have a power attribute");
        }

        quote! { #variant_name => #power }
    });

    let stream = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn binding_power(&self) -> usize {
                match self {
                    #( Self::#variants, )*
                }
            }
        }
    };

    proc_macro::TokenStream::from(stream)
}
