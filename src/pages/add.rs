use crate::pages::ApplicationPage;
use crate::simulator::generators::{sawtooth, sine, wave};
use crate::simulator::{SimulatorBridge, SimulatorState};
use itertools::Itertools;
use patternfly_yew::*;
use std::collections::HashSet;
use std::rc::Rc;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};
use yew::prelude::*;

#[derive(Clone, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(strum::Display, EnumMessage, EnumIter))]
pub enum SimulationTypes {
    #[strum_discriminants(strum(message = "Wave generator",))]
    Wave(wave::Properties),
    #[strum_discriminants(strum(message = "Sawtooth generator",))]
    Sawtooth(sawtooth::Properties),
    #[strum_discriminants(strum(message = "Simple sine wave generator",))]
    Sine(sine::Properties),
}

impl SimulationTypesDiscriminants {
    pub fn make_default(&self) -> SimulationTypes {
        match self {
            Self::Sine => SimulationTypes::Sine(sine::Properties {
                amplitude: 1.0f64.into(),
                length: Default::default(),
                period: Default::default(),
                target: Default::default(),
            }),
            Self::Sawtooth => SimulationTypes::Sawtooth(sawtooth::Properties {
                max: 1.0f64.into(),
                length: Default::default(),
                period: Default::default(),
                target: Default::default(),
            }),
            Self::Wave => SimulationTypes::Wave(wave::Properties {
                lengths: vec![],
                amplitudes: vec![],
                offset: 0f64.into(),
                period: Default::default(),
                target: Default::default(),
            }),
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
}

pub struct Add {
    content: SimulationTypes,
    simulator_state: SimulatorState,
    _simulator: SimulatorBridge,
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
        Self {
            content: Default::default(),
            simulator_state: Default::default(),
            _simulator: simulator,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Selected(sel) => {
                self.content = sel.make_default();
            }
            Msg::SimulatorState(simulator_state) => {
                self.simulator_state = simulator_state;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ids: HashSet<_> = self.simulator_state.simulations.keys().cloned().collect();
        let is_unique = Validator::Custom(Rc::new(move |value| {
            if value.is_empty() {
                return ValidationResult::error("Must not be empty");
            }

            match ids.contains(value) {
                true => ValidationResult::error("Value is already in use"),
                false => Default::default(),
            }
        }));

        html!(
            <PageSection variant={PageSectionVariant::Light} fill={true}>
                <Form limit_width=true>

                    <FormGroupValidated<TextInput>
                        required=true
                        label={"ID"}
                        validator={is_unique}
                        >
                        <TextInput placeholder="Unique ID for the simulation" />
                    </FormGroupValidated<TextInput>>

                    { self.render_type(ctx) }
                </Form>
            </PageSection>
        )
    }
}

impl Add {
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
}
