use crate::simulator::{
    publish::PublisherExt,
    simulations::{Context, FeatureTarget, Generator, SimulationState},
    Claim,
};
use crate::utils::ui::details;
use gloo_utils::window;
use patternfly_yew::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{rc::Rc, sync::RwLock};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::DeviceOrientationEvent;
use yew::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Properties {
    #[serde(default)]
    pub target: FeatureTarget,
}

pub struct AccelerometerSimulation {
    claims: Vec<Claim>,
    properties: Properties,
    ctx: Option<Context>,

    sensor: Option<Sensor>,
}

struct Sensor {
    state: Rc<RwLock<Option<Html>>>,
    listener: Closure<dyn Fn(DeviceOrientationEvent)>,
}

impl Sensor {
    pub fn new(ctx: Context, target: FeatureTarget) -> Self {
        let state = Rc::new(RwLock::new(None));
        let callback_state = state.clone();

        let listener = Closure::wrap(Box::new(move |e: DeviceOrientationEvent| {
            log::debug!("Device event: {e:?}");

            let details = if let Some(state) = Self::make_state(&e) {
                ctx.publisher().publish_feature(
                    target.channel.clone(),
                    target.feature.clone(),
                    state,
                );

                Some(details(state))
            } else {
                None
            };

            if let Ok(mut lock) = callback_state.write() {
                *lock = details.clone();
            }
            AccelerometerSimulation::update_state(&ctx, details, &target);
        }) as Box<dyn Fn(DeviceOrientationEvent)>);

        if let Some(cb) = listener.as_ref().dyn_ref() {
            window()
                .add_event_listener_with_callback("deviceorientation", cb)
                .ok();
        }
        Self { listener, state }
    }

    fn make_state(e: &DeviceOrientationEvent) -> Option<[(&str, f64); 3]> {
        match (e.alpha(), e.beta(), e.gamma()) {
            (Some(alpha), Some(beta), Some(gamma)) => {
                Some([("alpha", alpha), ("beta", beta), ("gamma", gamma)])
            }
            _ => None,
        }
    }

    pub fn details(&self) -> Option<Html> {
        if let Ok(state) = self.state.read() {
            state.clone()
        } else {
            None
        }
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
            ctx: None,
            sensor: None,
        }
    }

    fn claims(&self) -> &[Claim] {
        &self.claims
    }

    fn update(&mut self, properties: Self::Properties) {
        self.claims = properties.target.claims();
        if self.properties != properties {
            self.properties = properties;
            if let Some(ctx) = &self.ctx {
                self.sensor = Some(Sensor::new(ctx.clone(), self.properties.target.clone()));
            }
        }

        self.send_state();
    }

    fn start(&mut self, ctx: Context) {
        self.sensor = Some(Sensor::new(ctx.clone(), self.properties.target.clone()));
        self.ctx = Some(ctx);
        self.send_state();
    }

    fn stop(&mut self) {
        self.sensor.take();
        self.ctx = None;
    }
}

impl AccelerometerSimulation {
    fn send_state(&mut self) {
        if let Some(ctx) = &mut self.ctx {
            Self::update_state(
                ctx,
                self.sensor.as_ref().and_then(Sensor::details),
                &self.properties.target,
            );
        }
    }

    fn update_state(ctx: &Context, details: Option<Html>, target: &FeatureTarget) {
        ctx.update(SimulationState {
            description: target.describe("Accelerometer"),
            html: details.unwrap_or_else(|| default_details()),
        });
    }
}

fn default_details() -> Html {
    html!(
        <Content>
            { "No accelerometer data received from browser. This migth be due to fact that your browser does not have access to an accelerometer. Most desktop browsers don't have one." }
        </Content>
    )
}
