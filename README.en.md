# CubicLogin

**English** | [Español](README.md)

Authentication plugin for the [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin) Minecraft server.

CubicLogin stores accounts in an encrypted redb database, hashes passwords with Argon2id, and keeps
unauthenticated players in a safe "limbo" state until they log in or register.

## Features

- Argon2id password hashing with a configurable cost.
- AES-GCM encrypted account database (redb), with a plaintext fallback only when configured.
- Login throttling and temporary lockouts.
- Full limbo for unauthenticated players: chat, commands, movement, block interactions, inventory,
  item use, and most player actions are blocked.
- Automatic login after a successful registration.
- Prompt on join with clickable `/login` and `/register` suggestions.
- English and Spanish messages, configurable per key, plus custom language files.
- TOML configuration with automatic migration from the legacy JSON format.

## Requirements

- A Pumpkin server with WASM plugin support.
- Rust with the `wasm32-wasip2` target.
- A stable 32-byte database key, provided through an environment variable.

## Build

The repository pins the target in `.cargo/config.toml` and the toolchain in `rust-toolchain.toml`.

```bash
rustup target add wasm32-wasip2
cargo build --release
```

The plugin is written to:

```text
target/wasm32-wasip2/release/cubiclogin_pump.wasm
```

With Nix:

```bash
nix shell nixpkgs#rustup nixpkgs#gcc nixpkgs#wasm-tools \
  -c rustup run stable cargo build --release
```

## Install

1. Copy `cubiclogin_pump.wasm` into the server's `plugins/` directory.
2. Grant the plugin its data permissions and provide the encryption key.
3. Start the server.

Example Pumpkin configuration:

```toml
[plugins.overrides.CubicLogin]
allowed_permissions = ["fs.read.data", "fs.write.data"]

[plugins.overrides.CubicLogin.environment]
CUBICLOGIN_DB_KEY = "<64 hex characters>"
```

Generate a key with:

```bash
openssl rand -hex 32
```

> **Keep the database key stable.** If it is lost or changed, existing accounts cannot be decrypted.

On first start the plugin creates:

```text
plugins/data/CubicLogin/
├── accounts.redb
├── config.toml
└── lang/
    ├── en.toml
    └── es.toml
```

## Commands

| Command | Description |
| --- | --- |
| `/register <password> <confirm_password>` | Create an account. A successful registration logs the player in automatically. |
| `/login <password>` | Authenticate an existing account. |
| `/cubiclogin reload` | Reload configuration and language files without restarting. Requires the `CubicLogin:admin` permission. |

Changes to `encryption`, `db_key_env`, and `pepper_env` require a server restart.

## Configuration

Configuration lives in `plugins/data/CubicLogin/config.toml` and is reloaded on server start.
See [docs/configuration.md](docs/configuration.md) for the full reference, encryption modes, and
migration notes.

```toml
language = "en"
prefix = "&b[CubicLogin]&r "
require_login_on_join = true
min_password_length = 8
max_password_length = 64
max_failed_attempts = 5
lockout_seconds = 300
encryption = "required"
db_key_env = "CUBICLOGIN_DB_KEY"
```

## Languages

Player-facing messages are loaded from `plugins/data/CubicLogin/lang/<code>.toml`, selected with the
`language` option. Built-in languages are English (`en`) and Spanish (`es`). Missing keys fall back
to English, and custom languages only need the keys they want to override.

See [docs/languages.md](docs/languages.md) for the message key reference, placeholders, and color
codes.

## Security

- Use a unique, random 32-byte key per server.
- Keep `accounts.redb` and the key backed up together.
- The optional Argon2 pepper (`CUBICLOGIN_PEPPER`) adds a server-side secret to password hashing.
  Changing or losing it invalidates existing hashes.
- Do not enable `encryption = "off"` on production servers.

## Development

```bash
cargo fmt
cargo clippy --all-targets --target x86_64-unknown-linux-gnu -- -D warnings
cargo clippy --target wasm32-wasip2 -- -D warnings
cargo test --target x86_64-unknown-linux-gnu
cargo build --release
wasm-tools validate target/wasm32-wasip2/release/cubiclogin_pump.wasm
```

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
