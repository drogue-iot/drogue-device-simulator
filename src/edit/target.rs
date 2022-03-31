use super::*;
use crate::simulator::simulations::SingleTarget;

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
