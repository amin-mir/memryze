## MEMRYZE
Are you learning a new language? Want an efficient way for memorizing the countless new words, phrases
and sentences you encounter every day? Give this a try!

#### Run Desktop
To run in dev mode:

```bash
cargo tauri dev
```

To build in release mode:
```bash
SERVER_ADDR="server-addr" TAURI_BUILD="dev/release" VAULT_PASSWORD="pswd" cargo tauri build
```

Make sure to pass `SERVER_ADDR`, `TAURI_BUILD` and `VAULT_PASSWORD` environment variables.

## Download and Install

You can download the latest version of the app from the [releases page](https://github.com/amin-mir/memryze/releases).
