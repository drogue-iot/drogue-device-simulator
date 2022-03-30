use crate::{
    app::AppRoute,
    data::{SharedDataDispatcher, SharedDataOps},
    edit::*,
    pages::{ApplicationPage, SimulationDetails},
    settings::{Settings, Simulation},
    simulator::{
        generators::{sawtooth, sine, wave, SimulationFactory},
        Claim, SimulatorBridge, SimulatorState,
    },
    utils::to_yaml,
};
use itertools::Itertools;
use patternfly_yew::*;
use serde_json::{json, Value};
use std::{collections::HashSet, rc::Rc, time::Duration};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};
use uuid::Uuid;
use yew::prelude::*;
use yew_router::{
    agent::{RouteAgentDispatcher, RouteRequest},
    prelude::Route,
};

#[derive(Clone, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, EnumMessage, EnumIter))]
pub enum SimulationTypes {
    #[strum_discriminants(strum(message = "Wave generator",))]
    Wave(Box<wave::Properties>),
    #[strum_discriminants(strum(message = "Sawtooth generator",))]
    Sawtooth(Box<sawtooth::Properties>),
    #[strum_discriminants(strum(message = "Simple sine wave generator",))]
    Sine(Box<sine::Properties>),
}

impl SimulationTypes {
    pub fn to_json(&self) -> Value {
        match self {
            Self::Wave(props) => serde_json::to_value(props.as_ref()),
            Self::Sawtooth(props) => serde_json::to_value(props.as_ref()),
            Self::Sine(props) => serde_json::to_value(props.as_ref()),
        }
        .unwrap_or_default()
    }

    pub fn to_simulation(&self) -> Simulation {
        match self {
            Self::Sine(props) => Simulation::Sine(props.as_ref().clone()),
            Self::Sawtooth(props) => Simulation::Sawtooth(props.as_ref().clone()),
            Self::Wave(props) => Simulation::Wave(props.as_ref().clone()),
        }
    }

    pub fn to_claims(&self) -> Vec<Claim> {
        self.to_simulation().create().claims().to_vec()
    }
}

const fn default_period() -> Duration {
    Duration::from_secs(1)
}

impl SimulationTypesDiscriminants {
    pub fn make_default(&self) -> SimulationTypes {
        match self {
            Self::Sine => SimulationTypes::Sine(Box::new(sine::Properties {
                amplitude: 1.0f64.into(),
                length: Duration::from_secs(60),
                period: default_period(),
                target: Default::default(),
            })),
            Self::Sawtooth => SimulationTypes::Sawtooth(Box::new(sawtooth::Properties {
                max: 1.0f64.into(),
                length: Duration::from_secs(60),
                period: default_period(),
                target: Default::default(),
            })),
            Self::Wave => SimulationTypes::Wave(Box::new(wave::Properties {
                lengths: vec![],
                amplitudes: vec![],
                offset: 0f64.into(),
                period: default_period(),
                target: Default::default(),
            })),
        }
    }
}

impl Default for SimulationTypes {
    fn default() -> Self {
        SimulationTypesDiscriminants::Sine.make_default()
    }
}

pub enum Msg {
    SimulatorState(SimulatorState),
    Selected(SimulationTypesDiscriminants),
    SetId(String),
    Add,

    Set(Box<dyn FnOnce(&mut SimulationTypes)>),
    ValidationState(InputState),
}

pub struct Add {
    id: String,
    content: SimulationTypes,

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
        let cfg = self.content.to_simulation();

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

        let current: SimulationTypesDiscriminants = self.content.clone().into();

        html!(
            <FormGroup
                label={"Type"}
                required=true
                >

                <FormSelect<SimulationTypesDiscriminants>
                    variant={variant}
                >
                    { for SimulationTypesDiscriminants::iter()
                        .sorted_by(|a,b|Ord::cmp(&a.to_string(), &b.to_string()))
                        .map(|t| {

                        let selected = current == t;

                        html_nested!(
                            <FormSelectOption<SimulationTypesDiscriminants>
                                value={t}
                                selected={selected}
                                description={t.get_message()}
                                id={t.to_string()}
                            />
                        )}
                    )}
                </FormSelect<SimulationTypesDiscriminants>>

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
            SimulationTypes::Sawtooth(props) => render_sawtooth_editor(
                &setter.map_or(|state| match state {
                    SimulationTypes::Sawtooth(props) => Some(props.as_mut()),
                    _ => None,
                }),
                props,
            ),
            SimulationTypes::Sine(props) => render_sine_editor(
                &setter.map_or(|state| match state {
                    SimulationTypes::Sine(props) => Some(props.as_mut()),
                    _ => None,
                }),
                props,
            ),
            SimulationTypes::Wave(props) => render_wave_editor(
                &setter.map_or(|state| match state {
                    SimulationTypes::Wave(props) => Some(props.as_mut()),
                    _ => None,
                }),
                props,
            ),
        }
    }

    fn validate(&mut self) {
        let claims = self.content.to_claims();
        self.validation_result = if self.simulator_state.claims.is_claimed_any(&claims) {
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
