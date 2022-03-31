pub mod sawtooth;
pub mod sine;
pub mod tick;
pub mod wave;

use crate::{
    settings::Simulation,
    simulator::{
        publish::{Publisher, SimulatorStateUpdate},
        simulations::tick::TickedGenerator,
        Claim,
    },
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use yew::Html;

const fn default_period() -> Duration {
    Duration::from_secs(1)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SingleTarget {
    #[serde(default = "default_channel")]
    pub channel: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature: Option<String>,

    #[serde(default = "default_value_property")]
    pub property: String,
}

impl Default for SingleTarget {
    fn default() -> Self {
        Self {
            channel: default_channel(),
            feature: None,
            property: default_value_property(),
        }
    }
}

impl SingleTarget {
    pub fn describe(&self, label: &str, default_feature: &str) -> SimulationDescription {
        SimulationDescription {
            label: format!(
                "{} ({}/{})",
                label,
                self.channel,
                self.feature.as_deref().unwrap_or(default_feature)
            ),
        }
    }

    pub fn claims(&self, default_feature: &str) -> Vec<Claim> {
        vec![Claim::Property {
            channel: self.channel.clone(),
            feature: self
                .feature
                .as_deref()
                .unwrap_or(default_feature)
                .to_string(),
            property: self.property.clone(),
        }]
    }
}

fn default_channel() -> String {
    "state".into()
}

fn default_value_property() -> String {
    "value".into()
}

pub struct Context {
    pub id: String,
    publisher: Box<dyn Publisher>,
    updater: Box<dyn SimulatorStateUpdate>,
}

impl Context {
    pub fn new<P, U>(id: String, publisher: P, updater: U) -> Self
    where
        P: Publisher + 'static,
        U: SimulatorStateUpdate + 'static,
    {
        Self {
            id,
            publisher: Box::new(publisher),
            updater: Box::new(updater),
        }
    }

    pub fn publisher(&mut self) -> &mut dyn Publisher {
        self.publisher.as_mut()
    }

    pub fn update(&mut self, state: SimulationState) {
        self.updater.state(state)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SimulationDescription {
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SimulationState {
    pub description: SimulationDescription,
    pub html: Html,
}

pub trait Generator {
    type Properties;

    fn new(properties: Self::Properties) -> Self;
    /// get current claims
    fn claims(&self) -> &[Claim];

    fn update(&mut self, properties: Self::Properties);

    fn start(&mut self, ctx: Context);
    fn stop(&mut self);
}

pub trait SimulationHandler {
    fn start(&mut self, ctx: Context);
    fn stop(&mut self);
    fn claims(&self) -> &[Claim];
}

impl<G> SimulationHandler for G
where
    G: Generator,
{
    fn start(&mut self, ctx: Context) {
        Generator::start(self, ctx)
    }

    fn stop(&mut self) {
        Generator::stop(self)
    }

    fn claims(&self) -> &[Claim] {
        Generator::claims(self)
    }
}

pub trait SimulationFactory {
    fn create(&self) -> Box<dyn SimulationHandler>;
}

impl SimulationFactory for Simulation {
    fn create(&self) -> Box<dyn SimulationHandler> {
        match self {
            Simulation::Sine(props) => Box::new(sine::SineGenerator::new(props.as_ref().clone())),
            Simulation::Sawtooth(props) => {
                Box::new(sawtooth::SawtoothGenerator::new(props.as_ref().clone()))
            }
            Simulation::Wave(props) => Box::new(wave::WaveGenerator::new(props.as_ref().clone())),
        }
    }
}
