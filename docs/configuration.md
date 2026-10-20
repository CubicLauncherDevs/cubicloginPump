# Configuration

**English** | [Español](configuration.es.md)

CubicLogin reads its configuration from:

```text
plugins/data/CubicLogin/config.toml
```

The file is loaded once when the plugin starts. Restart the server after changing it.

## Migration from JSON

Older versions used `config.json`. On startup:

1. If `config.toml` exists, it is used.
2. If only `config.json` exists, it is converted to `config.toml`, and the original is renamed to
   `config.json.bak`.
3. If neither exists, a commented default `config.toml` is generated.

When a new plugin version adds options, missing keys are appended automatically. Existing values and
comments are preserved.

## Reloading

Use `/cubiclogin reload` to re-read `config.toml` and the active language file without restarting
the server. The command requires the `CubicLogin:admin` permission (op level 4 by default).

Applied immediately:

- `language` and `prefix`
- `require_login_on_join`
- password length limits and lockout policy
- Argon2 parameters for new password hashes

Authenticated sessions are preserved across a reload.

Changes to `encryption`, `db_key_env`, or `pepper_env` are rejected with a restart-required message.
If the configuration file is invalid, the previous configuration stays active.

## Reference

| Key | Type | Default | Description |
| --- | --- | --- | --- |
| `language` | string | `"en"` | Language code for player messages. Built-in: `en`, `es`. |
| `prefix` | string | `"&b[CubicLogin]&r "` | Prefix added to every player-facing message. Supports `&` color codes. |
| `require_login_on_join` | boolean | `true` | Require authentication after joining. |
| `min_password_length` | integer | `8` | Minimum password length. |
| `max_password_length` | integer | `64` | Maximum password length. |
| `session_timeout_seconds` | integer | `300` | Reserved for future session handling; currently unused. |
| `max_failed_attempts` | integer | `5` | Failed logins before a temporary lockout. `0` disables lockouts. |
| `lockout_seconds` | integer | `300` | Lockout duration in seconds. |
| `argon2_memory_kib` | integer | `19456` | Argon2id memory cost in KiB. |
| `argon2_iterations` | integer | `2` | Argon2id iterations. |
| `argon2_parallelism` | integer | `1` | Argon2id parallelism. |
| `encryption` | string | `"required"` | Database encryption mode: `required`, `auto`, or `off`. |
| `db_key_env` | string | `"CUBICLOGIN_DB_KEY"` | Environment variable containing the database key. |
| `pepper_env` | string | `"CUBICLOGIN_PEPPER"` | Optional environment variable with an Argon2 pepper. |

## Example

```toml
# ============================================================
#  CubicLogin configuration
# ============================================================

# ------------------------------------------------------------
#  Language and messages
# ------------------------------------------------------------
language = "en"
prefix = "&b[CubicLogin]&r "

# ------------------------------------------------------------
#  Authentication
# ------------------------------------------------------------
require_login_on_join = true
min_password_length = 8
max_password_length = 64
session_timeout_seconds = 300
max_failed_attempts = 5
lockout_seconds = 300

# ------------------------------------------------------------
#  Password hashing (Argon2id)
# ------------------------------------------------------------
argon2_memory_kib = 19456
argon2_iterations = 2
argon2_parallelism = 1

# ------------------------------------------------------------
#  Database
# ------------------------------------------------------------
encryption = "required"
db_key_env = "CUBICLOGIN_DB_KEY"
pepper_env = "CUBICLOGIN_PEPPER"
```

## Encryption modes

| Mode | Behavior |
| --- | --- |
| `required` | The environment variable named by `db_key_env` must exist and contain a valid 64-character hex key. Startup fails otherwise. |
| `auto` | Uses the key when it is present and valid; otherwise stores records without encryption and logs a warning. |
| `off` | Never encrypts stored records. Not recommended outside local testing. |

The key must be 32 bytes encoded as 64 hexadecimal characters:

```bash
openssl rand -hex 32
```

```bash
od -An -tx1 -N32 /dev/urandom | tr -d ' \n'
```

> Changing the database key makes existing accounts unreadable. Back up the key and
> `accounts.redb` together.

## Argon2 pepper

If `pepper_env` names a non-empty environment variable, its value is used as the Argon2 secret.
This is optional, but it adds a server-side secret to password hashing. Changing or losing the pepper
invalidates existing password hashes.

## Permissions and environment

CubicLogin needs read and write access to its plugin data folder:

```toml
[plugins.overrides.CubicLogin]
allowed_permissions = ["fs.read.data", "fs.write.data"]

[plugins.overrides.CubicLogin.environment]
CUBICLOGIN_DB_KEY = "<64 hex characters>"
# CUBICLOGIN_PEPPER = "<optional pepper>"
```

The `environment` override works even when the server is configured with `inherit_env = false`.
