mod editors;
mod fields;
mod setter;
mod target;

pub use editors::*;
pub use fields::*;
pub use setter::*;
pub use target::*;

use patternfly_yew::*;
use yew::prelude::*;

pub fn edit_field<F>(label: &str, value: F, setter: Callback<F>) -> Html
where
    F: FieldType + 'static,
{
    let setter = Callback::from(move |s: String| match F::parse(&s) {
        Ok(value) => setter.emit(value),
        Err(_) => {}
    });

    html!(
        <FormGroupValidated<TextInput>
            required={F::required()}
            validator={F::base_validator().unwrap_or_default()}
            label={label.to_string()}
            >
            <TextInput
                value={value.to_string()}
                onchange={setter}
                />
        </FormGroupValidated<TextInput>>
    )
}

pub fn setter_field<STATE, S, T, F>(setter: &S, label: &str, value: T, f: F) -> Html
where
    T: FieldType + 'static,
    F: FnOnce(&mut STATE, T) + 'static,
    S: Setter<STATE>,
{
    edit_field(label, value, setter.setter(f))
}
