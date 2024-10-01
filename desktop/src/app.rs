use message::QA;
use serde_wasm_bindgen::from_value;
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::commands::get_quiz;
use crate::quiz::QuizComponent;
use crate::submit::SubmitComponent;

#[derive(PartialEq, Copy, Clone)]
enum NavbarSelected {
    Submit,
    Quiz,
}

#[function_component(App)]
pub fn app() -> Html {
    let navbar_selected = use_state(|| NavbarSelected::Submit);
    let fetched_qas = use_state(|| Vec::<QA>::new());
    let current_qa_idx = use_state(|| 0);
    let status_message = use_state(|| String::from(""));

    {
        let fetched_qas = fetched_qas.clone();
        let status_message = status_message.clone();

        use_effect_with((), move |_| {
            let fetched_qas = fetched_qas.clone();
            let status_message = status_message.clone();

            spawn_local(async move {
                match refresh_quiz().await {
                    Ok(qas) => fetched_qas.set(qas),
                    Err(e) => status_message.set(e),
                }
            });
        });
    }

    let make_onselect_cb = |ns: NavbarSelected| {
        let navbar_selected = navbar_selected.clone();
        Callback::from(move |_| {
            navbar_selected.set(ns);
        })
    };
    let onselect_submit = make_onselect_cb(NavbarSelected::Submit);
    let onselect_quiz = make_onselect_cb(NavbarSelected::Quiz);

    let (nav_submit_cls, nav_quiz_cls) = if *navbar_selected == NavbarSelected::Submit {
        ("navbar-selected", "")
    } else {
        ("", "navbar-selected")
    };

    // TODO: Also when new questions are submitted, submit component calls a callback
    // to signal quiz should be refreshed. If the current qas is empty, then
    // a refresh should be attempted upon switching from submit to quiz component.

    let current_qa = fetched_qas.get(*current_qa_idx).cloned();
    let onerror = {
        let status_message = status_message.clone();
        Callback::from(move |msg| {
            status_message.set(msg);
        })
    };
    let onreview = {
        let fetched_qas = fetched_qas.clone();
        let current_qa_idx = current_qa_idx.clone();
        let status_message = status_message.clone();

        Callback::from(move |_| {
            let fetched_qas = fetched_qas.clone();
            let status_message = status_message.clone();

            let new_idx = *current_qa_idx + 1;
            if new_idx != fetched_qas.len() {
                current_qa_idx.set(new_idx);
            } else {
                current_qa_idx.set(0);
                spawn_local(async move {
                    match refresh_quiz().await {
                        Ok(qas) => {
                            web_sys::console::log_1(&"qas refreshed after full consumption".into());
                            fetched_qas.set(qas)
                        }
                        Err(e) => status_message.set(e),
                    }
                });
            }
        })
    };

    html! {
        <main class="container">
            <nav>
                <ul class="navbar">
                    <li class={nav_submit_cls} onclick={onselect_submit}>{"Submit"}</li>
                    <li class={nav_quiz_cls} onclick={onselect_quiz}>{"Quiz"}</li>
                </ul>
             </nav>

            <p><span>{if status_message.is_empty() { "" } else { "* "}}</span>{&*status_message}</p>

            if *navbar_selected == NavbarSelected::Submit {
                <SubmitComponent {onerror} />
            } else {
                <QuizComponent qa={current_qa}
                    {onreview}
                    {onerror}
                />
            }
        </main>
    }
}

async fn refresh_quiz() -> Result<Vec<QA>, String> {
    let quiz = get_quiz().await;
    match quiz {
        Ok(jsval) => match from_value(jsval) {
            Ok(qas) => Ok(qas),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.as_string().unwrap()),
    }
}
