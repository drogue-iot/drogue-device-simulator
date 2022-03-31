use crate::simulator::{
    publish::PublisherExt,
    simulations::{default_period, Context, FeatureTarget, Generator, SimulationState},
    Claim,
};
use crate::utils::ui::details;
use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::{select, FutureExt, SinkExt, StreamExt};
use gloo_timers::future::TimeoutFuture;
use gloo_utils::window;
use js_sys::Date;
use num_traits::ToPrimitive;
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
    pub delay: Duration,
    #[serde(default)]
    pub target: FeatureTarget,
}

pub struct AccelerometerSimulation {
    claims: Vec<Claim>,
    properties: Properties,
    ctx: Option<Context>,

    sensor: Option<Sensor>,
    tx: Option<UnboundedSender<Msg>>,
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
    pub fn new(tx: UnboundedSender<Msg>) -> Self {
        let listener = Closure::wrap(Box::new(move |e: DeviceOrientationEvent| {
            let state = State::from(e);
            let mut tx = tx.clone();
            spawn_local(async move {
                tx.send(Msg::Update(state)).await.ok();
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

pub enum Msg {
    Update(State),
    Configure(Properties),
}

struct Sender {
    rx: UnboundedReceiver<Msg>,
    ctx: Context,
    target: FeatureTarget,
    delay: Duration,
}

impl Sender {
    pub fn new(
        rx: UnboundedReceiver<Msg>,
        ctx: Context,
        target: FeatureTarget,
        delay: Duration,
    ) -> Self {
        Self {
            rx,
            ctx,
            target,
            delay,
        }
    }

    pub async fn run(mut self) {
        let mut target = self.target.clone();
        let mut delay = self.delay.as_millis().to_f64().unwrap_or(f64::MAX);

        // internally this is a i32, so infinity is i32::MAX, but as u32
        const INFINITY: u32 = i32::MAX as u32;

        let mut state = State::None;
        let mut next = Date::now();
        let mut timer = TimeoutFuture::new(INFINITY).fuse();

        loop {
            select! {
                msg = self.rx.next() => match msg {
                    Some(Msg::Update(s)) => {
                        state = s;
                        let now = Date::now();
                        let rem = next - now;
                        if rem < 0f64 {
                            self.send(&target, &state);
                            next = now + delay;
                        }  else {
                            timer = TimeoutFuture::new(rem.to_u32().unwrap_or(INFINITY)).fuse();
                        }
                    }
                    Some(Msg::Configure(properties)) => {
                        target = properties.target;
                        delay = properties.delay.as_millis().to_f64().unwrap_or(f64::MAX);
                    }
                    None => {
                        self.rx.close();
                        break;
                    }
                },
                () = timer => {
                    self.send(&target, &state);
                    timer = TimeoutFuture::new(INFINITY).fuse();
                }
            }
        }
    }

    fn send(&self, target: &FeatureTarget, state: &State) {
        let properties = state.to_properties();
        if let Some(properties) = &properties {
            self.ctx.publisher().publish_feature(
                &target.channel,
                &target.feature,
                properties.clone(),
            );
        }
        self.ctx.update(SimulationState {
            description: target.describe("Accelerometer"),
            html: properties
                .map(|s| details(s))
                .unwrap_or_else(|| default_details()),
        });
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
                    if let Err(err) = tx.send(Msg::Configure(properties)).await {
                        log::warn!("Failed to update configuration: {err}");
                    }
                })
            }
        }
    }

    fn start(&mut self, ctx: Context) {
        let (tx, rx) = mpsc::unbounded::<Msg>();

        let sensor = Sensor::new(tx.into());

        let sender = Sender::new(
            rx,
            ctx.clone(),
            self.properties.target.clone(),
            self.properties.delay,
        );
        spawn_local(async move { sender.run().await });

        self.sensor = Some(sensor);
        self.ctx = Some(ctx);
    }

    fn stop(&mut self) {
        self.sensor.take();
        if let Some(tx) = self.tx.take() {
            tx.close_channel();
        }
    }
}

fn default_details() -> Html {
    html!(
        <Content>
            { "No accelerometer data received from browser. This might be due to fact that your browser does not have access to an accelerometer. Most desktop browsers don't have one." }
        </Content>
    )
}
