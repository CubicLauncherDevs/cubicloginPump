# Languages

**English** | [Español](languages.es.md)

Player-facing messages are stored separately from the main configuration:

```text
plugins/data/CubicLogin/
├── config.toml
└── lang/
    ├── en.toml
    └── es.toml
```

Select a language in `config.toml`:

```toml
language = "en" # "en", "es", or a custom language code
```

Restart the server after changing the language, or apply the change with `/cubiclogin reload`.

## Built-in languages

| Code | Language |
| --- | --- |
| `en` | English |
| `es` | Spanish |

On startup, `lang/en.toml` and `lang/es.toml` are created if they do not exist. Both files are plain
TOML and can be edited freely. Existing files are never overwritten.

To restore the default text of a built-in language, delete its file and run `/cubiclogin reload` (or
restart the server).

## Custom languages

1. Copy `lang/en.toml` to `lang/<code>.toml` (for example, `lang/fr.toml`).
2. Translate the values. Keys must stay unchanged.
3. Set `language = "<code>"` in `config.toml`.
4. Run `/cubiclogin reload` or restart the server.

Language codes are case-insensitive and may contain letters, numbers, `_`, and `-`. A missing or
invalid language file falls back to English with a warning in the console.

## Fallback behavior

- Missing keys in a language file fall back to the embedded English text.
- Unknown keys are ignored.
- A partial translation is valid: only override the messages you want to change.

## Placeholders

Some messages support placeholders. Keep them in your translation if they are present:

| Placeholder | Value |
| --- | --- |
| `{seconds}` | Remaining lockout time in seconds |
| `{time}` | Remaining authentication time, formatted as `m:ss` |
| `{min}` | Minimum configured length |
| `{max}` | Maximum configured length |

## Color codes

Messages support legacy Minecraft formatting with `&`:

| Code | Meaning |
| --- | --- |
| `&0`–`&9` | Colors |
| `&a`–`&f` | Colors |
| `&l` | Bold |
| `&o` | Italic |
| `&n` | Underlined |
| `&m` | Strikethrough |
| `&k` | Obfuscated |
| `&r` | Reset formatting |
| `&x&R&R&G&G&B&B` | Hex color, for example `&x&f&c&0&0&0&0` |

The global prefix is configured in `config.toml`, not in the language files:

```toml
prefix = "&b[CubicLogin]&r "
```

## Click and hover text

Some messages include interactive text:

| Key | Interaction |
| --- | --- |
| `join_login` | Click suggests `/login ` |
| `join_register` | Click suggests `/register ` |
| `blocked` | Click suggests `/login ` |
| `hover_login` | Hover text for the login prompt |
| `hover_register` | Hover text for the register prompt |

## Message reference

| Key | Placeholders | Default (English) |
| --- | --- | --- |
| `join_login` | | Welcome back! Please authenticate with /login <password>. |
| `join_register` | | Welcome! Create your account with /register <password> <password>. |
| `auth_title_login` | | &b&lSIGN IN |
| `auth_title_register` | | &a&lCREATE YOUR ACCOUNT |
| `auth_title_fallback` | | &b&lCUBICLOGIN |
| `auth_subtitle_login` | | &7Use &f/login <password> &7to continue. |
| `auth_subtitle_register` | | &7Use &f/register <password> <password> &7to continue. |
| `auth_subtitle_fallback` | | &7Sign in or create an account to continue. |
| `auth_countdown` | `{time}` | &eTime remaining: &f{time} |
| `auth_timeout_kick` | | &cAuthentication time expired. Rejoin and use &f/login &cor &f/register&c. |
| `blocked` | | You must register or log in first: /register <password> <password> or /login <password>. |
| `hover_login` | | Click to fill in the login command |
| `hover_register` | | Click to fill in the register command |
| `only_players_login` | | Only players can use /login. |
| `only_players_register` | | Only players can use /register. |
| `login_empty_password` | | The password cannot be empty. |
| `login_locked` | `{seconds}` | Too many failed attempts. Try again in {seconds} seconds. |
| `login_success` | | Login successful. |
| `login_failed` | | Incorrect username or password. |
| `register_mismatch` | | The passwords do not match. |
| `register_success` | | Registration successful. You are now logged in. |
| `account_exists` | | A CubicLogin account already exists for this player. |
| `account_save_error` | | Could not save the account. |
| `username_too_short` | `{min}` | The username must contain at least {min} characters. |
| `username_too_long` | `{max}` | The username must contain at most {max} characters. |
| `username_invalid` | | The username may only contain letters, numbers and underscores. |
| `password_too_short` | `{min}` | The password must contain at least {min} characters. |
| `password_too_long` | `{max}` | The password must contain at most {max} characters. |
| `storage_unavailable` | | CubicLogin storage is unavailable. |
| `database_error` | | Could not access the account database. |
| `password_error` | | Could not process the password. |
| `state_unavailable` | | CubicLogin state is unavailable. |
| `reload_success` | | CubicLogin reloaded. |
| `reload_failed` | `{error}` | CubicLogin reload failed: {error} |
| `reload_restart_required` | `{fields}` | Restart required for: {fields} |
| `reload_usage` | | Usage: /cubiclogin reload |

## Example

```toml
# ============================================================
#  CubicLogin - English messages
# ============================================================
#
#  - Legacy color codes use '&' (e.g. &b, &l, &r).
#  - Placeholders: {min}, {max}, {seconds}, {time}.

# --- Join and authentication prompts ---
join_login = "&eWelcome back! &fPlease authenticate with &b/login <password>&f."
join_register = "&eWelcome! &fCreate your account with &b/register <password> <password>&f."
auth_title_login = "&b&lSIGN IN"
auth_title_register = "&a&lCREATE YOUR ACCOUNT"
auth_title_fallback = "&b&lCUBICLOGIN"
auth_subtitle_login = "&7Use &f/login <password> &7to continue."
auth_subtitle_register = "&7Use &f/register <password> <password> &7to continue."
auth_subtitle_fallback = "&7Sign in or create an account to continue."
auth_countdown = "&eTime remaining: &f{time}"
auth_timeout_kick = "&cAuthentication time expired. Rejoin and use &f/login &cor &f/register&c."
blocked = "&cYou must register or log in first: &b/register <password> <password> &cor &b/login <password>&c."
hover_login = "Click to fill in the login command"
hover_register = "Click to fill in the register command"

# --- Login ---
login_locked = "&cToo many failed attempts. Try again in &e{seconds}&c seconds."
login_success = "&aLogin successful."
login_failed = "&cIncorrect username or password."
```
