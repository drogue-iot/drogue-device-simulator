use crate::pages::ApplicationPage;
use crate::simulator::{Command, Request, Response, SimulatorBridge};
use patternfly_yew::*;
use serde_json::Value;
use std::rc::Rc;
use yew::prelude::*;

const DEFAULT_MAX_SIZE: usize = 200;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry(Command);

impl TableRenderer for Entry {
    fn render(&self, column: ColumnIndex) -> Html {
        match column.index {
            0 => html!(<code>{&self.0.name}</code>),
            1 => match &self.0.payload {
                Some(payload) => render_payload(payload, false),
                None => html!(),
            },
            _ => html!(),
        }
    }

    fn render_details(&self) -> Vec<Span> {
        match &self.0.payload {
            Some(payload) => {
                vec![Span::max(html!(render_payload(payload, true))).truncate()]
            }
            None => {
                vec![]
            }
        }
    }
}

fn render_payload(data: &[u8], expanded: bool) -> Html {
    if let Ok(json) = serde_json::from_slice::<Value>(data) {
        let json = match expanded {
            true => serde_json::to_string_pretty(&json).unwrap_or_default(),
            false => serde_json::to_string(&json).unwrap_or_default(),
        };
        return html!(
            <code><pre>
                {json}
            </pre></code>
        );
    }

    if let Ok(str) = String::from_utf8(data.to_vec()) {
        return html!(
            <pre>
                {str}
            </pre>
        );
    }

    html!("Binary data")
}

pub struct Commands {
    commands: SharedTableModel<Entry>,
    total_received: usize,
    _simulator: SimulatorBridge,
}

impl ApplicationPage for Commands {
    fn title() -> String {
        "Received commands".to_string()
    }
}

pub enum Msg {
    Add(Rc<Command>),
    Set(Vec<Command>),
    Clear,
}

impl Component for Commands {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut simulator =
            SimulatorBridge::new(ctx.link().batch_callback(|response| match response {
                Response::Command(command) => {
                    vec![Msg::Add(command)]
                }
                Response::CommandHistory(commands) => {
                    vec![Msg::Set(commands)]
                }
                _ => vec![],
            }));

        simulator.send(Request::FetchHistory);

        Self {
            commands: Default::default(),
            total_received: 0,
            _simulator: simulator,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Clear => {
                self.commands.clear();
            }
            Msg::Set(commands) => {
                for command in commands.into_iter().rev().take(DEFAULT_MAX_SIZE) {
                    self.commands.push(Entry(command));
                }
                self.total_received = self.commands.len();
            }
            Msg::Add(command) => {
                self.total_received += 1;
                self.commands.insert(0, Entry((*command).clone()));
                while self.commands.len() > DEFAULT_MAX_SIZE {
                    self.commands.pop();
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let header = html_nested! {
            <TableHeader>
                <TableColumn label="Command"/>
                <TableColumn label="Payload"/>
            </TableHeader>
        };

        html!(
            <>
                <PageSection variant={PageSectionVariant::Light} fill={true}>
                    <Toolbar>
                        <ToolbarGroup>

                            <ToolbarItem>
                                <Button
                                    label="Clear"
                                    icon={Icon::Times}
                                    variant={Variant::Secondary}
                                    onclick={ctx.link().callback(|_|Msg::Clear)}
                                    />
                            </ToolbarItem>
                        </ToolbarGroup>
                        <ToolbarItem modifiers={[ToolbarElementModifier::Right.all()]}>
                            <strong>{"Commands received: "}{self.total_received}</strong>
                        </ToolbarItem>
                    </Toolbar>

                    <Table<SharedTableModel<Entry>>
                        entries={self.commands.clone()}
                        mode={TableMode::CompactExpandable}
                        header={header}
                        >
                    </Table<SharedTableModel<Entry>>>

                </PageSection>
            </>
        )
    }
}
