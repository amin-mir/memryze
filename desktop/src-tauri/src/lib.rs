use std::fs::{self, File};
use std::io::Read;
use std::str;

use argon2::Argon2;
use iota_stronghold::{Client as StrongholdClient, KeyProvider, SnapshotPath, Stronghold};
use rand::rngs::OsRng;
use rand::RngCore;
use tauri::{Builder, Manager, State};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::error;
use zeroize::Zeroizing;

use message::{Message, QA};

const ADDR: &str = "127.0.0.1:8080";
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
    fn split_borrow_mut(&mut self) -> (&mut Vec<u8>, &mut Vec<u8>, Option<&mut TcpStream>) {
        (&mut self.in_buf, &mut self.out_buf, self.stream.as_mut())
    }
}

type AppState = Mutex<AppStateInner>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .setup(|app| {
            let data_dir = app.handle().path().app_data_dir().unwrap();
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
            let mut in_buf = vec![0u8; 512];
            let mut out_buf = vec![0u8; 512];

            // Setup stronghold.
            let snapshot = SnapshotPath::from_path(data_dir.join("vault.hold"));
            let enc_key = get_vault_encryption_key(&salt)?;

            println!("13");
            let stronghold = Stronghold::default();

            // When a snapshot file exists from before, we attempt to load a client from it,
            // read the API Key and connect to the backend already.
            println!("1");
            let (vault_cli, stream) = if snapshot.exists() {
                stronghold.load_snapshot(&enc_key, &snapshot)?;
                let vault_cli = stronghold.load_client(VAULT_CLIENT)?;
                // let vault_cli =
                //     stronghold.load_client_from_snapshot(VAULT_CLIENT, &enc_key, &snapshot)?;

                println!("2");
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

fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

fn get_vault_encryption_key(salt: &[u8]) -> anyhow::Result<KeyProvider> {
    let password = std::option_env!("VAULT_PASSWORD").unwrap_or("secret vault password");

    let mut encryption_key = vec![0u8; 32];
    println!("11");
    Argon2::default().hash_password_into(password.as_bytes(), salt, &mut encryption_key)?;

    println!("12");
    KeyProvider::try_from(Zeroizing::new(encryption_key)).map_err(Into::into)
}

async fn connect(in_buf: &mut [u8], out_buf: &mut [u8], token: &str) -> anyhow::Result<TcpStream> {
    let mut stream = TcpStream::connect(ADDR).await?;

    let handshake = Message::Handshake { version: 1, token };

    prot::write_msg(&mut stream, out_buf, &handshake).await?;

    let handshake_resp = prot::read_msg(&mut stream, in_buf).await?;

    let Message::HandshakeResp = handshake_resp else {
        error!(?handshake_resp, "Handshake reply has the wrong type");
        anyhow::bail!("Handshake resp has the wrong type");
    };

    Ok(stream)
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
    let new_stream = {
        let (in_buf, out_buf, _stream) = state.split_borrow_mut();

        connect(in_buf, out_buf, &api_key)
            .await
            .map_err(|e| e.to_string())?
    };

    let _ = state.stream.insert(new_stream);

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
async fn add_qa(state: State<'_, AppState>, msg: Message<'_>) -> Result<()> {
    let Message::AddQA { .. } = msg else {
        return Err(format!("expected AddQA, got {:?}", msg));
    };

    let mut state = state.lock().await;

    let (in_buf, out_buf, stream) = state.split_borrow_mut();
    let Some(stream) = stream else {
        return Err("Not connected to server".to_owned());
    };

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
async fn get_quiz(state: State<'_, AppState>) -> Result<Vec<QA>> {
    let mut qas = Vec::new();

    let msg = Message::GetQuiz;

    let mut state = state.lock().await;

    let (in_buf, out_buf, stream) = state.split_borrow_mut();
    let Some(stream) = stream else {
        return Err("Not connected to server".to_owned());
    };

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
    let Some(stream) = stream else {
        return Err("Not connected to server".to_owned());
    };

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
