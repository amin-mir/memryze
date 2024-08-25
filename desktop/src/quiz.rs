use gloo_timers::future::TimeoutFuture;
use message::{Message, QA};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::commands::{get_quiz, review_qa};

#[function_component(QuizComponent)]
pub fn quiz() -> Html {
    let q_ref = use_node_ref();
    let a_ref = use_node_ref();

    let fetched_qas = use_state(|| Vec::<QA>::new());
    let current_qa_idx = use_state(|| 0);
    let status_message = use_state(|| String::from("Fetching a quiz"));
    let submit_review_success = use_state(|| false);

    {
        // TODO: Every time we switch to this component a fetch request is sent to server.
        // Instead App should handle this and pass a callback that this component could
        // call to refresh the qas, and also recieve the qas as props.
        let fetched_qas = fetched_qas.clone();
        let current_qa_idx = current_qa_idx.clone();
        let status_message = status_message.clone();

        use_effect_with((), move |_| {
            let fetched_qas = fetched_qas.clone();
            let current_qa_idx = current_qa_idx.clone();
            let status_message = status_message.clone();

            spawn_local(async move {
                // TODO: try dereferencin the state qas.
                let qas: Vec<QA> = Vec::new();
                let args = to_value(&qas).unwrap();
                let res = get_quiz(args).await;
                match res {
                    Ok(jsval) => {
                        match from_value(jsval) {
                            Ok(qas) => {
                                fetched_qas.set(qas);
                            }
                            Err(e) => status_message.set(e.to_string()),
                        }
                        current_qa_idx.set(0);
                    }
                    Err(e) => {
                        status_message.set(e.as_string().unwrap());
                    }
                }
            });
        });
    }

    {
        // Change the q & a values when fetching from server.
        let q_ref = q_ref.clone();
        let a_ref = a_ref.clone();
        let fetched_qas = fetched_qas.clone();
        let current_qa_idx = current_qa_idx.clone();

        let dep = fetched_qas.clone();
        use_effect_with(dep, move |_| {
            if fetched_qas.len() == 0 {
                return;
            }

            let qas_str = format!("{:?}", *fetched_qas);
            web_sys::console::log_1(&qas_str.into());

            let qa = &fetched_qas[*current_qa_idx];

            let q = q_ref.cast::<web_sys::HtmlTextAreaElement>().unwrap();
            q.set_value(&qa.q);

            let a = a_ref.cast::<web_sys::HtmlTextAreaElement>().unwrap();
            a.set_value(&qa.a);
        });
    }

    // TODO: Also increase the current index and use_effect_with on that to
    // display the next question. If index = len - 1, we need to fetch a new
    // set of qas.
    let correct_review = {
        let fetched_qas = fetched_qas.clone();
        let current_qa_idx = current_qa_idx.clone();
        let status_message = status_message.clone();
        let submit_review_success = submit_review_success.clone();

        Callback::from(move |_: MouseEvent| {
            let fetched_qas = fetched_qas.clone();
            let current_qa_idx = current_qa_idx.clone();
            let status_message = status_message.clone();
            let submit_review_success = submit_review_success.clone();
            spawn_local(async move {
                let qa = &fetched_qas[*current_qa_idx];
                submit_review_qa(status_message, submit_review_success, qa.id, true).await
            });
        })
    };

    let wrong_review = {
        let fetched_qas = fetched_qas.clone();
        let current_qa_idx = current_qa_idx.clone();
        let status_message = status_message.clone();
        let submit_review_success = submit_review_success.clone();

        Callback::from(move |_: MouseEvent| {
            let fetched_qas = fetched_qas.clone();
            let current_qa_idx = current_qa_idx.clone();
            let status_message = status_message.clone();
            let submit_review_success = submit_review_success.clone();

            spawn_local(async move {
                let qa = &fetched_qas[*current_qa_idx];
                submit_review_qa(status_message, submit_review_success, qa.id, false).await;
            });
        })
    };

    html! {
        <>
            <p><span>{if status_message.is_empty() { "" } else { "* "}}</span>{&*status_message}</p>
            <div class="row">
                <div class="input-group">
                    <label>{"Question"}</label>
                    <textarea
                        class={classes!(if *submit_review_success { "successful-submit" } else { "" })}
                        ref={q_ref}
                        name="question"
                        rows=10
                    />
                </div>
                <div class="input-group">
                    <label>{"Answer"}</label>
                    <textarea
                        class={classes!(if *submit_review_success { "successful-submit" } else { "" })}
                        ref={a_ref}
                        name="answer"
                        rows=10
                    />
                </div>
            </div>
            <div class="actions">
                <button type="submit" class="submit-button" onclick={correct_review}>{"Correct"}</button>
                <button type="submit" class="submit-button error-button" onclick={wrong_review}>{"Wrong"}</button>
            </div>
        </>
    }
}

async fn submit_review_qa(
    status_message: UseStateHandle<String>,
    submit_review_success: UseStateHandle<bool>,
    id: i64,
    correct: bool,
) {
    let msg = Message::ReviewQA { id, correct };
    let args = to_value(&msg).unwrap();
    match review_qa(args).await {
        Ok(_) => {
            status_message.set("".to_string());
            submit_review_success.set(true);
            TimeoutFuture::new(500).await;
            submit_review_success.set(false);
        }
        Err(e) => status_message.set(e.as_string().unwrap()),
    }
}
