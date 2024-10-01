use gloo_timers::callback::Timeout;
use message::Message;
use serde_wasm_bindgen::to_value;
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::commands::add_qa;

#[derive(Properties, PartialEq)]
pub struct SubmitProperties {
    pub onerror: Callback<String>,
}

#[function_component(SubmitComponent)]
pub fn submit(props: &SubmitProperties) -> Html {
    let q_ref = use_node_ref();
    let a_ref = use_node_ref();

    let submit_disabled = use_state(|| true);
    let submit_success = use_state(|| false);

    let checkmark_class = if *submit_success { "visible" } else { "hidden" };

    let submit_qa = {
        let submit_success = submit_success.clone();
        let q_ref = q_ref.clone();
        let a_ref = a_ref.clone();
        let onerror = props.onerror.clone();

        Callback::from(move |_: MouseEvent| {
            onerror.emit("".to_string());

            let q = q_ref
                .cast::<web_sys::HtmlTextAreaElement>()
                .unwrap()
                .value();

            let a = a_ref
                .cast::<web_sys::HtmlTextAreaElement>()
                .unwrap()
                .value();

            if q.is_empty() || a.is_empty() {
                onerror.emit("Question/Answer can't be empty".to_string());
                return;
            }

            let submit_success = submit_success.clone();
            let onerror = onerror.clone();
            let q_ref = q_ref.clone();
            let a_ref = a_ref.clone();
            spawn_local(async move {
                let msg = Message::AddQA { q: &q, a: &a };
                let args = to_value(&msg).unwrap();
                let res = add_qa(args).await;
                match res {
                    Ok(_) => {
                        submit_success.set(true);
                        q_ref
                            .cast::<web_sys::HtmlTextAreaElement>()
                            .unwrap()
                            .set_value("");
                        a_ref
                            .cast::<web_sys::HtmlTextAreaElement>()
                            .unwrap()
                            .set_value("");

                        let submit_success = submit_success.clone();
                        Timeout::new(2000, move || {
                            submit_success.set(false);
                        })
                        .forget();
                    }
                    Err(e) => {
                        onerror.emit(e.as_string().unwrap());
                    }
                }
            });
        })
    };

    let submit_disabled_cb = {
        let submit_disabled = submit_disabled.clone();
        let a_ref = a_ref.clone();
        let q_ref = q_ref.clone();

        move || {
            let q = q_ref
                .cast::<web_sys::HtmlTextAreaElement>()
                .unwrap()
                .value();

            if q.is_empty() {
                submit_disabled.set(true);
                return;
            }

            let a = a_ref
                .cast::<web_sys::HtmlTextAreaElement>()
                .unwrap()
                .value();

            if a.is_empty() {
                submit_disabled.set(true);
                return;
            }

            submit_disabled.set(false);
        }
    };

    let onkeyup = {
        let submit_disabled_cb = submit_disabled_cb.clone();
        move |_| {
            submit_disabled_cb();
        }
    };

    let onblur = {
        let submit_disabled_cb = submit_disabled_cb.clone();
        move |_| {
            submit_disabled_cb();
        }
    };

    html! {
        <>
            <div class="row">
                <div class="input-group">
                    <label>{"Question"}</label>
                    <textarea ref={q_ref}
                        placeholder="Type your question..."
                        type="text"
                        name="question"
                        rows=10
                        onkeyup={onkeyup.clone()}
                        onblur={onblur.clone()}
                    />
                </div>
                <div class="input-group">
                    <label>{"Answer"}</label>
                    <textarea ref={a_ref}
                        placeholder="Type your answer..."
                        type="text"
                        name="answer"
                        rows=10
                        {onkeyup}
                        {onblur}
                    />
                </div>
            </div>
            <div class="actions actions-margined">
                <button type="submit" disabled={*submit_disabled} class="submit-button" onclick={submit_qa}>{"Submit"}</button>
                <span class={classes!("checkmark", checkmark_class)}>{ "\u{2713}" }</span>
            </div>
        </>
    }
}
