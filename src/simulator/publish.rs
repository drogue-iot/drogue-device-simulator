use crate::{settings::PayloadFormat, simulator::simulations::SimulationState};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The device state.
///
/// ```json
/// {
///   "features": {
///     "temperature": {
///       "value": 1.23
///     }
///   }
/// }
/// ```
#[derive(Clone, Debug, Serialize)]
pub struct ChannelState {
    pub features: BTreeMap<String, Feature>,
}

impl ChannelState {
    pub fn to_payload(&self, format: PayloadFormat) -> anyhow::Result<Vec<u8>> {
        match format {
            PayloadFormat::JsonCompact => {
                let features: BTreeMap<String, BTreeMap<String, Value>> = self
                    .features
                    .iter()
                    .map(|(k, v)| (k.clone(), v.properties.clone()))
                    .collect();

                let value = json!({ "features": features });
                Ok(serde_json::to_vec(&value)?)
            }
            PayloadFormat::Doppelgaenger => {
                let features: BTreeMap<String, BTreeMap<String, Value>> = self
                    .features
                    .iter()
                    .map(|(k, v)| (k.clone(), v.properties.clone()))
                    .collect();

                Ok(serde_json::to_vec(&json!({
                    "partial": false,
                    "state": features
                }))?)
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Feature {
    pub properties: BTreeMap<String, Value>,
}

#[derive(Debug)]
pub struct SingleFeature {
    pub name: String,
    pub state: Feature,
}

pub trait SimulatorStateUpdate {
    fn state(&self, state: SimulationState);
}

pub trait Publisher {
    fn publish(&self, event: PublishEvent);
}

pub trait PublisherExt {
    fn publish_feature<C, F, I, P, V>(self, channel: C, feature: F, properties: I)
    where
        C: Into<String>,
        F: Into<String>,
        P: Into<String>,
        V: Into<Value>,
        I: IntoIterator<Item = (P, V)>;

    fn publish_single<C, F, P, V>(self, channel: C, feature: F, property: P, value: V)
    where
        C: Into<String>,
        F: Into<String>,
        P: Into<String>,
        V: Into<Value>;
}

impl PublisherExt for &dyn Publisher {
    fn publish_feature<C, F, I, P, V>(self, channel: C, feature: F, properties: I)
    where
        C: Into<String>,
        F: Into<String>,
        P: Into<String>,
        V: Into<Value>,
        I: IntoIterator<Item = (P, V)>,
    {
        self.publish(PublishEvent::Single {
            channel: channel.into(),
            state: SingleFeature {
                name: feature.into(),
                state: Feature {
                    properties: {
                        let mut p = BTreeMap::new();
                        for (k, v) in properties {
                            p.insert(k.into(), v.into());
                        }
                        p
                    },
                },
            },
        })
    }

    fn publish_single<C, F, P, V>(self, channel: C, feature: F, property: P, value: V)
    where
        C: Into<String>,
        F: Into<String>,
        P: Into<String>,
        V: Into<Value>,
    {
        self.publish(PublishEvent::Single {
            channel: channel.into(),
            state: SingleFeature {
                name: feature.into(),
                state: Feature {
                    properties: {
                        let mut p = BTreeMap::new();
                        p.insert(property.into(), value.into());
                        p
                    },
                },
            },
        })
    }
}

#[derive(Debug)]
pub enum PublishEvent {
    Single {
        channel: String,
        state: SingleFeature,
    },
    Full {
        channel: String,
        state: ChannelState,
    },
}
