use crate::simulator::{
    publish::PublisherExt,
    simulations::{
        default_period, Context, FeatureTarget, Generator, Sender, SenderConfiguration,
        SenderHandle, SimulationState,
    },
    Claim,
};
use crate::utils::ui::details;
use gloo_utils::{format::JsValueSerdeExt, window};
use patternfly_yew::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::DeviceOrientationEvent;
use yew::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Properties {
    #[serde(default = "default_period")]
    #[serde(with = "humantime_serde")]
    pub delay: Duration,
    #[serde(default)]
    pub target: FeatureTarget,
}

impl SenderConfiguration for Properties {
    fn delay(&self) -> Duration {
        self.delay
    }
}

pub struct AccelerometerSimulation {
    claims: Vec<Claim>,
    properties: Properties,

    sensor: Option<Sensor>,
    tx: Option<SenderHandle<State, Properties>>,
}

struct Sensor {
    listener: Closure<dyn FnMut(DeviceOrientationEvent)>,
}

#[derive(Clone, Debug)]
pub enum State {
    Orientation { x: f64, y: f64, z: f64 },
    None,
}

impl State {
    pub fn to_properties(&self) -> Option<Vec<(&str, f64)>> {
        match self {
            State::None => None,
            State::Orientation { x, y, z } => Some(vec![("x", *x), ("y", *y), ("z", *z)]),
        }
    }
}

impl From<DeviceOrientationEvent> for State {
    fn from(e: DeviceOrientationEvent) -> Self {
        match (e.alpha(), e.beta(), e.gamma()) {
            (Some(alpha), Some(beta), Some(gamma)) => State::Orientation {
                x: alpha,
                y: beta,
                z: gamma,
            },
            _ => Self::None,
        }
    }
}

impl Sensor {
    pub fn new(tx: SenderHandle<State, Properties>) -> Self {
        let listener = Closure::wrap(Box::new(move |e: DeviceOrientationEvent| {
            let state = State::from(e);
            let mut tx = tx.clone();
            spawn_local(async move {
                tx.update(state).await.ok();
            });
        }) as Box<dyn FnMut(DeviceOrientationEvent)>);

        if let Some(cb) = listener.as_ref().dyn_ref() {
            window()
                .add_event_listener_with_callback("deviceorientation", cb)
                .ok();
        }

        Self { listener }
    }
}

impl Drop for Sensor {
    fn drop(&mut self) {
        log::info!("Dropping sensor");
        if let Some(cb) = self.listener.as_ref().dyn_ref() {
            if let Err(err) = window().remove_event_listener_with_callback("deviceorientation", cb)
            {
                log::warn!("Failed to remove listener: {:?}", err.into_serde::<Value>());
            }
        }
    }
}

impl Generator for AccelerometerSimulation {
    type Properties = Properties;

    fn new(properties: Self::Properties) -> Self {
        let claims = properties.target.claims();
        Self {
            claims,
            properties,
            sensor: None,
            tx: None,
        }
    }

    fn claims(&self) -> &[Claim] {
        &self.claims
    }

    fn update(&mut self, properties: Self::Properties) {
        self.claims = properties.target.claims();
        if self.properties != properties {
            self.properties = properties.clone();
            if let Some(tx) = &mut self.tx {
                let mut tx = tx.clone();
                spawn_local(async move {
                    if let Err(err) = tx.configure(properties).await {
                        log::warn!("Failed to update configuration: {err}");
                    }
                })
            }
        }
    }

    fn start(&mut self, ctx: Context) {
        let (tx, sender) = Sender::new(
            ctx,
            self.properties.clone(),
            State::None,
            |_, ctx, config, state| {
                let properties = state.to_properties();
                if let Some(properties) = &properties {
                    ctx.publisher().publish_feature(
                        &config.target.channel,
                        &config.target.feature,
                        properties.clone(),
                    );
                }
                ctx.update(SimulationState {
                    description: config.target.describe("Accelerometer"),
                    html: properties
                        .map(|s| details(s))
                        .unwrap_or_else(|| default_details()),
                });
            },
        );

        sender.start();

        let sensor = Sensor::new(tx.clone().into());

        self.tx = Some(tx);
        self.sensor = Some(sensor);
    }

    fn stop(&mut self) {
        self.sensor = None;
        self.tx = None;
    }
}

fn default_details() -> Html {
    html!(
        <Content>
            { "No accelerometer data received from browser. This might be due to fact that your browser does not have access to an accelerometer. Most desktop browsers don't have one." }
        </Content>
    )
}
