use super::*;
use crate::{
    edit::Setter,
    simulator::simulations::{sawtooth, sine, wave},
};
use patternfly_yew::*;
use yew::prelude::*;

pub fn render_sawtooth_editor<S>(setter: &S, props: &sawtooth::Properties) -> Html
where
    S: Setter<sawtooth::Properties>,
{
    html!(<>
        <FormSection title="Parameters">
            { setter_field(setter, "Maximum", props.max.0, | state, v| state.max = v.into() )}
            { setter_field(setter, "Period", humantime::Duration::from(props.period), |state, v| state.period = v.into() )}
            { setter_field(setter, "Length", humantime::Duration::from(props.length), |state, v| state.length = v.into() )}
        </FormSection>
        { edit_target(&setter.map(|props|&mut props.target), &props.target) }
    </>)
}

pub fn render_sine_editor<S>(setter: &S, props: &sine::Properties) -> Html
where
    S: Setter<sine::Properties>,
{
    html!(<>
        <FormSection title="Parameters">
            { setter_field(setter, "Amplitude", props.amplitude.0, | state, v| state.amplitude = v.into() )}
            { setter_field(setter,"Period", humantime::Duration::from(props.period),  |state, v| state.period = v.into() ) }
            { setter_field(setter, "Length", humantime::Duration::from(props.length), |state, v| state.length = v.into() ) }
        </FormSection>

        { edit_target(&setter.map(|props|&mut props.target), &props.target) }
    </>)
}

pub fn render_wave_editor<S>(setter: &S, props: &wave::Properties) -> Html
where
    S: Setter<wave::Properties>,
{
    html!(<>
        <FormSection title="Parameters">
            { setter_field(setter, "Offset", props.offset.0, | state, v| state.offset = v.into() ) }
            { setter_field(setter, "Period", humantime::Duration::from(props.period), |state, v| state.period = v.into()) }
            { setter_field(setter, "Amplitudes", props.amplitudes.clone(), |state, v| state.amplitudes = v) }
            { setter_field(setter, "Lengths", props.lengths.clone(),  |state, v| state.lengths = v) }
        </FormSection>
        { edit_target(&setter.map(|props|&mut props.target), &props.target) }
    </>)
}
