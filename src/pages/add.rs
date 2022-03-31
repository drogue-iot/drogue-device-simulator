use crate::{
    app::AppRoute,
    data::{SharedDataDispatcher, SharedDataOps},
    edit::*,
    pages::{ApplicationPage, SimulationDetails},
    settings::{Settings, Simulation, SimulationDiscriminants},
    simulator::{SimulatorBridge, SimulatorState},
    utils::to_yaml,
};
use itertools::Itertools;
use patternfly_yew::*;
use serde_json::json;
use std::{collections::HashSet, rc::Rc};
use strum::{EnumMessage, IntoEnumIterator};
use uuid::Uuid;
use yew::prelude::*;
use yew_router::{
    agent::{RouteAgentDispatcher, RouteRequest},
    prelude::Route,
};

pub enum Msg {
    SimulatorState(SimulatorState),
    Selected(SimulationDiscriminants),
    SetId(String),
    Add,

    Set(Box<dyn FnOnce(&mut Simulation)>),
    ValidationState(InputState),
}

pub struct Add {
    id: String,
    content: Simulation,

    simulator_state: SimulatorState,
    _simulator: SimulatorBridge,
    settings_agent: SharedDataDispatcher<Settings>,

    validation_result: Option<FormAlert>,
    validation_state: InputState,
}

impl ApplicationPage for Add {
    fn title() -> String {
        "Add simulation".into()
    }
}

impl Component for Add {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let simulator = SimulatorBridge::from(ctx.link(), Msg::SimulatorState);
        let settings_agent = SharedDataDispatcher::<Settings>::new();

        Self {
            id: Uuid::new_v4().to_string(),
            content: Default::default(),
            simulator_state: Default::default(),
            _simulator: simulator,
            settings_agent,
            validation_result: Default::default(),
            validation_state: InputState::Default,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Selected(sel) => {
                self.content = sel.make_default();
                self.validate();
            }
            Msg::SimulatorState(simulator_state) => {
                self.simulator_state = simulator_state;
                self.validate();
            }
            Msg::SetId(id) => self.id = id,
            Msg::Set(setter) => {
                setter(&mut self.content);
                self.validate();
            }
            Msg::Add => {
                self.add();
            }
            Msg::ValidationState(state) => {
                self.validation_state = state;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ids: HashSet<_> = self.simulator_state.simulations.keys().cloned().collect();

        let is_unique = Validator::Custom(Rc::new(move |ctx: ValidationContext<String>| {
            if ctx.value.is_empty() {
                return ValidationResult::error("Must not be empty");
            }

            match ids.contains(&ctx.value) {
                true => ValidationResult::error("Value is already in use"),
                false => ValidationResult::help("Provide a unique ID"),
            }
        }));

        html!(
            <PageSection variant={PageSectionVariant::Light} fill={true}>

                <Grid gutter=true>
                    <GridItem cols={[6.lg(), 12.all()]} >

                        <Form
                            alert={self.validation_result.clone()}
                            horizontal={[FormHorizontal.xl()]}
                            onvalidated={ctx.link().callback(Msg::ValidationState)}
                        >

                            <FormGroupValidated<TextInput>
                                required=true
                                label={"ID"}
                                validator={is_unique}
                                >
                                <TextInput
                                    placeholder="Unique ID for the simulation"
                                    value={self.id.clone()}
                                    onchange={ctx.link().callback(Msg::SetId)}
                                />
                            </FormGroupValidated<TextInput>>

                            { self.render_type(ctx) }

                            { self.render_properties(ctx) }

                            <ActionGroup>
                                <Button
                                    id="add"
                                    label="Add"
                                    variant={Variant::Primary}
                                    onclick={ctx.link().callback(|_|Msg::Add)}
                                    disabled={self.is_disabled()}
                                    />
                            </ActionGroup>
                        </Form>
                    </GridItem>

                    <GridItem cols={[6.lg(), 12.all()]} >
                        <Clipboard
                            code=true readonly=true variant={ClipboardVariant::Expanded}
                            value={self.make_yaml()}/>
                    </GridItem>
                </Grid>

            </PageSection>
        )
    }
}

impl Add {
    fn add(&mut self) {
        let id = self.id.clone();
        let cfg = self.content.clone();

        self.settings_agent.update(|settings| {
            settings.simulations.insert(id, cfg);
        });

        let route = Route::<()>::from(AppRoute::Simulation {
            id: self.id.clone(),
            details: SimulationDetails::Configuration,
        });
        RouteAgentDispatcher::new().send(RouteRequest::ChangeRoute(route));
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

    /// Render the type select dropdown
    fn render_type(&self, ctx: &Context<Self>) -> Html {
        let variant = SelectVariant::Single(ctx.link().callback(|sel| Msg::Selected(sel)));

        let current: SimulationDiscriminants = self.content.clone().into();

        html!(
            <FormGroup
                label={"Type"}
                required=true
                >

                <FormSelect<SimulationDiscriminants>
                    variant={variant}
                >
                    { for SimulationDiscriminants::iter()
                        .sorted_by(|a,b|Ord::cmp(&a.to_string(), &b.to_string()))
                        .map(|t| {

                        let selected = current == t;

                        html_nested!(
                            <FormSelectOption<SimulationDiscriminants>
                                value={t}
                                selected={selected}
                                description={t.get_message()}
                                id={t.to_string()}
                            />
                        )}
                    )}
                </FormSelect<SimulationDiscriminants>>

            </FormGroup>
        )
    }

    /// Render the current state as YAML
    fn make_yaml(&self) -> String {
        to_yaml(&json!({
            &self.id: self.content.to_json()
        }))
    }

    /// Render the properties of the selected type
    fn render_properties(&self, ctx: &Context<Self>) -> Html {
        let setter = ContextSetter::from((ctx, Msg::Set));
        match &self.content {
            Simulation::Sawtooth(props) => render_sawtooth_editor(
                &setter.map_or(|state| match state {
                    Simulation::Sawtooth(props) => Some(props.as_mut()),
                    _ => None,
                }),
                props,
            ),
            Simulation::Sine(props) => render_sine_editor(
                &setter.map_or(|state| match state {
                    Simulation::Sine(props) => Some(props.as_mut()),
                    _ => None,
                }),
                props,
            ),
            Simulation::Wave(props) => render_wave_editor(
                &setter.map_or(|state| match state {
                    Simulation::Wave(props) => Some(props.as_mut()),
                    _ => None,
                }),
                props,
            ),
        }
    }

    fn validate(&mut self) {
        let claims = self.content.to_claims();
        self.validation_result = if self.simulator_state.claims.is_claimed_any(&claims, None) {
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
}
