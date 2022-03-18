use serde_json::Value;
use yew::prelude::*;

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
