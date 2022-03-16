mod mqtt;

use crate::connector::mqtt::QoS;
use crate::data::{self, SharedDataBridge};
use crate::settings::{Credentials, Settings, Target};
use crate::simulator::mqtt::MqttConnector;
use std::{
    collections::HashSet,
    fmt::{Debug, Display, Formatter},
    ops::{Deref, DerefMut},
    rc::Rc,
};
use yew::{html::Scope, Callback, Component};
use yew_agent::*;

pub struct ConnectorOptions<'a> {
    pub url: &'a str,
    pub credentials: &'a Credentials,
    pub settings: &'a Settings,

    pub on_command: Callback<Command>,
    pub on_connection_lost: Callback<String>,
}

pub struct ConnectOptions {
    pub on_success: Callback<()>,
    pub on_failure: Callback<String>,
}

pub struct SubscribeOptions {
    pub on_success: Callback<()>,
    pub on_failure: Callback<String>,
}

pub trait Connector {
    fn connect(&mut self, opts: ConnectOptions) -> anyhow::Result<()>;
    fn subscribe(&mut self, opts: SubscribeOptions) -> anyhow::Result<()>;
    fn publish(&mut self, channel: String, payload: Vec<u8>, qos: QoS);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub payload: Option<Vec<u8>>,
}

pub struct Simulator {
    link: AgentLink<Self>,
    subscribers: HashSet<HandlerId>,

    state: SimulatorState,

    _settings_agent: SharedDataBridge<Settings>,
    settings: Settings,

    connector: Option<Box<dyn Connector>>,
    commands: Vec<Command>,
}

#[derive(Debug)]
pub enum Msg {
    Settings(Settings),
    Connected,
    Subscribed,
    Disconnected(String),
    Command(Command),
}

pub enum Request {
    Start,
    Stop,
    Publish { channel: String, payload: Vec<u8> },
    FetchHistory,
}

pub enum Response {
    State(SimulatorState),
    Command(Rc<Command>),
    CommandHistory(Vec<Command>),
}

#[derive(Clone, Debug)]
pub struct SimulatorState {
    pub running: bool,
    pub state: State,
}

impl Default for SimulatorState {
    fn default() -> Self {
        Self {
            running: false,
            state: State::Disconnected,
        }
    }
}

#[derive(Clone, Debug)]
pub enum State {
    Connecting,
    Subscribing,
    Connected,
    Disconnected,
    Failed(String),
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connecting => f.write_str("Connecting"),
            Self::Subscribing => f.write_str("Subscribing"),
            Self::Connected => f.write_str("Connected"),
            Self::Disconnected => f.write_str("Disconnected"),
            Self::Failed(err) => write!(f, "Failed ({})", err),
        }
    }
}

impl State {
    #[allow(unused)]
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }
}

impl Agent for Simulator {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        log::info!("Created new simulator");

        let mut settings_agent = SharedDataBridge::new(link.callback(|response| match response {
            data::Response::State(settings) => Msg::Settings(settings),
        }));
        settings_agent.request_state();

        Self {
            link,
            subscribers: HashSet::new(),
            state: SimulatorState {
                running: false,
                state: State::Disconnected,
            },
            _settings_agent: settings_agent,
            settings: Default::default(),
            connector: None,
            commands: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) {
        log::debug!("update: {msg:?}");
        match msg {
            Msg::Settings(settings) => {
                self.update_settings(settings);
            }
            Msg::Connected => {
                self.state.state = State::Subscribing;
                self.send_state();
                if let Some(connector) = &mut self.connector {
                    if let Err(err) = connector.subscribe(SubscribeOptions {
                        on_success: self.link.callback(|_| Msg::Subscribed),
                        on_failure: self.link.callback(|err| Msg::Disconnected(err)),
                    }) {
                        log::warn!("Failed to subscribe: {err}");
                    };
                }
            }
            Msg::Subscribed => {
                self.state.state = State::Connected;
                self.send_state();
            }
            Msg::Disconnected(err) => {
                self.state.state = State::Failed(err);
                self.send_state();
            }
            Msg::Command(command) => {
                // record in history

                self.commands.push(command.clone());
                let command = Rc::new(command);

                // broadcast

                for id in &self.subscribers {
                    self.link
                        .respond(id.clone(), Response::Command(command.clone()));
                }
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        if id.is_respondable() {
            self.subscribers.insert(id);
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Request::Start => {
                if !self.state.running {
                    self.start();
                }
            }
            Request::Stop => {
                if self.state.running {
                    self.stop();
                }
            }
            Request::Publish { channel, payload } => {
                if let Some(connector) = &mut self.connector {
                    connector.publish(channel, payload, QoS::QoS0);
                }
            }
            Request::FetchHistory => {
                if id.is_respondable() {
                    self.link
                        .respond(id, Response::CommandHistory(self.commands.clone()));
                }
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        if id.is_respondable() {
            self.subscribers.remove(&id);
        }
    }
}

impl Simulator {
    fn send_state(&self) {
        log::debug!("Broadcast state: {:?}", self.state);
        for id in &self.subscribers {
            self.link
                .respond(id.clone(), Response::State(self.state.clone()));
        }
    }

    fn start(&mut self) {
        self.state.running = true;
        self.send_state();

        log::info!("Creating client");

        let connector = match &self.settings.target {
            Target::Mqtt { url, credentials } => {
                let mut connector = MqttConnector::new(ConnectorOptions {
                    credentials,
                    url,
                    settings: &self.settings,
                    on_connection_lost: self.link.callback(|err| Msg::Disconnected(err)),
                    on_command: self.link.callback(|msg| Msg::Command(msg)),
                });

                self.state.state = State::Connecting;
                self.send_state();

                if let Err(err) = connector.connect(ConnectOptions {
                    on_success: self.link.callback(|_| Msg::Connected),
                    on_failure: self.link.callback(|err| Msg::Disconnected(err)),
                }) {
                    log::warn!("Failed to start connecting: {err}");
                }

                Some(Box::new(connector) as Box<dyn Connector>)
            }
            // FIXME: implement HTTP too
            _ => None,
        };

        self.connector = connector;

        log::info!("Created");
    }

    fn stop(&mut self) {
        self.connector.take();
        self.state.running = false;
        self.state.state = State::Disconnected;
        self.send_state();
    }

    fn update_settings(&mut self, settings: Settings) {
        self.settings = settings;
        if self.state.running {
            // disconnect to trigger reconnect
            self.stop();
            self.start();
        } else if self.settings.auto_connect {
            // auto-connect on, but not started yet
            self.start();
        }
    }
}

pub struct SimulatorBridge(Box<dyn Bridge<Simulator>>);

impl SimulatorBridge {
    pub fn new(callback: Callback<Response>) -> SimulatorBridge {
        Self(Simulator::bridge(callback))
    }

    pub fn from<C, F>(link: &Scope<C>, f: F) -> Self
    where
        C: Component,
        F: Fn(SimulatorState) -> C::Message + 'static,
    {
        let callback = link.batch_callback(move |msg| match msg {
            Response::State(data) => vec![f(data)],
            _ => vec![],
        });
        Self::new(callback)
    }

    pub fn start(&mut self) {
        self.send(Request::Start);
    }

    pub fn stop(&mut self) {
        self.send(Request::Stop);
    }

    pub fn publish<C, P>(&mut self, channel: C, payload: P)
    where
        C: Into<String>,
        P: Into<Vec<u8>>,
    {
        self.send(Request::Publish {
            channel: channel.into(),
            payload: payload.into(),
        })
    }
}

impl Deref for SimulatorBridge {
    type Target = Box<dyn Bridge<Simulator>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SimulatorBridge {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
