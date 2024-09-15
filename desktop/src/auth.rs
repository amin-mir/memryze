use serde_wasm_bindgen::{from_value, to_value};
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::commands::{is_connected, update_api_key};

#[derive(Properties, PartialEq)]
pub struct AuthProperties {
    pub onconnect: Callback<()>,
}

#[function_component(Auth)]
pub fn auth(props: &AuthProperties) -> Html {
    let api_key_ref = use_node_ref();
    let status_message = use_state(|| String::from(""));
    let submit_disabled = use_state(|| false);

    {
        let status_message = status_message.clone();
        let onconnect = props.onconnect.clone();

        use_effect_with((), move |_| {
            let status_message = status_message.clone();
            let onconnect = onconnect.clone();

            spawn_local(async move {
                match is_connected().await {
                    Ok(jsval) => match from_value(jsval) {
                        Ok(connected) => {
                            web_sys::console::log_1(&format!("is_connected: {connected}").into());
                            if connected {
                                onconnect.emit(());
                            }
                        }
                        Err(e) => status_message.set(e.to_string()),
                    },
                    Err(e) => status_message.set(e.as_string().unwrap()),
                }
            });
        });
    };

    let onblur = {
        let submit_disabled = submit_disabled.clone();
        move |e: FocusEvent| {
            let api_key: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            submit_disabled.set(api_key.value().is_empty());
        }
    };

    let onclick = {
        let api_key_ref = api_key_ref.clone();
        let status_message = status_message.clone();
        let onconnect = props.onconnect.clone();

        Callback::from(move |_: MouseEvent| {
            status_message.set("".to_string());

            // Button only enabled when textarea value is not empty.
            let api_key = api_key_ref
                .cast::<web_sys::HtmlTextAreaElement>()
                .unwrap()
                .value();

            let status_message = status_message.clone();
            let onconnect = onconnect.clone();
            spawn_local(async move {
                let args = to_value(&api_key).unwrap();
                let res = update_api_key(args).await;
                match res {
                    Ok(_) => {
                        web_sys::console::log_1(&"API Key updated".into());
                        onconnect.emit(());
                    }
                    Err(e) => {
                        status_message.set(e.as_string().unwrap());
                    }
                }
            });
        })
    };

    html! {
        <main class="container">
            <p><span>{if status_message.is_empty() { "" } else { "* "}}</span>{&*status_message}</p>
            <div class="row">
                <div class="input-group">
                    <label>{"Insert your API Key"}</label>
                    <textarea
                        name="api-key"
                        placeholder={"API KEY"}
                        rows=10
                        ref={api_key_ref}
                        {onblur}
                    />
                </div>
            </div>
            <div class="actions">
                <button
                    type="submit"
                    class="submit-button"
                    disabled={*submit_disabled}
                    {onclick}
                >{"Submit"}</button>
            </div>
        </main>
    }
}
