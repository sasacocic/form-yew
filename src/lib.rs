use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DataStruct, DeriveInput, Field, FieldsNamed};
use syn::{Data, Fields};
mod helpers;
use helpers::parse_struct_body;



/*
 TODOs:
 - Refactor this code so it has more structure and it's more organized. It's really 
 hard to work with right now. The mental load is a lot, and I feel like I have to
 re-learn it each time I come back to it.
 - error handling. If something is done wrong the error handling should
 actually tell the user, and give a decent error message for a way to fix
 it.
 - String + Bool support 
 */




#[proc_macro_derive(YewForm, attributes(ele))]
pub fn yew_form_derive(token_stream: TokenStream) -> TokenStream {
    /*
    <input .. /> types this macro doesn't handle
    - checkbox (would map nice to bool)
    - file - Custom type I export?
    - email - Custom type I export?
    - number
     */

    
    let der_input = parse_macro_input!(token_stream as DeriveInput);

    let struct_name = der_input.ident;

    // Message enum for the provider
    let enum_name = format_ident!("Update{}", struct_name);
    // Providers name
    let provider_name = format_ident!("{}Provider", struct_name);
    // (looks like) Props for provider 
    let props_name = format_ident!("{}Props", struct_name);
    /*

    messages_enum - associate type Message in a Component - in this case for the Provider
    but this is the TokenStream to generate that code.
    struct_field_names - the names of the fields in a struct
    html_tag_type - html element corresponding to the types of the struct fields
    cast_type - yew specific - when reading an input you need to cast it to a specific type e.g.
    HtmlInputElement (i.e. <input ..../>) or HtmlTexrtAreaElement (i.e. <textarea ... />)


     */
    let (messages_enum,
        struct_field_names,
        (html_tag_type, cast_type),
        struct_field_types) = parse_struct_body(der_input.data, &enum_name);
    let struct_field_names_for_update = struct_field_names
        .iter()
        .map(|v| format_ident!("update_{}", v))
        .collect::<Vec<proc_macro2::Ident>>();
    
    // NOTE: This is only here to get rid of compiler warnings for non-capitalized variant names
    // also, need to come up with a better way to do this than what I'm doing right now
    let enum_variants = struct_field_names.iter().map(|v| format_ident!("{}", v.to_string().to_uppercase())).collect::<Vec<proc_macro2::Ident>>();
    // no in provider yet.
    let field_validation_errors = struct_field_names.iter().map(|v| format_ident!("{}_error", v.to_string())).collect::<Vec<proc_macro2::Ident>>();

    // add field_validation_errors to the struct_field_names

    let function_component_function_name = struct_field_names.iter().map(|v| format_ident!("{}_fn",v.to_string().to_uppercase())).collect::<Vec<proc_macro2::Ident>>();
    let function_component_names = struct_field_names.iter().map(|v| {
        let chars = v.to_string().chars().enumerate().map(|(ind, chr)|{
            if ind == 0 {
                chr.to_ascii_uppercase()
            } else {
                chr
            }
        }).collect::<String>();
        format_ident!("{}", chars)
    }).collect::<Vec<proc_macro2::Ident>>();
    let html_tag_type = html_tag_type.iter().map(|v| format_ident!("{}", v)).collect::<Vec<proc_macro2::Ident>>();

    let update_funcs = struct_field_names.iter().map(|v| format_ident!("update_{}", v)).collect::<Vec<proc_macro2::Ident>>();

    let struct_field_names_as_strings = struct_field_names.iter().map(|v| v.to_string()).collect::<Vec<String>>();

    let function_component_props = function_component_names.iter().map(|v| format_ident!("{}_Props",v)).collect::<Vec<proc_macro2::Ident>>();

    // eprintln!("html tag {:#?} - rust type {:#?}", html_tag_type, cast_type);
    

    let generated_html = helpers::gen_html_inputs(html_tag_type.clone(), struct_field_names_as_strings.clone(), struct_field_types.clone());

    let generated_callbacks = helpers::gen_update_callbacks(struct_field_types.clone(), enum_variants.clone(), cast_type.clone(), field_validation_errors.clone());

    let final_output = quote! {

        /*
        this generates an enum that looks like approx. like this
        pub enum Update#NameOfStructThisIsDerivedFrom {
            struct_field_name_1(struct_field_1_type),
            struct_field_name_2(struct_field_2_type),
            ...
        }
         */
        #messages_enum


        #[derive(PartialEq, Properties)]
        pub struct #props_name {
            pub children: yew::Children
        }


        /*  looks like
        pub enum Update#original_struct_name {
            #struct_field_one(#struct_field_type_one),
            #struct_field_two(#struct_field_type_two),
            ...
        }

         */
        #[derive(Clone, PartialEq, Debug)]
        pub struct #provider_name {
            pub form: #struct_name,
            #( pub #struct_field_names_for_update : yew::Callback<yew::html::onchange::Event>,)*
            // #( pub #field_validation_errors: std::option::Option<std::string::String>, )*
        }

        impl Component for #provider_name {
            type Message = #enum_name;
            type Properties = #props_name;

            fn create(ctx: &Context<Self>) -> Self {

                // #(
                //     let #struct_field_names_for_update = |e: yew::events::Event| -> Self::Message {
                //     let input = e.target_dyn_into::<#cast_type>().unwrap();
                //     let parse_type: #struct_field_types = input.value().parse().unwrap();
                //     Self::Message::#enum_variants(parse_type)
                //     }; 
                //     let #struct_field_names_for_update = ctx.link().callback(#struct_field_names_for_update);
                // )*


                #(
                    let #struct_field_names_for_update = #generated_callbacks; 
                    let #struct_field_names_for_update = ctx.link().callback(#struct_field_names_for_update);
                )*




                Self {
                    form: #struct_name {
                        #( #struct_field_names: Default::default(), )*
                    },
                    // #( #field_validation_errors: None, )*
                    #( #struct_field_names_for_update, )*
                }
            }

            fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
                match msg {
                     #(Self::Message::#enum_variants(inside_value) => {
                         self.form.#struct_field_names = inside_value;
                     }
                    ),*
                }
                true
            }

            fn view(&self, ctx: &Context<Self>) -> Html {
                html! {
                    <ContextProvider<#provider_name> context={self.clone()}>
                        {ctx.props().children.clone()}
                    </ContextProvider<#provider_name>>
                }
            }
        }


        #(#[derive(Properties, PartialEq)]
        pub struct #function_component_props {
            pub class: std::option::Option<String>
        }
        )*

        #(#[function_component(#function_component_names)]
        pub fn #function_component_function_name(p: &#function_component_props) -> yew::Html {

            let #provider_name {
                form: #struct_name {
                    #struct_field_names: struct_field_name,
                    ..
                },
                #update_funcs: update_func,
                // #field_validation_errors: validation_error,
                ..
            } = use_context::<#provider_name>().expect("context for this field");



            /*
            - if this thing is a boolean then I need to have a
            <#html_tag_type type="checkbox" class={&p.class} onchange={update_func} value={struct_field_name} placeholder={#struct_field_names_as_strings} /> 
             */

             html! {
                <>
                #generated_html
                // if validation_error.is_some() {
                //     <i>{"validation error: x is bad"}</i>
                // }
                </>
             }
            //html! {
                //// <#html_tag_type class={&p.class} onchange={update_func} value={struct_field_name.to_string()} placeholder={#struct_field_names_as_strings} /> 
            //
         })*
        



    }
    .into();

    // eprintln!("output tokens: {}",final_output);

    final_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(true);
    }
}
