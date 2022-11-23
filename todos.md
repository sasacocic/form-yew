# TODO

- make checkboxes work with both `String` and `bool` types

# Do Eventually

- checkbox functionality
  - allow using the string type on a checkbox and pass the value
    inside `<input type="checkbox" value="THIS_VALUE" .... />`as the
    value the will be passed to the string if the checkbox is checked
    and empty string otherwise.

# think about

- figure out a way to easily test this macro
  - make this a workspace so I can test the macro easily
- from the angel list takehome how would I handle being able to dynamically add a field to the this macro?
  Like if you have `Vec` or something? How would it be handled to handle the updating of that nested struct field?

- tailwindui componenets - how will this integrate with tailwindcss and the components there - basically thinking how this
  macro will work with other inputs

- form validation for this macro
  - is this something this crate should handle? I was thinking it should only handle binding
    between inputs and Rust structs. I can take two approaches
    1. Fall-over if something is entered in correctly?
    2. Actually handle validation

## exercises

1. think of some ui / forms that could exist then see what it would take to get that binding logic done in form-yew
