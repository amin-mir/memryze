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

    let submit_qa = {
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

            let onerror = onerror.clone();
            spawn_local(async move {
                let msg = Message::AddQA { q: &q, a: &a };
                let args = to_value(&msg).unwrap();
                let res = add_qa(args).await;
                match res {
                    Ok(_) => {
                        web_sys::console::log_1(&"QA submitted successfully".into());
                    }
                    Err(e) => {
                        onerror.emit(e.as_string().unwrap());
                    }
                }
            });
        })
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
                    />
                </div>
                <div class="input-group">
                    <label>{"Answer"}</label>
                    <textarea ref={a_ref}
                        placeholder="Type your answer..."
                        type="text"
                        name="answer"
                        rows=10
                    />
                </div>
            </div>
            <button type="submit" class="submit-button" onclick={submit_qa}>{"Submit"}</button>
        </>
    }
}
