use message::{Message, QA};
use serde_wasm_bindgen::to_value;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::commands::review_qa;

#[derive(Properties, PartialEq, Clone)]
pub struct QuizProperties {
    pub qa: Option<QA>,
    pub onreview: Callback<()>,
    pub onerror: Callback<String>,
}

#[function_component(QuizComponent)]
pub fn quiz(props: &QuizProperties) -> Html {
    let revealed = use_state(|| false);
    if props.qa.is_none() {
        return html! {
            <p class="cond-render">{ "There are no questions to review" }</p>
        };
    }

    let qa = props.qa.as_ref().unwrap();

    let onreveal = {
        let revealed = revealed.clone();
        move |_| {
            revealed.set(!*revealed);
        }
    };

    let q_val = &qa.q;
    let a_val = if *revealed {
        &qa.a
    } else {
        "Click Reveal to show..."
    };

    let make_review_cb = |correct: bool| /* -> Callback<()> */ {
        let qa_id = qa.id;
        let onerror = props.onerror.clone();
        let onreview = props.onreview.clone();
        let revealed = revealed.clone();

        Callback::from(move |_: MouseEvent| {
            let onerror = onerror.clone();
            let onreview = onreview.clone();
            let revealed = revealed.clone();

            spawn_local(async move {
                // TODO: what if the server fails? We'll skip this question and go to next one.
                submit_review_qa(onerror, qa_id, correct).await;
                onreview.emit(());
                revealed.set(false);
            });
        })
    };

    let correct_review = make_review_cb(true);
    let wrong_review = make_review_cb(false);

    html! {
        <>
            <div class="row">
                <div class="input-group">
                    <label>{"Question"}</label>
                    <textarea
                        name="question"
                        rows=10
                        value={q_val.clone()}
                    />
                </div>
                <div class="input-group">
                    <label>{"Answer"}</label>
                    <textarea
                        disabled={ !*revealed }
                        name="answer"
                        rows=10
                        value={a_val.to_string()}
                    />
                </div>
            </div>
            <div class="actions">
                <button type="submit"
                    disabled={ !*revealed }
                    class="submit-button"
                    onclick={correct_review}
                >{"Correct"}</button>

                <button type="submit"
                    disabled={ !*revealed }
                    class="submit-button error-button"
                    onclick={wrong_review}
                >{"Wrong"}</button>

                <button type="button"
                    disabled={ *revealed }
                    class="submit-button neutral-button"
                    onclick={onreveal}
                >{"Reveal"}</button>
            </div>
        </>
    }
}

async fn submit_review_qa(onerror: Callback<String>, id: i64, correct: bool) {
    let msg = Message::ReviewQA { id, correct };
    let args = to_value(&msg).unwrap();
    match review_qa(args).await {
        Ok(_) => {
            onerror.emit("".to_string());
        }
        Err(e) => onerror.emit(e.as_string().unwrap()),
    }
}
