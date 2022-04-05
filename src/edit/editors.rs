use super::*;
use crate::{
    edit::Setter,
    settings::Simulation,
    simulator::simulations::{accelerometer, sawtooth, sine, slider, wave},
};
use humantime_serde::Serde;
use patternfly_yew::*;
use std::{ops::Deref, time::Duration};
use yew::prelude::*;

/// Render the editor for a simulation.
pub fn render_editor<S>(simulation: &Simulation, setter: S) -> Html
where
    S: Setter<Simulation>,
{
    match simulation {
        Simulation::Sawtooth(props) => render_sawtooth_editor(
            &setter.map_or(|state| match state {
                Simulation::Sawtooth(props) => Some(props.as_mut()),
                _ => None,
            }),
            props,
        ),
        Simulation::Sine(props) => render_sine_editor(
            &setter.map_or(|state| match state {
                Simulation::Sine(props) => Some(props.as_mut()),
                _ => None,
            }),
            props,
        ),
        Simulation::Wave(props) => render_wave_editor(
            &setter.map_or(|state| match state {
                Simulation::Wave(props) => Some(props.as_mut()),
                _ => None,
            }),
            props,
        ),
        Simulation::Accelerometer(props) => render_accelerometer_editor(
            &setter.map_or(|state| match state {
                Simulation::Accelerometer(props) => Some(props.as_mut()),
                _ => None,
            }),
            props,
        ),
        Simulation::Slider(props) => render_slider_editor(
            &setter.map_or(|state| match state {
                Simulation::Slider(props) => Some(props.as_mut()),
                _ => None,
            }),
            props,
        ),
    }
}

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
        { edit_single_target(&setter.map(|props|&mut props.target), &props.target) }
    </>)
}

pub fn render_sine_editor<S>(setter: &S, props: &sine::Properties) -> Html
where
    S: Setter<sine::Properties>,
{
    html!(<>
        <FormSection title="Parameters">
            { setter_field(setter, "Amplitude", props.amplitude.0, | state, v| state.amplitude = v.into() )}
            { setter_field(setter, "Period", humantime::Duration::from(props.period),  |state, v| state.period = v.into() ) }
            { setter_field(setter, "Length", humantime::Duration::from(props.length), |state, v| state.length = v.into() ) }
        </FormSection>

        { edit_single_target(&setter.map(|props|&mut props.target), &props.target) }
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
            { setter_field(
                setter, "Lengths",
                props
                    .lengths.iter()
                    .map(|l|humantime::Duration::from(l.deref().clone())).collect::<Vec<_>>(), 
                |state, v| state.lengths = v.into_iter().map(|l|{
                    let l: Duration = l.into();
                    Serde::from(l)
                }).collect()) }
        </FormSection>
        { edit_single_target(&setter.map(|props|&mut props.target), &props.target) }
    </>)
}

pub fn render_accelerometer_editor<S>(setter: &S, props: &accelerometer::Properties) -> Html
where
    S: Setter<accelerometer::Properties>,
{
    html!(<>
        <FormSection title="Parameters">
            { setter_field(setter, "Delay", humantime::Duration::from(props.delay), | state, v| state.delay = v.into() )}
        </FormSection>
        { edit_feature_target(&setter.map(|props|&mut props.target), &props.target) }
    </>)
}

pub fn render_slider_editor<S>(setter: &S, props: &slider::Properties) -> Html
where
    S: Setter<slider::Properties>,
{
    html!(<>
        <FormSection title="Parameters">
            { setter_field(setter, "Delay", humantime::Duration::from(props.delay), | state, v| state.delay = v.into() )}
            { setter_field(setter, "Min", props.min.value(), | state, v| state.min.set_value(v) )}
            { setter_field(setter, "Min (Label)", props.min.label(), | state, v| state.min.set_label(v) )}
            { setter_field(setter, "Max", props.max.value(), | state, v| state.max.set_value(v) )}
            { setter_field(setter, "Max (Label)", props.max.label(), | state, v| state.max.set_label(v) )}
        </FormSection>
        { edit_single_target(&setter.map(|props|&mut props.target), &props.target) }
    </>)
}
