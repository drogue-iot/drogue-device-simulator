use crate::{
    app::AppRoute,
    data::{SharedDataBridge, SharedDataOps},
    edit::*,
    settings::{self, Settings},
    simulator::{
        simulations::{SimulationFactory, SimulationState},
        Response, SimulatorBridge, SimulatorState,
    },
};
use patternfly_yew::*;
use yew::prelude::*;
use yew_router::{agent::RouteRequest, prelude::*};

#[derive(Switch, Debug, Clone, PartialEq, Eq)]
pub enum SimulationDetails {
    #[to = "configuration"]
    Configuration,
    #[end]
    Overview,
}

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Properties {
    pub details: SimulationDetails,
    pub id: String,
}

pub struct Simulation {
    simulator: SimulatorBridge,

    simulation_id: String,
    state: SimulationState,
    simulator_state: SimulatorState,

    settings_agent: SharedDataBridge<Settings>,
    settings: Settings,

    validation_result: Option<FormAlert>,
    validation_state: InputState,
}

pub enum Msg {
    State(SimulationState),
    SimulatorState(SimulatorState),
    Settings(Settings),
    Set(Box<dyn FnOnce(&mut settings::Simulation)>),
    ValidationState(InputState),
    Apply,
    Delete,
}

impl Component for Simulation {
    type Message = Msg;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let mut simulator =
            SimulatorBridge::new(ctx.link().batch_callback(|response| match response {
                Response::State(state) => vec![Msg::SimulatorState(state)],
                Response::SimulationState(state) => vec![Msg::State(state)],
                _ => vec![],
            }));

        simulator.subscribe_simulation(ctx.props().id.clone());

        let mut settings_agent = SharedDataBridge::from(ctx.link(), Msg::Settings);
        settings_agent.request_state();

        Self {
            state: SimulationState::default(),
            simulator_state: Default::default(),
            simulation_id: ctx.props().id.clone(),
            simulator,
            settings_agent,
            settings: Default::default(),
            validation_result: None,
            validation_state: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(state) => {
                self.state = state;
            }
            Msg::SimulatorState(state) => {
                self.simulator_state = state;
                self.validate();
            }
            Msg::Settings(settings) => {
                self.settings = settings;
                self.validate();
            }
            Msg::Set(setter) => {
                let sim = self.settings.simulations.get_mut(&self.simulation_id);
                if let Some(sim) = sim {
                    setter(sim);
                }
                self.validate();
            }
            Msg::ValidationState(state) => {
                self.validation_state = state;
            }
            Msg::Apply => {
                if let Some(sim) = self.settings.simulations.get(&self.simulation_id) {
                    let id = self.simulation_id.clone();
                    let sim = sim.clone();
                    self.settings_agent.update(move |settings| {
                        settings.simulations.insert(id, sim);
                    });
                }
            }
            Msg::Delete => {
                let id = self.simulation_id.clone();
                self.settings_agent.update(move |settings| {
                    settings.simulations.remove(&id);
                });

                let route = Route::<()>::from(AppRoute::Overview);
                RouteAgentDispatcher::new().send(RouteRequest::ChangeRoute(route));
            }
        }
        true
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        if self.simulation_id != ctx.props().id {
            self.simulator
                .unsubscribe_simulation(self.simulation_id.clone());
            self.simulation_id = ctx.props().id.clone();
            self.simulator
                .subscribe_simulation(self.simulation_id.clone());
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let id = ctx.props().id.clone();
        let transformer = SwitchTransformer::new(
            |global| match global {
                AppRoute::Simulation { details, .. } => Some(details),
                _ => None,
            },
            move |local| AppRoute::Simulation {
                id: id.clone(),
                details: local,
            },
        );

        html!(
            <>
                <PageSection variant={PageSectionVariant::Light}>
                    <Title level={Level::H1} size={Size::XXXXLarge}>{ "Simulation" }
                        <small>
                            { format!(" – {}", self.state.description.label) }
                        </small>
                    </Title>
                </PageSection>

                <PageTabs limit_width=true>
                    <TabsRouter<AppRoute, SimulationDetails>
                        transformer={transformer}
                        inset={Inset::Page}
                        >
                        <TabRouterItem<SimulationDetails> to={SimulationDetails::Overview} label="Overview"/>
                        <TabRouterItem<SimulationDetails> to={SimulationDetails::Configuration} label="Configuration"/>
                    </TabsRouter<AppRoute, SimulationDetails>>
                </PageTabs>
                <PageSection variant={PageSectionVariant::Light} limit_width=true>
                {
                    match ctx.props().details {
                        SimulationDetails::Overview => html!(
                            { self.state.html.clone() }
                        ),
                        SimulationDetails::Configuration => html!(
                            { self.render_editor(ctx) }
                        ),
                    }
                }
                </PageSection>
            </>
        )
    }
}

impl Simulation {
    fn render_editor(&self, ctx: &Context<Self>) -> Html {
        let setter = ContextSetter::from((ctx, Msg::Set));

        html!(
            <Form
                alert={self.validation_result.clone()}
                horizontal={[FormHorizontal.xl()]}
                onvalidated={ctx.link().callback(Msg::ValidationState)}
                >
                if let Some(sim) = self.settings.simulations.get(&self.simulation_id) {
                    {match sim {
                        settings::Simulation::Sawtooth(props) => render_sawtooth_editor(
                            &setter.map_or(|state| match state {
                                settings::Simulation::Sawtooth(props) => Some(props.as_mut()),
                                _ => None,
                            }),
                            props,
                        ),
                        settings::Simulation::Sine(props) => render_sine_editor(
                            &setter.map_or(|state| match state {
                                settings::Simulation::Sine(props) => Some(props.as_mut()),
                                _ => None,
                            }),
                            props,
                        ),
                        settings::Simulation::Wave(props) => render_wave_editor(
                            &setter.map_or(|state| match state {
                                settings::Simulation::Wave(props) => Some(props.as_mut()),
                                _ => None,
                            }),
                            props,
                        ),
                    }}
                }

                <ActionGroup>
                    <Button
                        id="add"
                        label="Apply"
                        variant={Variant::Primary}
                        onclick={ctx.link().callback(|_|Msg::Apply)}
                        disabled={self.is_disabled()}
                        />
                    <Button
                        id="delete"
                        label="Delete"
                        variant={Variant::DangerSecondary}
                        onclick={ctx.link().callback(|_|Msg::Delete)}
                        />
                </ActionGroup>

            </Form>
        )
    }

    fn validate(&mut self) {
        let claims = self
            .settings
            .simulations
            .get(&self.simulation_id)
            .map(|sim| sim.create().claims().to_vec())
            .unwrap_or_default();

        self.validation_result = if self
            .simulator_state
            .claims
            .is_claimed_any(&claims, Some(&self.simulation_id))
        {
            Some(FormAlert {
                r#type: Type::Warning,
                title: "Conflicting claims".into(),
                children: html!({
                    "The simulation will conflict with targets of existing simulations."
                }),
            })
        } else {
            None
        };
    }

    fn is_disabled(&self) -> bool {
        matches!(self.validation_state, InputState::Error)
            || matches!(
                self.validation_result,
                Some(FormAlert {
                    r#type: Type::Danger,
                    ..
                })
            )
    }
}
