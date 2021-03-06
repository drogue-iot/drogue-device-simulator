use super::default_period;
use crate::{
    simulator::{
        publish::PublisherExt,
        simulations::{
            tick::{TickState, TickedGenerator},
            Context, SimulationState, SingleTarget,
        },
        Claim,
    },
    utils::{
        float::{ApproxF64, Zero},
        ui::details,
    },
};
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Properties {
    pub max: ApproxF64<Zero, 2>,

    #[serde(with = "humantime_serde")]
    pub length: Duration,

    #[serde(default = "default_period", with = "humantime_serde")]
    pub period: Duration,

    #[serde(default)]
    pub target: SingleTarget,
}

pub struct State {
    pub max: f64,
    pub length: f64,
    pub period: Duration,
    pub target: SingleTarget,
}

impl TickState for State {
    fn period(&self) -> Duration {
        self.period
    }
}

pub struct SawtoothGenerator;

impl TickedGenerator for SawtoothGenerator {
    type Properties = Properties;
    type State = State;

    fn make_state(
        properties: &Self::Properties,
        _current_state: Option<Self::State>,
    ) -> Self::State {
        Self::State {
            max: properties.max.0,
            length: properties.length.as_millis().to_f64().unwrap_or(f64::MAX),
            period: properties.period,
            target: properties.target.clone(),
        }
    }

    fn make_claims(properties: &Self::Properties) -> Vec<Claim> {
        properties.target.claims()
    }

    fn tick(now: f64, state: &mut Self::State, ctx: &mut Context) {
        let value = (now * 1000.0 / state.length) % state.max;

        ctx.update(SimulationState {
            description: state.target.describe("Sawtooth"),
            html: details([("Timestamp", now), ("Value", value)]),
        });

        ctx.publisher().publish_single(
            &state.target.channel,
            &state.target.feature,
            &state.target.property,
            value,
        );
    }
}
