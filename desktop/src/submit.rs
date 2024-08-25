use message::Message;
use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::commands::add_qa;

#[function_component(SubmitComponent)]
pub fn submit() -> Html {
    let q_ref = use_node_ref();
    let a_ref = use_node_ref();

    let submit_error = use_state(|| String::from("No error at the beginning"));

    let submit_qa = {
        let q_ref = q_ref.clone();
        let a_ref = a_ref.clone();
        let submit_error = submit_error.clone();

        Callback::from(move |_: MouseEvent| {
            submit_error.set("".to_string());

            let q = q_ref
                .cast::<web_sys::HtmlTextAreaElement>()
                .unwrap()
                .value();
            if q.is_empty() {
                submit_error.set("Question can't be empty".to_string());
                return;
            }

            let a = a_ref
                .cast::<web_sys::HtmlTextAreaElement>()
                .unwrap()
                .value();
            if a.is_empty() {
                submit_error.set("Answer can't be empty".to_string());
                return;
            }

            let submit_error = submit_error.clone();
            spawn_local(async move {
                let msg = Message::AddQA { q: &q, a: &a };
                let args = to_value(&msg).unwrap();
                let res = add_qa(args).await;
                match res {
                    Ok(_) => {
                        web_sys::console::log_1(&"QA submitted successfully".into());
                    }
                    Err(e) => {
                        // let res: Result<(), String> = from_value(res).unwrap();
                        // let obj = format!("{:?}", e);
                        // web_sys::console::log_1(&obj.into());
                        submit_error.set(e.as_string().unwrap());
                    }
                }
            });
        })
    };

    html! {
        <>
            <p><span>{"* "}</span>{&*submit_error}</p>
            <div class="row">
                <div class="input-group">
                    <label>{"Question"}</label>
                    <textarea ref={q_ref} type="text" name="question" rows=10 />
                </div>
                <div class="input-group">
                    <label>{"Answer"}</label>
                    <textarea ref={a_ref} type="text" name="answer" rows=10 />
                </div>
            </div>
            <button type="submit" class="submit-button" onclick={submit_qa}>{"Submit"}</button>
        </>
    }
}
