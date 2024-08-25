use yew::prelude::*;

use crate::quiz::QuizComponent;
use crate::submit::SubmitComponent;

#[derive(PartialEq)]
enum NavbarSelected {
    Submit,
    Quiz,
}

#[function_component(App)]
pub fn app() -> Html {
    let navbar_selected = use_state(|| NavbarSelected::Quiz);

    let select_submit = {
        let navbar_selected = navbar_selected.clone();

        Callback::from(move |_| {
            web_sys::console::log_1(&"submit selected".into());
            navbar_selected.set(NavbarSelected::Submit);
        })
    };

    let select_quiz = {
        let navbar_selected = navbar_selected.clone();

        Callback::from(move |_| {
            web_sys::console::log_1(&"quiz selected".into());
            navbar_selected.set(NavbarSelected::Quiz);
        })
    };

    let navbar_li_class = {
        let navbar_selected = navbar_selected.clone();

        move |nbs: NavbarSelected| -> &'static str {
            if *navbar_selected == nbs {
                "navbar-selected"
            } else {
                ""
            }
        }
    };

    html! {
        <main class="container">
            <nav>
                <ul class="navbar">
                    <li class={classes!(navbar_li_class(NavbarSelected::Submit))} onclick={select_submit}>{"Submit"}</li>
                    <li class={classes!(navbar_li_class(NavbarSelected::Quiz))} onclick={select_quiz}>{"Quiz"}</li>
                </ul>
            </nav>

            if *navbar_selected == NavbarSelected::Submit {
                <SubmitComponent />
            } else {
                <QuizComponent />
            }
        </main>
    }
}
