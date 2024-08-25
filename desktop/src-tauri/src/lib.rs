use tauri::{Builder, Manager, State};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use message::{Message, QA};

const ADDR: &str = "127.0.0.1:8080";

pub type Result<T> = std::result::Result<T, String>;

struct AppStateInner {
    in_buf: Vec<u8>,
    out_buf: Vec<u8>,
    stream: TcpStream,
}

impl AppStateInner {
    fn split_borrow_mut(&mut self) -> (&mut Vec<u8>, &mut Vec<u8>, &mut TcpStream) {
        (&mut self.in_buf, &mut self.out_buf, &mut self.stream)
    }
}

type AppState = Mutex<AppStateInner>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .setup(|app| {
            let mut in_buf = vec![0u8; 512];
            let mut out_buf = vec![0u8; 512];

            let stream =
                tauri::async_runtime::block_on(async { connect(&mut in_buf, &mut out_buf).await })?;

            let app_state = Mutex::new(AppStateInner {
                stream,
                in_buf,
                out_buf,
            });

            app.manage(app_state);

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![add_qa, get_quiz, review_qa])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn connect(in_buf: &mut [u8], prim_out_buf: &mut [u8]) -> Result<TcpStream> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    let mut stream = TcpStream::connect(ADDR).await.map_err(|e| e.to_string())?;

    let handshake = Message::Handshake { version: 1 };

    prot::write_msg(&mut stream, prim_out_buf, &handshake)
        .await
        .map_err(|e| e.to_string())?;

    let handshake_reply = prot::read_msg(&mut stream, in_buf)
        .await
        .map_err(|e| e.to_string())?;

    let Message::Handshake { version } = handshake_reply else {
        error!(?handshake_reply, "Handshake reply has the wrong type");
        return Err("Handshake reply has the wrong type".into());
    };

    info!(version, "Received handshake from server");

    Ok(stream)
}

#[tauri::command]
async fn add_qa(state: State<'_, AppState>, msg: Message<'_>) -> Result<()> {
    let Message::AddQA { .. } = msg else {
        return Err(format!("expected AddQA, got {:?}", msg));
    };

    let mut state = state.lock().await;

    let (in_buf, out_buf, stream) = state.split_borrow_mut();
    match prot::write_msg(stream, out_buf, &msg).await {
        Ok(_) => (),
        Err(e) => return Err(e.to_string()),
    }

    let resp = match prot::read_msg(stream, in_buf).await {
        Ok(msg) => msg,
        Err(e) => return Err(e.to_string()),
    };

    let Message::AddQAResp = resp else {
        return Err(format!("expected AddQAResp, got {:?}", resp));
    };
    Ok(())
}

#[tauri::command]
async fn get_quiz(state: State<'_, AppState>, mut qas: Vec<QA>) -> Result<Vec<QA>> {
    qas.clear();

    let msg = Message::GetQuiz;

    let mut state = state.lock().await;

    let (in_buf, out_buf, stream) = state.split_borrow_mut();
    match prot::write_msg(stream, out_buf, &msg).await {
        Ok(_) => (),
        Err(e) => return Err(e.to_string()),
    }

    let resp = match prot::read_msg(stream, in_buf).await {
        Ok(msg) => msg,
        Err(e) => return Err(e.to_string()),
    };

    let Message::Quiz { count, qas_bytes } = resp else {
        return Err(format!("expected Quiz, got {:?}", resp));
    };

    match prot::deser_from_bytes(qas_bytes, count, &mut qas) {
        Ok(_) => (),
        Err(e) => return Err(e.to_string()),
    };

    Ok(qas)
}

#[tauri::command]
async fn review_qa(state: State<'_, AppState>, msg: Message<'_>) -> Result<()> {
    let Message::ReviewQA { .. } = msg else {
        return Err(format!("expected ReviewQA, got {:?}", msg));
    };

    let mut state = state.lock().await;

    let (in_buf, out_buf, stream) = state.split_borrow_mut();
    match prot::write_msg(stream, out_buf, &msg).await {
        Ok(_) => (),
        Err(e) => return Err(e.to_string()),
    }

    let resp = match prot::read_msg(stream, in_buf).await {
        Ok(msg) => msg,
        Err(e) => return Err(e.to_string()),
    };

    let Message::ReviewQAResp = resp else {
        return Err(format!("expected ReviewQAResp, got {:?}", resp));
    };

    Ok(())
}
