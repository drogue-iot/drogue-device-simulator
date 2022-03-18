pub mod sawtooth;
pub mod sine;
pub mod tick;

use crate::simulator::publish::Publisher;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

fn default_channel() -> String {
    "state".into()
}

fn default_value_property() -> String {
    "value".into()
}

pub struct Context {
    pub id: String,
    publisher: Box<dyn Publisher>,
}

impl Context {
    pub fn new<P: Publisher + 'static>(id: String, publisher: P) -> Self {
        Self {
            id,
            publisher: Box::new(publisher),
        }
    }

    pub fn publisher(&mut self) -> &mut dyn Publisher {
        self.publisher.as_mut()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulationState {
    pub label: String,
}

pub trait Generator {
    type Properties;

    fn new(properties: Self::Properties) -> Self;
    fn update(&mut self, properties: Self::Properties);

    fn start(&mut self, ctx: Context);
    fn stop(&mut self);

    fn state(&self) -> SimulationState;
}
