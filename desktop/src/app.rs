use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use types::MyGreetArgs;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[function_component(App)]
pub fn app() -> Html {
    let args = MyGreetArgs { name: "Amin" };
    let jsval = to_value(&args).unwrap();
    web_sys::console::log_1(&jsval);
    let greet_input_ref = use_node_ref();

    let name = use_state(|| String::new());

    let greet_msg = use_state(|| String::new());
    {
        let greet_msg = greet_msg.clone();
        let name = name.clone();
        let name2 = name.clone();
        use_effect_with(name2, move |_| {
            web_sys::console::log_1(&"use_effect_with started".into());
            spawn_local(async move {
                if name.is_empty() {
                    return;
                }

                let args = to_value(&GreetArgs { name: &*name }).unwrap();
                // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
                let new_msg = invoke("greet", args).await.as_string().unwrap();
                greet_msg.set(new_msg);
            });

            move || {
                web_sys::console::log_1(&"Clean Up!".into());
            }
            // || {}
        });
    }

    let greet = {
        let name = name.clone();
        let greet_input_ref = greet_input_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            name.set(
                greet_input_ref
                    .cast::<web_sys::HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        })
    };

    html! {
        <main class="container">
            <div class="row">
                <div class="input-group">
                    <label>{"Question"}</label>
                    <textarea type="text" name="question" rows=10 />
                </div>
                <div class="input-group">
                    <label>{"Answer"}</label>
                    <textarea type="text" name="answer" rows=10 />
                </div>
            </div>
            <button type="submit" class="submit-button">{"Submit"}</button>
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
