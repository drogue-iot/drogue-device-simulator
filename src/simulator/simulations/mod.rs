pub mod accelerometer;
pub mod sawtooth;
pub mod sine;
pub mod tick;
pub mod wave;

mod context;

pub use context::*;

use crate::{
    settings::Simulation,
    simulator::{simulations::tick::TickedGenerator, Claim},
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

    #[serde(default = "default_feature")]
    pub feature: String,

    #[serde(default = "default_value_property")]
    pub property: String,
}

impl Default for SingleTarget {
    fn default() -> Self {
        Self {
            channel: default_channel(),
            feature: default_feature(),
            property: default_value_property(),
        }
    }
}

impl SingleTarget {
    #[allow(unused)]
    pub fn new<C, F, P>(channel: C, feature: F, property: P) -> Self
    where
        C: Into<String>,
        F: Into<String>,
        P: Into<String>,
    {
        Self {
            channel: channel.into(),
            feature: feature.into(),
            property: property.into(),
        }
    }

    pub fn describe(&self, label: &str) -> SimulationDescription {
        SimulationDescription {
            label: format!(
                "{} ({}/{}/{})",
                label, self.channel, self.feature, self.property,
            ),
        }
    }

    pub fn claims(&self) -> Vec<Claim> {
        vec![Claim::Property {
            channel: self.channel.clone(),
            feature: self.feature.clone(),
            property: self.property.clone(),
        }]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeatureTarget {
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(default = "default_feature")]
    pub feature: String,
}

impl Default for FeatureTarget {
    fn default() -> Self {
        Self {
            channel: default_channel(),
            feature: default_feature(),
        }
    }
}

impl FeatureTarget {
    #[allow(unused)]
    pub fn new<C, F>(channel: C, feature: F) -> Self
    where
        C: Into<String>,
        F: Into<String>,
    {
        Self {
            channel: channel.into(),
            feature: feature.into(),
        }
    }

    pub fn describe(&self, label: &str) -> SimulationDescription {
        SimulationDescription {
            label: format!("{} ({}/{})", label, self.channel, self.feature,),
        }
    }

    pub fn claims(&self) -> Vec<Claim> {
        vec![Claim::Feature {
            channel: self.channel.clone(),
            feature: self.feature.clone(),
        }]
    }
}

pub fn default_channel() -> String {
    "state".into()
}

pub fn default_feature() -> String {
    "feature".into()
}

pub fn default_value_property() -> String {
    "value".into()
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
            Simulation::Accelerometer(props) => Box::new(
                accelerometer::AccelerometerSimulation::new(props.as_ref().clone()),
            ),
        }
    }
}
