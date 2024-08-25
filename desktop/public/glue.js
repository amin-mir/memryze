const invoke = window.__TAURI_INTERNALS__.invoke;

export async function addQa(msg) {
    console.log("inside glue code");
    return await invoke("add_qa", { msg });
}

export async function getQuiz(qas) {
    return await invoke("get_quiz", { qas });
}

export async function reviewQa(msg) {
    return await invoke("review_qa", { msg });
}
