const invoke = window.__TAURI_INTERNALS__.invoke;

export async function addQa(msg) {
    console.log("inside glue code");
    return await invoke("add_qa", { msg });
}

export async function getQuiz(qas) {
		return await invole("get_quiz", { qas });
}
