use yew::prelude::*;

mod app;
mod auth;
mod commands;
mod quiz;
mod submit;

use app::App;
use auth::Auth;

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<Main>::new().render();
}

#[function_component(Main)]
fn main() -> Html {
    let connected = use_state(|| false);

    let onconnect = {
        let connected = connected.clone();

        move |_| {
            connected.set(true);
        }
    };

    html! {
        if *connected {
            <App />
        } else {
            <Auth {onconnect} />
        }
    }
}
