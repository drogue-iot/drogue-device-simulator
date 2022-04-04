use crate::simulator::simulations::{Sender, SenderConfiguration, SenderHandle};
use crate::{
    simulator::{
        publish::PublisherExt,
        simulations::{default_period, Context, Generator, SimulationState, SingleTarget},
        Claim,
    },
    utils::{
        float::{ApproxF64, Zero},
        ui::details,
    },
};
use patternfly_yew::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use wasm_bindgen_futures::spawn_local;
use yew::html::IntoPropValue;
use yew::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Step {
    Value(ApproxF64<Zero, 2>),
    WithLabel {
        value: ApproxF64<Zero, 2>,
        label: String,
    },
}

impl IntoPropValue<patternfly_yew::Step> for &Step {
    fn into_prop_value(self) -> patternfly_yew::Step {
        patternfly_yew::Step {
            value: self.value(),
            label: self.label(),
        }
    }
}

impl Step {
    pub fn value(&self) -> f64 {
        match self {
            Self::Value(v) => v.0,
            Self::WithLabel { value, .. } => value.0,
        }
    }

    pub fn set_value(&mut self, new_value: f64) {
        match self {
            Self::Value(v) => v.0 = new_value,
            Self::WithLabel { value, .. } => value.0 = new_value,
        }
    }

    pub fn label(&self) -> Option<String> {
        match self {
            Self::Value(_) => None,
            Self::WithLabel { label, .. } => Some(label.to_string()),
        }
    }

    pub fn set_label(&mut self, new_label: Option<String>) {
        *self = match new_label {
            None => Self::Value(self.value().into()),
            Some(new_label) => Self::WithLabel {
                value: self.value().into(),
                label: new_label,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Properties {
    #[serde(default = "default_period")]
    #[serde(with = "humantime_serde")]
    pub delay: Duration,
    #[serde(default)]
    pub target: SingleTarget,

    pub min: Step,
    pub max: Step,
}

impl SenderConfiguration for Properties {
    fn delay(&self) -> Duration {
        self.delay
    }
}

pub struct SliderSimulation {
    claims: Vec<Claim>,
    properties: Properties,

    sender: Option<SenderHandle<f64, Properties>>,
}

impl Generator for SliderSimulation {
    type Properties = Properties;

    fn new(properties: Self::Properties) -> Self {
        let claims = properties.target.claims();
        Self {
            claims,
            properties,
            sender: None,
        }
    }

    fn claims(&self) -> &[Claim] {
        &self.claims
    }

    fn update(&mut self, properties: Self::Properties) {
        self.claims = properties.target.claims();
        if self.properties != properties {
            self.properties = properties.clone();
            if let Some(sender) = &mut self.sender {
                let mut sender = sender.clone();
                spawn_local(async move {
                    if let Err(err) = sender.configure(properties).await {
                        log::warn!("Failed to update configuration: {err}");
                    }
                })
            }
        }
    }

    fn start(&mut self, ctx: Context) {
        let (handle, sender) = Sender::new(
            ctx.clone(),
            self.properties.clone(),
            0f64,
            |handle, ctx, config, state| {
                ctx.publisher().publish_single(
                    &config.target.channel,
                    &config.target.feature,
                    &config.target.property,
                    *state,
                );

                let handle = handle.to_sync();
                let onchange = Callback::from(move |value: f64| handle.update(value));

                ctx.update(SimulationState {
                    description: config.target.describe("Slider"),
                    html: html!(
                        <>
                            <div>
                                { details([(
                                    "Value", state,
                                )]) }
                            </div>
                            <Slider
                                min={&config.min}
                                max={&config.max}
                                {onchange}
                                />
                        </>
                    ),
                });
            },
        );

        sender.start();

        self.sender = Some(handle);
    }

    fn stop(&mut self) {
        self.sender = None;
    }
}
