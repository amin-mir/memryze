use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/public/glue.js")]
extern "C" {
    #[wasm_bindgen(js_name = isConnected, catch)]
    pub async fn is_connected() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = updateApiKey, catch)]
    pub async fn update_api_key(api_key: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = addQa, catch)]
    pub async fn add_qa(msg: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = getQuiz, catch)]
    pub async fn get_quiz() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = reviewQa, catch)]
    pub async fn review_qa(msg: JsValue) -> Result<JsValue, JsValue>;
}
