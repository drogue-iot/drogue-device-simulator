use super::default_period;
use crate::simulator::generators::tick::{TickState, TickedGenerator};
use crate::simulator::generators::{Context, SimulationState, SingleTarget};
use crate::simulator::publish::PublisherExt;
use crate::utils::float::{ApproxF64, Zero};
use js_sys::Date;
use js_sys::Math::sin;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::f64::consts::TAU;
use std::time::{Duration, SystemTime};
use yew::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Properties {
    pub lengths: Vec<ApproxF64<Zero, 2>>,
    pub amplitudes: Vec<ApproxF64<Zero, 2>>,

    pub offset: ApproxF64<Zero, 2>,

    #[serde(default = "default_period", with = "humantime_serde")]
    pub period: Duration,

    #[serde(default)]
    pub target: SingleTarget,
}

pub struct State {
    pub offset: f64,
    pub parameters: Vec<[f64; 2]>,
    pub period: Duration,
    pub target: SingleTarget,
}

impl TickState for State {
    fn period(&self) -> Duration {
        self.period
    }
}

pub struct WaveGenerator;

const DEFAULT_FEATURE: &str = "wave";

impl TickedGenerator for WaveGenerator {
    type Properties = Properties;
    type State = State;

    fn make_state(
        properties: &Self::Properties,
        _current_state: Option<Self::State>,
    ) -> Self::State {
        Self::State {
            parameters: properties
                .lengths
                .iter()
                .map(|v| v.0)
                .zip(properties.amplitudes.iter().map(|v| v.0))
                .map(|(l, a)| [l, a])
                .collect(),
            offset: properties.offset.0,
            period: properties.period,
            target: properties.target.clone(),
        }
    }

    fn tick(now: f64, state: &mut Self::State, ctx: &mut Context) {
        let mut value = state.offset;

        for [l, a] in &state.parameters {
            value += sin(now * (TAU / l)) * a;
        }

        ctx.update(SimulationState {
            description: state.target.describe("Wave", DEFAULT_FEATURE),
            html: html!(
                <>
                    <dl>
                        <dt>{ "Timestamp: "}</dt><dd> { now } </dd>
                        <dt>{ "Value: "}</dt><dd> { value } </dd>
                    </dl>
                </>
            ),
        });

        ctx.publisher().publish_single(
            &state.target.channel,
            state.target.feature.as_deref().unwrap_or(DEFAULT_FEATURE),
            &state.target.property,
            value,
        );
    }
}
