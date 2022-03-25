use serde_json::Value;
use yew::prelude::*;
use yew::virtual_dom::VNode;

pub fn render_payload(data: &[u8], expanded: bool) -> Html {
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

pub trait ToDetail {
    fn to_details(&self) -> (VNode, VNode);
}

impl<V> ToDetail for (&str, V)
where
    V: Into<VNode> + Clone,
{
    fn to_details(&self) -> (VNode, VNode) {
        (self.0.into(), self.1.clone().into())
    }
}

pub fn details<'d, const N: usize>(details: [&dyn ToDetail; N]) -> Html {
    html!(
        <dl>
          { for details.into_iter().map(|details|{
              let (label, value) = details.to_details();
              html!(
                  <>
                    <dt>{ label }{ ":" }</dt>
                    <dd>{ value }</dd>
                  </>
              )
          })}
        </dl>
    )
}
