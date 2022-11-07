# This is a work in progress

_WARNING_ this crate is badly documented and, becuase of that it makes it hard to use

TODOS

- Add documentation for doc.rs

# What is this

- this is a macro to help with forms in yew. How it works

```rust
use yew_form_derive::YewForm;

// TODO: idk if this is a minial example
// this create input elements that can be used in a from
// to use these elements use the context provided by this
// macro
#[derive(YewForm, PartialEq)]
pub struct AdvertiseForm {
    pub company: String,
    pub position: String,
    pub salary_range: String,
    pub link_to_apply: String, // better suited as a URL type
    #[ele = "textarea"]
    pub description: String,
}


// later on in another file or the same on

...

#[function_component(FormConsumer)]
pub fn consumer() -> Html {
    let form = use_context::<AdvertiseFormProvider>().expect("an advertiser form");

    let input_style = "text-xl sm:text-5xl bg-transparent border-black border p-2.5 rounded-lg";
    html! {
        <form>
            <Company class={input_style} />
            <Position class={input_style} />
            <Salary_range class={input_style} />
            <Description class={input_style}/>
            <Link_to_apply class={input_style}/>

            <button type="submit">
                {"Checkout ->"}
            </button>
        </form>
    }
}

#[function_component(AdvertisePage)]
pub fn advertise_page() -> Html {
    html! {
        <AdvertiseFormProvider>
            <FormConsumer />
        </AdvertiseFormProvider>
    }
}
```
