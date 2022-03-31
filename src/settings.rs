use crate::simulator::simulations::accelerometer;
use crate::simulator::{
    simulations::{
        self, default_channel, default_feature, default_value_property, sawtooth, sine, wave,
        SimulationFactory, SingleTarget,
    },
    Claim,
};
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
    time::Duration,
};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, EnumString};

pub const DEFAULT_CONFIG_KEY: &str = "drogue.io/device-simulator/defaultConfiguration";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub auto_connect: bool,
    pub target: Target,
    pub application: String,
    pub device: String,

    #[serde(default)]
    pub simulations: BTreeMap<String, Simulation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumDiscriminants)]
#[serde(rename_all = "camelCase")]
#[strum_discriminants(derive(strum::Display, EnumMessage, EnumIter, EnumString))]
pub enum Simulation {
    #[strum_discriminants(strum(message = "Simple sine wave generator",))]
    Sine(Box<simulations::sine::Properties>),
    #[strum_discriminants(strum(message = "Sawtooth generator",))]
    Sawtooth(Box<simulations::sawtooth::Properties>),
    #[strum_discriminants(strum(message = "Wave generator",))]
    Wave(Box<simulations::wave::Properties>),
    #[strum_discriminants(strum(message = "Accelerometer",))]
    Accelerometer(Box<simulations::accelerometer::Properties>),
}

impl Simulation {
    pub fn to_json(&self) -> Value {
        serde_json::to_value(&self).unwrap_or_default()
    }

    pub fn to_claims(&self) -> Vec<Claim> {
        self.create().claims().to_vec()
    }
}

impl Default for Simulation {
    fn default() -> Self {
        SimulationDiscriminants::Sine.make_default()
    }
}

const fn default_period() -> Duration {
    Duration::from_secs(1)
}

impl SimulationDiscriminants {
    pub fn make_default(&self) -> Simulation {
        match self {
            Self::Sine => Simulation::Sine(Box::new(sine::Properties {
                amplitude: 1.0f64.into(),
                length: Duration::from_secs(60),
                period: default_period(),
                target: Default::default(),
            })),
            Self::Sawtooth => Simulation::Sawtooth(Box::new(sawtooth::Properties {
                max: 1.0f64.into(),
                length: Duration::from_secs(60),
                period: default_period(),
                target: Default::default(),
            })),
            Self::Wave => Simulation::Wave(Box::new(wave::Properties {
                lengths: vec![],
                amplitudes: vec![],
                offset: 0f64.into(),
                period: default_period(),
                target: Default::default(),
            })),
            Self::Accelerometer => Simulation::Accelerometer(Box::new(accelerometer::Properties {
                delay: default_period(),
                target: Default::default(),
            })),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_connect: false,
            target: Target::Mqtt {
                url: "wss://mqtt-endpoint-ws-browser-drogue-dev.apps.wonderful.iot-playground.org/mqtt".into(),
                credentials: Credentials::Password("my-password".into()),
            },
            application: "my-application".into(),
            device: "my-device".into(),
            simulations: {
                let mut s = BTreeMap::new();
                s.insert("sine1".to_string(), Simulation::Sine(Box::new(sine::Properties{
                    amplitude: 100f64.into(),
                    length: Duration::from_secs(60),
                    period: Duration::from_secs(1),
                    target: SingleTarget{
                        channel: default_channel(),
                        feature: default_feature(),
                        property: default_value_property(),
                    }
                })));
                s
            }
        }
    }
}

impl Settings {
    pub fn load() -> Option<anyhow::Result<Self>> {
        let json: Option<String> = LocalStorage::get(DEFAULT_CONFIG_KEY).ok();
        json.map(|json| serde_json::from_str(&json).map_err(|err| anyhow::Error::new(err)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Target {
    Mqtt {
        url: String,
        credentials: Credentials,
    },
    Http {
        url: String,
        credentials: Credentials,
    },
}

impl Target {
    pub fn as_protocol(&self) -> Protocol {
        match self {
            Self::Mqtt { .. } => Protocol::Mqtt,
            Self::Http { .. } => Protocol::Http,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Credentials {
    None,
    Password(String),
    UsernamePassword { username: String, password: String },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, EnumString)]
pub enum Protocol {
    Http,
    Mqtt,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http => f.write_str("HTTP"),
            Self::Mqtt => f.write_str("MQTT"),
        }
    }
}
