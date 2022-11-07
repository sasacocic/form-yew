use crate::*;
use proc_macro2::{Ident, Span};
use syn::MetaNameValue;

fn parse_meta(m: syn::Meta) -> (Ident, TokenStream2) {
    let html_input_element: proc_macro2::TokenStream = quote! { web_sys::HtmlInputElement };
    let html_textbox_element: proc_macro2::TokenStream = quote! { web_sys::HtmlTextAreaElement };
    match m {
        syn::Meta::NameValue(MetaNameValue {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => {
            let html_tag = lit_str.value();

            // don't need this can fold it into the part below
            if !["input", "textarea"].contains(&html_tag.as_str()) {
                panic!("the tag doesn't match one of the tags: input or textarea")
            }

            let web_sys_element_type = match &*html_tag {
                "input" => html_input_element,
                "textarea" => html_textbox_element,
                _ => panic!("couldn't find correcponding type"),
            };

            (
                Ident::new(&html_tag, Span::call_site()),
                web_sys_element_type,
            )
        }
        _ => {
            panic!("found a attribute that didn't conform to Meta::NamedValue")
        }
    }
}

pub fn parse_struct_body(
    data: Data,
    enum_name: &proc_macro2::Ident,
) -> (
    TokenStream2,
    Vec<proc_macro2::Ident>,
    (Vec<Ident>, Vec<TokenStream2>),
) {
    /*
    I want to make this
    pub enum update#nameOfStruct {
        #fieldName1(#fieldNameType)

        ...
    }
     */

    // duplicated because I couldn't put this in the global scope of this module,
    // because of some reason I don't fully understand with consts - this use to be a const
    let html_input_element: proc_macro2::TokenStream = quote! { web_sys::HtmlInputElement };

    let mut struct_field_names = Vec::new();
    // let mut struct_data_types = Vec::new();
    let mut v = Vec::new();
    let mut attirbutes = Vec::new();
    match data {
        Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                for field in named {
                    let Field {
                        ident, ty, attrs, ..
                    } = field;
                    // struct_field_names.push(ident.expect("a name here"));
                    // struct_data_types.push(ty);
                    let id = ident.expect("a named field");

                    let mut html_tag = attrs
                        .iter()
                        .map(|v| v.parse_meta())
                        .filter(|v| v.is_ok())
                        .map(|v| v.unwrap())
                        .map(parse_meta)
                        .collect::<Vec<(Ident, TokenStream2)>>();

                    if html_tag.len() == 0 {
                        html_tag.push((
                            Ident::new("input", Span::call_site()),
                            quote! { #html_input_element },
                        ));
                    }

                    // all attributes found
                    // eprintln!("metas foun for field {} : {:#?}", id, html_tag);
                    attirbutes.append(&mut html_tag);

                    let ideny = format_ident!("{}", id.to_string().to_uppercase());
                    v.push(quote! { #ideny(#ty) });
                    struct_field_names.push(id);
                }
            }
            _ => panic!("no named fields"),
        },
        _ => {
            panic!("bad")
        }
    }

    // Vec<(Ident,Ident)> -> (Vec,Vec)
    let attributes = attirbutes.iter().fold(
        (Vec::new(), Vec::new()),
        |(mut tags, mut r#types), (tag, r#type)| {
            tags.push(tag.clone());
            r#types.push(r#type.clone());
            (tags, r#types)
        },
    );

    // only issue with this is I'd like the variants to be
    // capitalized, but I don't really think that matters that much
    (
        quote! {
            pub enum #enum_name {
                // #( #struct_field_names(#struct_data_types) ),*
                #( #v ),*
            }
        },
        struct_field_names,
        attributes,
    )
}
