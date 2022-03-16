use crate::connector::mqtt::{MqttClient, MqttConnectOptions, MqttMessage, QoS};
use crate::data::{self, SharedDataBridge};
use crate::settings::{Credentials, Settings, Target};
use std::{
    collections::HashSet,
    fmt::{Debug, Display, Formatter},
    ops::{Deref, DerefMut},
    rc::Rc,
    time::Duration,
};
use yew::{html::Scope, Callback, Component};
use yew_agent::*;

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

    client: Option<MqttClient>,
    commands: Vec<Command>,
}

#[derive(Debug)]
pub enum Msg {
    Settings(Settings),
    Connected,
    Subscribed,
    Disconnected(String),
    Message(MqttMessage),
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
            client: None,
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
                self.subscribe();
            }
            Msg::Subscribed => {
                self.state.state = State::Connected;
                self.send_state();
            }
            Msg::Disconnected(err) => {
                self.state.state = State::Failed(err);
                self.send_state();
            }
            Msg::Message(msg) => {
                let command = Command {
                    name: msg.topic,
                    payload: Some(msg.payload),
                };

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
                if let Some(client) = &self.client {
                    client.publish(channel, payload, QoS::QoS0, false);
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

        let client = match &self.settings.target {
            Target::Mqtt { url, credentials } => {
                let mut client = MqttClient::new(url, None);
                client.set_on_connection_lost(self.link.callback(|err| Msg::Disconnected(err)));
                client.set_on_message_arrived(self.link.callback(|msg| Msg::Message(msg)));

                let (username, password) = match credentials {
                    Credentials::None => (None, None),
                    Credentials::Password(password) => (
                        Some(format!(
                            "{}@{}",
                            self.settings.device, self.settings.application
                        )),
                        Some(password.clone()),
                    ),
                    Credentials::UsernamePassword { username, password } => {
                        (Some(username.clone()), Some(password.clone()))
                    }
                };

                self.state.state = State::Connecting;
                self.send_state();

                client.connect(
                    MqttConnectOptions {
                        username,
                        password,
                        clean_session: true,
                        reconnect: true,
                        keep_alive_interval: Some(Duration::from_secs(2)),
                        timeout: Some(Duration::from_secs(5)),
                    },
                    self.link.callback(|_| Msg::Connected),
                    self.link.callback(|err| Msg::Disconnected(err)),
                );

                Some(client)
            }
            // FIXME: implement HTTP too
            _ => None,
        };

        self.client = client;

        log::info!("Created");
    }

    fn stop(&mut self) {
        self.state.running = false;
        self.send_state();
    }

    fn subscribe(&mut self) {
        if let Some(client) = &self.client {
            client.subscribe(
                "command/inbox/#",
                QoS::QoS0,
                Duration::from_secs(5),
                self.link.callback(|_| Msg::Subscribed),
                self.link.callback(|err| Msg::Disconnected(err)),
            );
        }
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
