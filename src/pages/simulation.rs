use patternfly_yew::*;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Properties {
    pub id: String,
}

pub struct Simulation {}

impl Component for Simulation {
    type Message = ();
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html!(
            <>
                <PageSection>
                    <Title level={Level::H1} size={Size::XXXXLarge}>{ "Simulation" }</Title>
                </PageSection>
            </>
        )
    }
}
