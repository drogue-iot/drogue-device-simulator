use crate::settings::Payload;
use crate::{
    data::{SharedDataBridge, SharedDataOps},
    pages::ApplicationPage,
    settings::{Credentials, PayloadFormat, Protocol, Settings, Target},
};
use patternfly_yew::*;
use std::fmt::{Display, Formatter};
use std::time::Duration;
use strum::EnumString;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

pub struct Connection {
    // stored settings
    settings: Settings,
    settings_agent: SharedDataBridge<Settings>,

    // in edit settings
    auto_connect: bool,
    protocol: Protocol,
    url: String,
    credentials: CredentialsType,
    username: String,
    password: String,
    application: String,
    device: String,
    payload: PayloadFormatType,

    // refs
    refs: Refs,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, EnumString)]
enum CredentialsType {
    None,
    Password,
    UsernamePassword,
}

impl Display for CredentialsType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => f.write_str("None"),
            Self::Password => f.write_str("Password"),
            Self::UsernamePassword => f.write_str("Username/password"),
        }
    }
}

/// Wrapper type for working with the UI form.
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, EnumString, strum::Display)]
pub enum PayloadFormatType {
    #[default]
    JsonCompact,
    Doppelgaenger,
}

#[derive(Clone, Default)]
struct Refs {
    protocol: NodeRef,
    credentials: NodeRef,
    payload: NodeRef,
}

impl ApplicationPage for Connection {
    fn title() -> String {
        "Connection".into()
    }
}

pub enum Msg {
    Settings(Settings),

    Set(Box<dyn FnOnce(&mut Connection)>),

    Apply,
    Reset,
}

impl Component for Connection {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut settings_agent = SharedDataBridge::from(ctx.link(), Msg::Settings);
        settings_agent.request_state();

        Self {
            settings: Default::default(),
            settings_agent,

            auto_connect: true,
            protocol: Protocol::Mqtt,
            url: Default::default(),
            credentials: CredentialsType::None,
            username: Default::default(),
            password: Default::default(),
            application: Default::default(),
            device: Default::default(),
            payload: Default::default(),

            refs: Default::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Settings(settings) => {
                self.settings = settings;
                self.sync();
            }

            Msg::Set(mutator) => mutator(self),

            Msg::Reset => {
                self.sync();
            }
            Msg::Apply => {
                self.update_settings();
                ToastDispatcher::new().toast(Toast {
                    title: "Configuration updated".to_string(),
                    r#type: Type::Success,
                    timeout: Some(Duration::from_secs(5)),
                    body: Default::default(),
                    actions: vec![],
                });
                return false;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let selected_protocol = |protocol| self.protocol == protocol;
        let selected_credentials = |credentials| self.credentials == credentials;
        let selected_payload = |payload| self.payload == payload;

        let set_protocol = ctx
            .link()
            .callback(|v| Msg::Set(Box::new(move |c| c.protocol = v)));
        let set_credentials = ctx
            .link()
            .callback(|v| Msg::Set(Box::new(move |c| c.credentials = v)));
        let set_payload_format = ctx
            .link()
            .callback(|v| Msg::Set(Box::new(move |c| c.payload = v)));

        let (username_disabled, password_disabled) = match self.credentials {
            CredentialsType::None => (true, true),
            CredentialsType::Password => (true, false),
            CredentialsType::UsernamePassword => (false, false),
        };

        html!(
            <PageSection variant={PageSectionVariant::Light} fill={true}>
                <Flex>
                    <FlexItem modifiers={[FlexModifier::Grow]}>
                        <Form horizontal={[FormHorizontal]} >
                            <FormSection title="Connection">

                                <FormGroup
                                    required=true
                                    label={"Connection type"}>
                                    <FormSelect<Protocol> variant={SelectVariant::Single(set_protocol)} ref={self.refs.protocol.clone()}>
                                        <FormSelectOption<Protocol> selected={selected_protocol(Protocol::Mqtt)} value={Protocol::Mqtt} description="MQTT over WebSocket"  />
                                    </FormSelect<Protocol>>
                                </FormGroup>

                                <FormGroup
                                    required=true
                                    label="URL"
                                    >
                                    <TextInput
                                        r#type="url"
                                        onchange={ctx.link().callback(|v| Msg::Set(Box::new(|c|c.url = v)))}
                                        value={self.url.clone()}
                                    />
                                </FormGroup>

                                <FormGroup
                                    label="Application"
                                    >
                                    <TextInput
                                        r#type="text"
                                        onchange={ctx.link().callback(|v| Msg::Set(Box::new(|c|c.application = v)))}
                                        value={self.application.clone()}
                                    />
                                </FormGroup>

                                <FormGroup
                                    label="Device"
                                    >
                                    <TextInput
                                        r#type="text"
                                        onchange={ctx.link().callback(|v| Msg::Set(Box::new(|c|c.device = v)))}
                                        value={self.device.clone()}
                                    />
                                </FormGroup>

                                <FormGroup
                                    required=true
                                    label={"Credentials type"}>
                                    <FormSelect<CredentialsType> variant={SelectVariant::Single(set_credentials)} ref={self.refs.credentials.clone()}>
                                        <FormSelectOption<CredentialsType> selected={selected_credentials(CredentialsType::None)} value={CredentialsType::None} />
                                        <FormSelectOption<CredentialsType> selected={selected_credentials(CredentialsType::Password)} value={CredentialsType::Password} />
                                        <FormSelectOption<CredentialsType> selected={selected_credentials(CredentialsType::UsernamePassword)} value={CredentialsType::UsernamePassword} />
                                    </FormSelect<CredentialsType>>
                                </FormGroup>

                                <FormGroup
                                    label="Username"
                                    >
                                    <TextInput
                                        disabled={username_disabled}
                                        onchange={ctx.link().callback(|v| Msg::Set(Box::new(|c|c.username = v)))}
                                        value={self.username.clone()}
                                    />
                                </FormGroup>

                                <FormGroup
                                    label="Password"
                                    >
                                    <TextInput
                                        disabled={password_disabled}
                                        r#type="password"
                                        onchange={ctx.link().callback(|v| Msg::Set(Box::new(|c|c.password = v)))}
                                        value={self.password.clone()}
                                    />
                                </FormGroup>

                                <FormGroup
                                    label="Auto-connect"
                                    >
                                    <Switch
                                        checked={self.auto_connect}
                                        on_change={ctx.link().callback(|v| Msg::Set(Box::new(move |c|c.auto_connect = v)))}
                                    />
                                </FormGroup>
                            </FormSection>

                            <FormSection title="Payload">
                                <FormGroup
                                    label="Payload format"
                                >
                                    <FormSelect<PayloadFormatType> variant={SelectVariant::Single(set_payload_format)} ref={self.refs.payload.clone()}>
                                        <FormSelectOption<PayloadFormatType> description="Compact JSON" selected={selected_payload(PayloadFormatType::JsonCompact)} value={PayloadFormatType::JsonCompact} />
                                        <FormSelectOption<PayloadFormatType> description="Doppelgänger" selected={selected_payload(PayloadFormatType::Doppelgaenger)} value={PayloadFormatType::Doppelgaenger} />
                                    </FormSelect<PayloadFormatType>>
                                </FormGroup>
                            </FormSection>

                            <ActionGroup>
                                <Button label={"Apply"} variant={Variant::Primary} onclick={ctx.link().callback(|_|Msg::Apply)}/>
                                <Button label={"Reset"} variant={Variant::Secondary} onclick={ctx.link().callback(|_|Msg::Reset)}/>
                            </ActionGroup>
                        </Form>
                    </FlexItem>
                    <FlexItem modifiers={[FlexModifier::Grow]}></FlexItem>
                </Flex>
            </PageSection>
        )
    }
}

impl Connection {
    /// update the form from the settings
    fn sync(&mut self) {
        self.auto_connect = self.settings.auto_connect;
        self.protocol = self.settings.target.as_protocol();
        if let Some(input) = self.refs.protocol.cast::<HtmlInputElement>() {
            input.set_value(&self.protocol.to_string());
        }
        let (url, credentials) = match &self.settings.target {
            Target::Http { url, credentials } | Target::Mqtt { url, credentials } => {
                (url, credentials)
            }
        };
        self.url = url.clone();
        self.application = self.settings.application.clone();
        self.device = self.settings.device.clone();
        match credentials {
            Credentials::None => {
                self.username = Default::default();
                self.password = Default::default();
                self.credentials = CredentialsType::None;
            }
            Credentials::Password(password) => {
                self.password = password.into();
                self.credentials = CredentialsType::Password;
            }
            Credentials::UsernamePassword { username, password } => {
                self.username = username.into();
                self.password = password.into();
                self.credentials = CredentialsType::UsernamePassword;
            }
        }
        self.payload = match self.settings.payload.format {
            PayloadFormat::JsonCompact => PayloadFormatType::JsonCompact,
            PayloadFormat::Doppelgaenger => PayloadFormatType::Doppelgaenger,
        };
        if let Some(input) = self.refs.payload.cast::<HtmlSelectElement>() {
            input.set_value(&self.payload.to_string());
        }
    }

    /// update the settings from the form
    fn update_settings(&mut self) {
        let protocol = self.protocol;

        let credentials = match self.credentials {
            CredentialsType::None => Credentials::None,
            CredentialsType::Password => Credentials::Password(self.password.clone()),
            CredentialsType::UsernamePassword => Credentials::UsernamePassword {
                username: self.username.clone(),
                password: self.password.clone(),
            },
        };

        let url = self.url.clone();
        let auto_connect = self.auto_connect;

        let application = self.application.clone();
        let device = self.device.clone();

        let payload = Payload {
            format: match self.payload {
                PayloadFormatType::Doppelgaenger => PayloadFormat::Doppelgaenger,
                PayloadFormatType::JsonCompact => PayloadFormat::JsonCompact,
            },
        };

        self.settings_agent.update(move |settings| {
            settings.auto_connect = auto_connect;
            settings.application = application;
            settings.device = device;
            settings.payload = payload;

            match protocol {
                Protocol::Http => {
                    settings.target = Target::Http { url, credentials };
                }
                Protocol::Mqtt => {
                    settings.target = Target::Mqtt { url, credentials };
                }
            }
        })
    }
}
