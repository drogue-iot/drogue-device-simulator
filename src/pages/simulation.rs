use crate::simulator::{generators::SimulationState, Response, SimulatorBridge};
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
            state: SimulationState::default(),
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
                            { format!(" – {}", self.state.description.label) }
                        </small>
                    </Title>
                </PageSection>
                <PageSection variant={PageSectionVariant::Light} fill={true}>
                    { self.state.html.clone() }
                </PageSection>
            </>
        )
    }
}
