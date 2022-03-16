mod commands;
mod config;
mod connection;
mod overview;
mod publish;

pub use commands::*;
pub use config::*;
pub use connection::*;
pub use overview::*;
pub use publish::*;

use patternfly_yew::*;
use std::marker::PhantomData;
use yew::prelude::*;

pub trait ApplicationPage: Component {
    fn title() -> String;
}

pub struct AppPage<P: ApplicationPage> {
    _marker: PhantomData<P>,
}

impl<P> Component for AppPage<P>
where
    P: ApplicationPage + 'static,
    P::Properties: Clone,
{
    type Message = ();
    type Properties = P::Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            _marker: Default::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props().clone();

        html!(
            <>
                <PageSection>
                    <Title level={Level::H1} size={Size::XXXXLarge}>{ P::title() }</Title>
                </PageSection>

                <P ..props />
            </>
        )
    }
}
