const invoke = window.__TAURI_INTERNALS__.invoke;

export async function isConnected() {
    return await invoke("is_connected");
}

export async function updateApiKey(apiKey) {
    return await invoke("update_api_key", { apiKey })
}

export async function addQa(msg) {
    return await invoke("add_qa", { msg });
}

export async function getQuiz() {
    return await invoke("get_quiz");
}

export async function reviewQa(msg) {
    return await invoke("review_qa", { msg });
}
