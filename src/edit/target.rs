use super::*;
use crate::simulator::simulations::{FeatureTarget, SingleTarget};

pub fn edit_target<S>(setter: &S, target: &SingleTarget) -> Html
where
    S: Setter<SingleTarget>,
{
    html!(<>
        <FormSection title="Target">
            { setter_field(setter, "Channel", target.channel.clone(), |state, value| state.channel = value ) }
            { setter_field(setter, "Feature", target.feature.clone(), |state, value| state.feature = value ) }
            { setter_field(setter, "Property", target.property.clone(), |state, value| state.property = value ) } 
        </FormSection>
    </>)
}

pub fn edit_feature_target<S>(setter: &S, target: &FeatureTarget) -> Html
where
    S: Setter<FeatureTarget>,
{
    html!(<>
        <FormSection title="Target">
            { setter_field(setter, "Channel", target.channel.clone(), |state, value| state.channel = value ) }
            { setter_field(setter, "Feature", target.feature.clone(), |state, value| state.feature = value ) }
        </FormSection>
    </>)
}
