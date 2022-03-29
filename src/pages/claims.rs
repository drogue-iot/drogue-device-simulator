use crate::{
    app::AppRoute,
    pages::ApplicationPage,
    simulator::{Claim, Claims, SimulatorBridge, SimulatorState},
};
use multimap::MultiMap;
use patternfly_yew::*;
use yew::prelude::*;
use yew_router::components::RouterAnchor;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    channel: String,
    feature: Option<String>,
    property: Option<String>,
    simulators: Vec<String>,
}

impl TableRenderer for Entry {
    fn render(&self, column: ColumnIndex) -> Html {
        match column.index {
            0 => html!(
                <>
                if self.simulators.len() > 1 {
                    <span class="pf-u-mx-sm" style="color: var(--pf-global--warning-color--100);">{ Icon::ExclamationTriangle }</span>
                }
                { self.channel.clone() }
                </>
            ),
            1 => html!({ self.feature.as_deref().unwrap_or("*") }),
            2 => html!({ self.property.as_deref().unwrap_or("*") }),
            3 => html!(
                <ul>
                { for self.simulators.iter()
                    .map(|s|html!(
                        <li>
                            <RouterAnchor<AppRoute>
                                route={AppRoute::Simulation(s.clone())}
                                >{ s }
                            </RouterAnchor<AppRoute>>
                        </li>
                    ))
                }
                </ul>
            ),
            _ => html!(),
        }
    }
}

pub struct InternalClaims {
    claims: SharedTableModel<Entry>,
    _simulator: SimulatorBridge,
}

pub enum Msg {
    State(SimulatorState),
}

impl ApplicationPage for InternalClaims {
    fn title() -> String {
        "Claims".into()
    }
}

impl Component for InternalClaims {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let simulator = SimulatorBridge::from(ctx.link(), Msg::State);
        Self {
            claims: Default::default(),
            _simulator: simulator,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(state) => {
                let claims = Self::make_entries(state.claims);
                self.claims.clear();
                for claim in claims {
                    self.claims.push(claim);
                }
            }
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let header = html_nested! {
            <TableHeader>
                <TableColumn label="Channel"/>
                <TableColumn label="Feature"/>
                <TableColumn label="Property"/>
                <TableColumn label="Simulators"/>
            </TableHeader>
        };

        html!(
            <>
                <PageSection variant={PageSectionVariant::Light}>
                    <Content>
                        { "This page shows which channels, features, or properties are claimed by simulator modules. Showing possible overlaps in naming." }
                    </Content>
                </PageSection>
                <PageSection variant={PageSectionVariant::Light} fill={true}>
                    <Table<SharedTableModel<Entry>>
                        entries={self.claims.clone()}
                        mode={TableMode::Compact}
                        header={header}
                        >
                    </Table<SharedTableModel<Entry>>>
                </PageSection>
            </>
        )
    }
}

impl InternalClaims {
    fn make_entries(claims: Claims) -> Vec<Entry> {
        // step one, record what gets used/touched

        let mut touch = MultiMap::<Claim, String>::new();

        for (id, claims) in claims.iter() {
            for claim in claims {
                match claim {
                    Claim::Channel { channel } => {
                        touch.insert(
                            Claim::Channel {
                                channel: channel.clone(),
                            },
                            id.clone(),
                        );
                    }
                    Claim::Feature { channel, feature } => {
                        touch.insert(
                            Claim::Channel {
                                channel: channel.clone(),
                            },
                            id.clone(),
                        );
                        touch.insert(
                            Claim::Feature {
                                channel: channel.clone(),
                                feature: feature.clone(),
                            },
                            id.clone(),
                        );
                    }
                    Claim::Property {
                        channel,
                        feature,
                        property,
                    } => {
                        touch.insert(
                            Claim::Channel {
                                channel: channel.clone(),
                            },
                            id.clone(),
                        );
                        touch.insert(
                            Claim::Feature {
                                channel: channel.clone(),
                                feature: feature.clone(),
                            },
                            id.clone(),
                        );
                        touch.insert(
                            Claim::Property {
                                channel: channel.clone(),
                                feature: feature.clone(),
                                property: property.clone(),
                            },
                            id.clone(),
                        );
                    }
                }
            }
        }

        // step two, check if what is claims touches some other namespace

        let mut result = Vec::new();

        for (_, claims) in claims.iter() {
            for claim in claims {
                let sims = touch.get_vec(claim).cloned().unwrap_or_default();
                match claim {
                    Claim::Channel { channel } => {
                        result.push(Entry {
                            channel: channel.clone(),
                            feature: None,
                            property: None,
                            simulators: sims.to_vec(),
                        });
                    }
                    Claim::Feature { channel, feature } => {
                        result.push(Entry {
                            channel: channel.clone(),
                            feature: Some(feature.clone()),
                            property: None,
                            simulators: sims.to_vec(),
                        });
                    }
                    Claim::Property {
                        channel,
                        feature,
                        property,
                    } => {
                        result.push(Entry {
                            channel: channel.clone(),
                            feature: Some(feature.clone()),
                            property: Some(property.clone()),
                            simulators: sims.to_vec(),
                        });
                    }
                }
            }
        }

        // return result

        result
    }
}
