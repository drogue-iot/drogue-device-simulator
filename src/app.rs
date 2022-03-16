use crate::data::{SharedDataBridge, SharedDataOps};
use gloo_utils::window;
use patternfly_yew::*;
use url::Url;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages;
use crate::settings::Settings;
use crate::simulator::{SimulatorBridge, SimulatorState};

#[derive(Switch, Debug, Clone, PartialEq, Eq)]
pub enum AppRoute {
    #[to = "/connection"]
    Connection,
    #[to = "/publish"]
    Publish,
    #[to = "/commands"]
    Commands,
    #[to = "/config"]
    Configuration,
    #[to = "/!"]
    Overview,
}

pub struct Application {}

impl Component for Application {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html!(
            <>
                <BackdropViewer/>
                <ToastViewer/>

                <ApplicationView/>
            </>
        )
    }
}

pub enum Msg {
    Settings(Settings),
    Simulator(SimulatorState),

    Start,
    Stop,
}

pub struct ApplicationView {
    settings: Settings,
    _settings_agent: SharedDataBridge<Settings>,
    simulator: SimulatorBridge,
    simulator_state: SimulatorState,
}

impl Component for ApplicationView {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let cfg = find_config();

        let mut _settings_agent = SharedDataBridge::from(ctx.link(), Msg::Settings);

        match cfg {
            Some(cfg) => {
                _settings_agent.set(cfg);
            }
            None => {
                _settings_agent.request_state();
            }
        }

        let simulator = SimulatorBridge::from(ctx.link(), Msg::Simulator);

        Self {
            settings: Default::default(),
            _settings_agent,
            simulator,
            simulator_state: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Settings(settings) => {
                self.settings = settings;
            }
            Msg::Simulator(state) => {
                self.simulator_state = state;
            }
            Msg::Start => {
                self.simulator.start();
            }
            Msg::Stop => {
                self.simulator.stop();
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let sidebar = html_nested! {
            <PageSidebar>
                <Nav>
                    <NavList>
                        <NavRouterExpandable<AppRoute> title="Home">
                            <NavRouterItem<AppRoute> to={AppRoute::Overview}>{"Overview"}</NavRouterItem<AppRoute>>
                            <NavRouterItem<AppRoute> to={AppRoute::Connection}>{"Connection"}</NavRouterItem<AppRoute>>
                            <NavRouterItem<AppRoute> to={AppRoute::Configuration}>{"Configuration"}</NavRouterItem<AppRoute>>
                        </NavRouterExpandable<AppRoute>>
                        <NavRouterExpandable<AppRoute> title="Basic">
                            <NavRouterItem<AppRoute> to={AppRoute::Publish}>{"Publish"}</NavRouterItem<AppRoute>>
                            <NavRouterItem<AppRoute> to={AppRoute::Commands}>{"Received Commands"}</NavRouterItem<AppRoute>>
                        </NavRouterExpandable<AppRoute>>
                    </NavList>
                </Nav>
            </PageSidebar>
        };

        let logo = html_nested! {
            <Logo src="/images/logo.png" alt="Drogue IoT" />
        };

        let tools = vec![
            html!(
                <div>
                    <strong>{"State: "}</strong> { self.simulator_state.state.to_string() }
                </div>
            ),
            html!(
                <>
                    <Button
                        icon={Icon::Play}
                        variant={Variant::Plain}
                        disabled={self.simulator_state.running}
                        onclick={ctx.link().callback(|_|Msg::Start)}
                    />
                    <Button
                        icon={Icon::Pause}
                        variant={Variant::Plain}
                        disabled={!self.simulator_state.running}
                        onclick={ctx.link().callback(|_|Msg::Stop)}
                    />
                </>
            ),
        ];

        html! (
            <Page
                logo={logo}
                sidebar={sidebar}
                tools={Children::new(tools)}
                >
                    <Router<AppRoute, ()>
                            redirect = {Router::redirect(|_|AppRoute::Overview)}
                            render = {Router::render(move |switch: AppRoute| {
                                match switch {
                                    AppRoute::Overview => html!{<pages::AppPage<pages::Overview>/>},
                                    AppRoute::Connection => html!{<pages::AppPage<pages::Connection>/>},
                                    AppRoute::Publish => html!{<pages::AppPage<pages::Publish>/>},
                                    AppRoute::Commands => html!{<pages::AppPage<pages::Commands>/>},
                                    AppRoute::Configuration => html!{<pages::AppPage<pages::Configuration>/>},
                                }
                            })}
                        />
            </Page>
        )
    }
}

fn find_config() -> Option<Settings> {
    if let Some(cfg) = find_config_str() {
        log::info!("Found provided settings");
        base64::decode_config(&cfg, base64::URL_SAFE)
            .map_err(|err| {
                log::info!("Failed to decode base64 encoding: {err} was: {cfg}");
            })
            .ok()
            .and_then(|cfg| {
                serde_json::from_slice(&cfg)
                    .map_err(|err| {
                        log::info!(
                            "Failed to parse provided configuration: {err} was: {:?}",
                            String::from_utf8(cfg)
                        );
                        err
                    })
                    .ok()
            })
    } else if let Ok(settings) = Settings::load() {
        log::info!("Found default settings");
        Some(settings)
    } else {
        log::info!("Not settings found");
        None
    }
}

fn find_config_str() -> Option<String> {
    if let Ok(href) = window().location().href() {
        if let Ok(url) = Url::parse(&href) {
            for q in url.query_pairs() {
                if q.0 == "c" {
                    return Some(q.1.to_string());
                }
            }
        }
    }
    None
}
