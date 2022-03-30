use super::*;
use crate::simulator::generators::SingleTarget;
use std::rc::Rc;

pub fn edit_target<STATE, S, F>(setter: &S, target: &SingleTarget, f: F) -> Html
where
    F: Fn(&mut STATE) -> Option<&mut SingleTarget> + 'static,
    S: Setter<STATE>,
{
    let f = Rc::new(f);

    html!(<>
            <FormSection title="Target">
                { setter_field(setter, "Channel", target.channel.clone(), {
                    let f = f.clone();
                    move |state, value: String| {
                        if let Some(target) = f(state) {
                            target.channel = value;
                        }
                    }
                }) }

                { setter_field(setter, "Feature", target.feature.clone(), {
                    let f = f.clone();
                    move |state, value: Option<String>| {
                        if let Some(target) = f(state) {
                            target.feature = value;
                        }
                    }
                }) }

                { setter_field(setter, "Property", target.property.clone(), move |state, value| {
                   if let Some(target) = f(state) {
                        target.property = value;
                    }
                }) }

            </FormSection>
        </>)
}
