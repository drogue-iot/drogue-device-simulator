use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub const DEFAULT_CONFIG_KEY: &str = "drogue.io/device-simulator/defaultConfiguration";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub auto_connect: bool,
    pub target: Target,
    pub application: String,
    pub device: String,
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
        }
    }
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let json: String = LocalStorage::get(DEFAULT_CONFIG_KEY)?;
        Ok(serde_json::from_str(&json)?)
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
