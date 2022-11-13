use std::path::Path;

use crate::*;
use proc_macro2::{Ident, Span};
use syn::{MetaNameValue, Type, TypePath};

fn parse_meta(m: syn::Meta) -> (Ident, TokenStream2) {
    let html_input_element: proc_macro2::TokenStream = quote! { web_sys::HtmlInputElement };
    let html_textbox_element: proc_macro2::TokenStream = quote! { web_sys::HtmlTextAreaElement };
    match m {
        syn::Meta::NameValue(MetaNameValue {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => {
            let html_tag = lit_str.value();

            let web_sys_element_type = match &*html_tag {
                "input" => html_input_element,
                "textarea" => html_textbox_element,
                "checkbox" => html_input_element,
                _ => panic!("ele needs to be one of: input, textarea, or checkbox"),
            };

            // TODO: remove this - only temp
            if &*html_tag == "checkbox" {
                (
                    Ident::new("input".into(), Span::call_site()),
                    web_sys_element_type,
                )
            } else {
                (
                    Ident::new(&html_tag, Span::call_site()),
                    web_sys_element_type,
                )
            }
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
    Vec<Type>,
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
    let mut struct_field_types = Vec::new();
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
                    struct_field_types.push(ty.clone());

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
        struct_field_types,
    )
}

pub fn generate_html_inputs(
    html_tag_types: Vec<Ident>,
    struct_field_names: Vec<String>,
    struct_field_types: Vec<Type>,
) -> Vec<TokenStream2> {
    // <#html_tag_type class={&p.class} onchange={update_func} value={struct_field_name} placeholder={#struct_field_names_as_strings} />

    let mut html = Vec::new();
    for ((html_tag_type, struct_field_name), struct_field_type) in html_tag_types
        .iter()
        .zip(struct_field_names)
        .zip(struct_field_types)
    {
        // if structfieldtype == "bool" {then generates}
        // I actually want this to work for Strings as well
        let html_tag = if &struct_field_name == "remote" {
            // without the checked attribute here the field is quite buggy and I don't know why.
            // checked also isn't rendered in the final html...
            quote! {
                <#html_tag_type type={"checkbox"} class={&p.class} onchange={update_func} checked={struct_field_name}/>
            }
        } else {
            quote! {
                <#html_tag_type class={&p.class} onchange={update_func} value={struct_field_name} placeholder={#struct_field_name} />
            }
        };

        html.push(html_tag);
    }

    html
}

/*
    not a great name - this will give us callbacks that we can
    use to update the yew provider

*/
pub fn update_callbacks(
    struct_field_types: Vec<Type>,
    enum_variants: Vec<Ident>,
    cast_types: Vec<TokenStream2>,
    struct_field_names: Vec<Ident>,
) -> Vec<TokenStream2> {
    let mut callbacks = Vec::new();

    let type_name = struct_field_names.iter().zip(struct_field_types);

    for (((struct_field_name, struct_field_type), enum_variant), cast_type) in
        type_name.zip(enum_variants).zip(cast_types)
    {
        //        let callback = if struct_field_type.eq(&Type::Path(TypePath {
        //            qself: None,
        //            path: syn::Path::from(Ident::new("bool", Span::call_site())),
        //        })) {

        let callback = if &struct_field_name.to_string() == "remote" {
            // basically need to toggle the value for the boolean
            // not sure how exactly I'm going to do that here...

            // TODO: make this work with the bool type which would use input.checked
            quote! {
            |e: yew::events::Event| -> Self::Message {
                let input = e.target_dyn_into::<#cast_type>().unwrap();
                // let parse_type: #struct_field_type = input.value().parse().unwrap();
                let checked = input.checked();
                // let checkbox_value = if checked {
                //     // input.value() <- use this eventually
                //     "TRUE".to_string()
                // } else {
                //     "FALSE".to_string()
                // };
                // log::debug!( " checked -> {}", checked );
                Self::Message::#enum_variant(checked)
                };
            }
        } else {
            quote! {
            |e: yew::events::Event| -> Self::Message {
                let input = e.target_dyn_into::<#cast_type>().unwrap();
                let parse_type: #struct_field_type = input.value().parse().unwrap();
                Self::Message::#enum_variant(parse_type)
                };
            }
        };
        callbacks.push(callback);
    }
    callbacks
}
