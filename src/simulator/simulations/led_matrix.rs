use crate::simulator::publish::PublisherExt;
use crate::simulator::{
    simulations::{Context, Generator, SimulationState, SingleTarget},
    Claim, Command,
};
use patternfly_yew::{Card, Flex, FlexItem};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use yew::{function_component, html};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Properties {
    #[serde(default)]
    pub target: SingleTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_off: Option<String>,
}

pub struct LedMatrixSimulation {
    claims: Vec<Claim>,
    properties: Properties,

    state: State,
    context: Option<Context>,
}

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum State {
    On(u8),
    Off,
}

impl LedMatrixSimulation {
    fn publish(&self, ctx: &Context) {
        ctx.publisher().publish_single(
            &self.properties.target.channel,
            &self.properties.target.feature,
            &self.properties.target.property,
            json!({
                "on": !matches!(self.state, State::Off),
                "brightness": match self.state {
                    State::On(brightness) => brightness,
                    State::Off => 0,
                }
            }),
        );
    }

    fn notify(&self, ctx: &Context) {
        self.publish(&ctx);
        ctx.update(SimulationState {
            description: self.properties.target.describe("Led Matrix"),
            html: html! {
                <LedMatrixComponent
                    color={self.properties.color.clone().unwrap_or_default()}
                    color_off={self.properties.color_off.clone().unwrap_or_default()}
                    state={self.state}
                />
            },
        });
    }
}

impl Generator for LedMatrixSimulation {
    type Properties = Properties;

    fn new(properties: Self::Properties) -> Self {
        let claims = properties.target.claims();
        Self {
            claims,
            properties,
            state: State::Off,
            context: None,
        }
    }

    fn claims(&self) -> &[Claim] {
        &self.claims
    }

    fn update(&mut self, properties: Self::Properties) {
        self.claims = properties.target.claims();
        if self.properties != properties {
            self.properties = properties.clone();
        }
    }

    fn start(&mut self, ctx: Context) {
        self.notify(&ctx);
        self.context = Some(ctx);
    }

    fn stop(&mut self) {
        self.context = None;
    }

    fn command(&mut self, command: &Command) {
        if command.name != self.properties.target.channel {
            return;
        }

        if let Some(payload) = &command.payload {
            if let Ok(json) = serde_json::from_slice::<Value>(&payload) {
                match &json[&self.properties.target.feature][&self.properties.target.property] {
                    Value::Bool(true) => {
                        self.state = State::On(255);
                    }
                    Value::Bool(false) => {
                        self.state = State::Off;
                    }
                    Value::Number(brightness) => {
                        if let Some(b) = brightness.as_f64() {
                            self.state = State::On(b.clamp(0.0, 255.0) as u8);
                        } else if let Some(b) = brightness.as_i64() {
                            self.state = State::On(b.clamp(0, 255) as u8);
                        } else if let Some(b) = brightness.as_u64() {
                            self.state = State::On(b.clamp(0, 255) as u8);
                        }
                    }
                    _ => {}
                }
            }
        }
        if let Some(ctx) = &self.context {
            self.notify(ctx);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, yew::Properties)]
pub struct LedMatrixProperties {
    pub state: State,
    pub color: String,
    pub color_off: String,
}

#[function_component(LedMatrixComponent)]
fn render(props: &LedMatrixProperties) -> Html {
    const STYLE: &str =
        "height: 25px; width: 25px; border-radius: 50%; margin: 5px; display: inline-block;";

    let style = match props.state {
        State::On(brightness) => {
            let color = if props.color.is_empty() {
                "red"
            } else {
                &props.color
            };
            let opacity = brightness as f32 / 255.0;
            format!("{STYLE} opacity: {opacity}; background-color: {color};")
        }
        State::Off => {
            let color = if props.color_off.is_empty() {
                "lightgray"
            } else {
                &props.color_off
            };
            format!("{STYLE} background-color: {color};")
        }
    };

    html!(
        <>
        <Flex>
        <FlexItem>
        <Card>
        <div>
                <span id="0x0" style={style.clone()} />
                <span id="0x1" style={style.clone()} />
                <span id="0x2" style={style.clone()} />
                <span id="0x3" style={style.clone()} />
                <span id="0x4" style={style.clone()} />
            <br />
                <span id="1x0" style={style.clone()} />
                <span id="1x1" style={style.clone()} />
                <span id="1x2" style={style.clone()} />
                <span id="1x3" style={style.clone()} />
                <span id="1x4" style={style.clone()} />
            <br />
                <span id="2x0" style={style.clone()} />
                <span id="2x1" style={style.clone()} />
                <span id="2x2" style={style.clone()} />
                <span id="2x3" style={style.clone()} />
                <span id="2x4" style={style.clone()} />
            <br />
                <span id="3x0" style={style.clone()} />
                <span id="3x1" style={style.clone()} />
                <span id="3x2" style={style.clone()} />
                <span id="3x3" style={style.clone()} />
                <span id="3x4" style={style.clone()} />
            <br />
                <span id="4x0" style={style.clone()} />
                <span id="4x1" style={style.clone()} />
                <span id="4x2" style={style.clone()} />
                <span id="4x3" style={style.clone()} />
                <span id="4x4" style={style.clone()} />
            <br />
        </div>
        </Card>
        </FlexItem>
        </Flex>
        </>
    )
}
