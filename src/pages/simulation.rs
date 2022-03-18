use crate::simulator::generators::SimulationState;
use crate::simulator::{Response, SimulatorBridge};
use patternfly_yew::*;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Properties {
    pub id: String,
}

pub struct Simulation {
    _simulator: SimulatorBridge,

    state: SimulationState,
}

pub enum Msg {
    State(SimulationState),
}

impl Component for Simulation {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let mut simulator =
            SimulatorBridge::new(ctx.link().batch_callback(|response| match response {
                Response::SimulationState(state) => vec![Msg::State(state)],
                _ => vec![],
            }));

        simulator.subscribe_simulation(ctx.props().id.clone());

        Self {
            state: SimulationState { label: "".into() },
            _simulator: simulator,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(state) => {
                self.state = state;
            }
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html!(
            <>
                <PageSection>
                    <Title level={Level::H1} size={Size::XXXXLarge}>{ "Simulation" }
                        <small>
                            { format!(" – {}", self.state.label) }
                        </small>
                    </Title>
                </PageSection>
            </>
        )
    }
}
