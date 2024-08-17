use message::Message;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use types::MyGreetArgs;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen(module = "/public/glue.js")]
extern "C" {
    #[wasm_bindgen(js_name = addQa, catch)]
    async fn add_qa(msg: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize, Deserialize)]
struct AddQAArgs<'a> {
    #[serde(borrow)]
    msg: Message<'a>,
}

#[function_component(App)]
pub fn app() -> Html {
    // let args = MyGreetArgs { name: "Amin" };
    // let jsval = to_value(&args).unwrap();
    // web_sys::console::log_1(&jsval);

    let q_ref = use_node_ref();
    let a_ref = use_node_ref();

    let submit_error = use_state(|| String::new());

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

            // NOTE: Should I switch to tauri-sys?
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
        <main class="container">
            <p>{&*submit_error}</p>
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
        </main>
    }
}

// html! {
//     <main class="container">
//         <div class="row">
//             <a href="https://tauri.app" target="_blank">
//                 <img src="public/tauri.svg" class="logo tauri" alt="Tauri logo"/>
//             </a>
//             <a href="https://yew.rs" target="_blank">
//                 <img src="public/yew.png" class="logo yew" alt="Yew logo"/>
//             </a>
//         </div>
//
//         <p>{"Click on the Tauri and Yew logos to learn more."}</p>
//
//         <form class="row" onsubmit={greet}>
//             <input id="greet-input" ref={greet_input_ref} placeholder="Enter a name..." />
//             <button type="submit">{"Greet"}</button>
//         </form>
//
//         <p><b>{ &*greet_msg }</b></p>
//
//         <Inputs />
//     </main>
// }
