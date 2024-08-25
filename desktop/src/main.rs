mod app;
mod commands;
mod quiz;
mod submit;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
