use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/public/glue.js")]
extern "C" {
    #[wasm_bindgen(js_name = addQa, catch)]
    pub async fn add_qa(msg: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = getQuiz, catch)]
    pub async fn get_quiz(msg: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = reviewQa, catch)]
    pub async fn review_qa(msg: JsValue) -> Result<JsValue, JsValue>;
}
