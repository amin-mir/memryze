use std::cmp::min;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::str;

use argon2::Argon2;
use iota_stronghold::{Client as StrongholdClient, KeyProvider, SnapshotPath, Stronghold};
use rand::rngs::OsRng;
use rand::RngCore;
use tauri::{App, AppHandle, Builder, Emitter, Manager, State};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use tracing::error;

use message::{Message, QA};

const VAULT_CLIENT: &str = "ApiKeyClient";
const VAULT_API_KEY: &str = "ApiKey";

pub type Result<T> = std::result::Result<T, String>;

struct AppStateInner {
    in_buf: Vec<u8>,
    out_buf: Vec<u8>,
    stream: Option<TcpStream>,
    vault_enc_key: KeyProvider,
    vault_snapshot_path: SnapshotPath,
    vault_srv: Stronghold,
    vault_cli: StrongholdClient,
}

impl AppStateInner {
    fn split_borrow_mut(
        &mut self,
    ) -> (
        &mut Vec<u8>,
        &mut Vec<u8>,
        &mut Option<TcpStream>,
        &mut StrongholdClient,
    ) {
        (
            &mut self.in_buf,
            &mut self.out_buf,
            &mut self.stream,
            &mut self.vault_cli,
        )
    }
}

type AppState = Mutex<AppStateInner>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .setup(|app| {
            let data_dir = get_data_dir(app);
            let salt_path = data_dir.join("salt.txt");

            let salt = if !salt_path.exists() {
                fs::create_dir_all(&data_dir).unwrap();
                let salt = generate_salt();
                fs::write(&salt_path, &salt).unwrap();
                println!("wrote salt to {}", salt_path.display());
                salt
            } else {
                let mut file = File::open(salt_path)?;
                let mut salt = [0u8; 16];
                file.read_exact(&mut salt).unwrap();
                salt
            };
            println!("salt len={}", salt.len());

            // Initialize network buffers.
            let mut in_buf = vec![0u8; 2048];
            let mut out_buf = vec![0u8; 2048];

            // Setup stronghold.
            let snapshot = SnapshotPath::from_path(data_dir.join("vault.hold"));
            let enc_key = get_vault_encryption_key(&salt)?;

            let stronghold = Stronghold::default();

            // When a snapshot file exists from before, we attempt to load a client from it,
            // read the API Key and connect to the backend already.
            let (vault_cli, stream) = if snapshot.exists() {
                stronghold.load_snapshot(&enc_key, &snapshot)?;
                let vault_cli = stronghold.load_client(VAULT_CLIENT)?;
                // let vault_cli =
                //     stronghold.load_client_from_snapshot(VAULT_CLIENT, &enc_key, &snapshot)?;

                match vault_cli.store().get(VAULT_API_KEY.as_bytes())? {
                    Some(api_key) => {
                        println!("read the api key from vault");
                        let stream = tauri::async_runtime::block_on(async {
                            let api_key = str::from_utf8(&api_key)?;
                            connect(&mut in_buf, &mut out_buf, api_key).await
                        })?;
                        (vault_cli, Some(stream))
                    }
                    None => {
                        println!("No API Key for already existing client");
                        (vault_cli, None)
                    }
                }
            } else {
                println!("client created for the first time");
                let vault_cli = stronghold.create_client("ApiKeyClient")?;
                stronghold.commit_with_keyprovider(&snapshot, &enc_key)?;
                (vault_cli, None)
            };

            let app_state = Mutex::new(AppStateInner {
                in_buf,
                out_buf,
                stream,
                vault_enc_key: enc_key,
                vault_snapshot_path: snapshot,
                vault_srv: stronghold,
                vault_cli,
            });

            app.manage(app_state);

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            is_connected,
            update_api_key,
            add_qa,
            get_quiz,
            review_qa
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_data_dir(app: &mut App) -> PathBuf {
    let build_type = std::option_env!("TAURI_BUILD").unwrap_or("dev");
    if build_type == "dev" {
        PathBuf::from(".")
    } else {
        app.handle().path().app_data_dir().unwrap()
    }
}

fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

fn get_vault_encryption_key(salt: &[u8]) -> anyhow::Result<KeyProvider> {
    let password = std::option_env!("VAULT_PASSWORD").unwrap_or("secret vault password");

    let mut encryption_key = vec![0u8; 32];
    Argon2::default().hash_password_into(password.as_bytes(), salt, &mut encryption_key)?;

    KeyProvider::try_from(encryption_key).map_err(Into::into)
}

async fn retry_connect(
    in_buf: &mut [u8],
    out_buf: &mut [u8],
    token: &str,
) -> anyhow::Result<TcpStream> {
    let bkf_max = Duration::from_secs(4);
    let mut bkf = Duration::from_millis(100);
    let mut retries = 0;

    loop {
        match connect(in_buf, out_buf, token).await {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                if retries == 20 {
                    return Err(e);
                }
                retries += 1;
            }
        }

        time::sleep(bkf).await;
        println!("#[{}] Retrying connection...", retries);
        if bkf < bkf_max {
            bkf = min(bkf * 2, bkf_max);
        }
    }
}

async fn connect(in_buf: &mut [u8], out_buf: &mut [u8], token: &str) -> anyhow::Result<TcpStream> {
    let mut stream = TcpStream::connect(get_server_addr()).await?;

    let handshake = Message::Handshake { version: 1, token };

    prot::write_msg(&mut stream, out_buf, &handshake).await?;

    let handshake_resp = prot::read_msg(&mut stream, in_buf).await?;

    let Message::HandshakeResp = handshake_resp else {
        error!(?handshake_resp, "Handshake reply has the wrong type");
        anyhow::bail!("Handshake resp has the wrong type");
    };

    Ok(stream)
}

fn get_server_addr() -> &'static str {
    return std::option_env!("SERVER_ADDR").unwrap_or("127.0.0.1:8080");
}

#[tauri::command]
async fn is_connected(state: State<'_, AppState>) -> Result<bool> {
    let state = state.lock().await;
    Ok(state.stream.is_some())
}

#[tauri::command]
async fn update_api_key(state: State<'_, AppState>, api_key: String) -> Result<()> {
    let mut state = state.lock().await;

    // Connect to server with this API Key and store the TcpStream in State.
    let (in_buf, out_buf, stream, _) = state.split_borrow_mut();

    let new_stream = retry_connect(in_buf, out_buf, &api_key)
        .await
        .map_err(|e| e.to_string())?;

    let _ = stream.insert(new_stream);

    // Store the API Key in vault and commit for later use.
    state
        .vault_cli
        .store()
        .insert(
            VAULT_API_KEY.as_bytes().to_vec(),
            api_key.as_bytes().to_vec(),
            None,
        )
        .map_err(|e| e.to_string())?;

    state
        .vault_srv
        .commit_with_keyprovider(&state.vault_snapshot_path, &state.vault_enc_key)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn add_qa(app: AppHandle, state: State<'_, AppState>, msg: Message<'_>) -> Result<()> {
    app.emit("reconnecting", ()).map_err(|e| e.to_string())?;
    let Message::AddQA { .. } = msg else {
        return Err(format!("expected AddQA, got {:?}", msg));
    };

    let mut state = state.lock().await;

    let (in_buf, out_buf, stream, vault_cli) = state.split_borrow_mut();

    let handle_resp = |resp: &Message| match resp {
        Message::AddQAResp => Ok(()),
        _ => anyhow::bail!("expected AddQAResp, got {:?}", resp),
    };
    request_reconnect(stream, in_buf, out_buf, vault_cli, &msg, handle_resp)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_quiz(state: State<'_, AppState>) -> Result<Vec<QA>> {
    let mut qas = Vec::new();

    let msg = Message::GetQuiz;

    let mut state = state.lock().await;

    let (in_buf, out_buf, stream, vault_cli) = state.split_borrow_mut();

    let handle_resp = |resp: &Message| {
        let Message::Quiz { count, qas_bytes } = resp else {
            anyhow::bail!("expected Quiz, got {:?}", resp);
        };

        prot::deser_from_bytes(qas_bytes, *count, &mut qas).map_err(Into::into)
    };
    request_reconnect(stream, in_buf, out_buf, vault_cli, &msg, handle_resp)
        .await
        .map_err(|e| e.to_string())?;

    Ok(qas)
}

#[tauri::command]
async fn review_qa(state: State<'_, AppState>, msg: Message<'_>) -> Result<()> {
    let Message::ReviewQA { .. } = msg else {
        return Err(format!("expected ReviewQA, got {:?}", msg));
    };

    let mut state = state.lock().await;

    let (in_buf, out_buf, stream, vault_cli) = state.split_borrow_mut();

    let handle_resp = |resp: &Message| match resp {
        Message::ReviewQAResp => Ok(()),
        _ => anyhow::bail!("expected ReviewQAResp, got {:?}", resp),
    };
    request_reconnect(stream, in_buf, out_buf, vault_cli, &msg, handle_resp)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// request_reconnect will send a request to the server with the provided message,
// and if it detects disconnection will attempt to re-establish the connection
// using retry_connect and tries the request one more time afterwards.
async fn request_reconnect<H>(
    stream: &mut Option<TcpStream>,
    in_buf: &mut [u8],
    out_buf: &mut [u8],
    vault_cli: &StrongholdClient,
    msg: &Message<'_>,
    mut handle: H,
) -> anyhow::Result<()>
where
    H: FnMut(&Message<'_>) -> anyhow::Result<()>,
{
    let req_res = request(stream, in_buf, out_buf, &msg).await;
    if let Ok(resp) = req_res {
        return handle(&resp);
    }

    match req_res.unwrap_err() {
        prot::Error::StreamClosed => {
            println!("Disconnect detected");
            let api_key = get_api_key(vault_cli)?;
            let new_stream = retry_connect(in_buf, out_buf, &api_key).await?;
            _ = stream.insert(new_stream);
            match request(stream, in_buf, out_buf, &msg).await {
                Ok(resp) => handle(&resp),
                Err(e) => Err(e.into()),
            }
        }
        e => Err(e.into()),
    }
}

fn get_api_key(vault_cli: &StrongholdClient) -> anyhow::Result<String> {
    let api_key = vault_cli.store().get(VAULT_API_KEY.as_bytes())?;
    let Some(api_key) = api_key else {
        anyhow::bail!("Missing API Key in vault")
    };
    String::from_utf8(api_key).map_err(Into::into)
}

async fn request<'a>(
    stream: &mut Option<TcpStream>,
    in_buf: &'a mut [u8],
    out_buf: &mut [u8],
    msg: &Message<'_>,
) -> anyhow::Result<Message<'a>, prot::Error> {
    let stream = stream.as_mut().ok_or(prot::Error::StreamClosed)?;
    prot::write_msg(stream, out_buf, &msg).await?;
    prot::read_msg(stream, in_buf).await
}
