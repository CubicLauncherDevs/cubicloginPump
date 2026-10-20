# Idiomas

[English](languages.md) | **Español**

Los mensajes para jugadores se guardan por separado de la configuración principal:

```text
plugins/data/CubicLogin/
├── config.toml
└── lang/
    ├── en.toml
    └── es.toml
```

Selecciona un idioma en `config.toml`:

```toml
language = "en" # "en", "es" o el código de un idioma personalizado
```

Reinicia el servidor después de cambiar el idioma.

## Idiomas incluidos

| Código | Idioma |
| --- | --- |
| `en` | Inglés |
| `es` | Español |

Al iniciar, se crean `lang/en.toml` y `lang/es.toml` si no existen. Ambos archivos son TOML normal y
se pueden editar libremente. Los archivos existentes nunca se sobrescriben.

Para restaurar los textos por defecto de un idioma incluido, elimina su archivo y reinicia el
servidor.

## Idiomas personalizados

1. Copia `lang/en.toml` a `lang/<código>.toml` (por ejemplo, `lang/fr.toml`).
2. Traduce los valores. Las claves deben mantenerse igual.
3. Configura `language = "<código>"` en `config.toml`.
4. Reinicia el servidor.

Los códigos de idioma no distinguen mayúsculas y pueden contener letras, números, `_` y `-`. Si el
archivo de idioma no existe o no es válido, se usa el inglés y se muestra una advertencia en la
consola.

## Comportamiento de respaldo

- Las claves que falten en un archivo de idioma usan el texto inglés embebido.
- Las claves desconocidas se ignoran.
- Una traducción parcial es válida: solo necesitas sobrescribir los mensajes que quieras cambiar.

## Marcadores

Algunos mensajes admiten marcadores. Mantenlos en tu traducción si aparecen:

| Marcador | Valor |
| --- | --- |
| `{seconds}` | Tiempo de bloqueo restante en segundos |
| `{min}` | Longitud mínima configurada |
| `{max}` | Longitud máxima configurada |

## Códigos de color

Los mensajes admiten el formato legacy de Minecraft con `&`:

| Código | Significado |
| --- | --- |
| `&0`–`&9` | Colores |
| `&a`–`&f` | Colores |
| `&l` | Negrita |
| `&o` | Cursiva |
| `&n` | Subrayado |
| `&m` | Tachado |
| `&k` | Ofuscado |
| `&r` | Restablecer formato |
| `&x&R&R&G&G&B&B` | Color hexadecimal, por ejemplo `&x&f&c&0&0&0&0` |

El prefijo global se configura en `config.toml`, no en los archivos de idioma:

```toml
prefix = "&b[CubicLogin]&r "
```

## Texto al hacer clic y al pasar el cursor

Algunos mensajes incluyen texto interactivo:

| Clave | Interacción |
| --- | --- |
| `join_login` | Al hacer clic sugiere `/login ` |
| `join_register` | Al hacer clic sugiere `/register ` |
| `blocked` | Al hacer clic sugiere `/login ` |
| `hover_login` | Texto al pasar el cursor sobre el aviso de inicio de sesión |
| `hover_register` | Texto al pasar el cursor sobre el aviso de registro |

## Referencia de mensajes

| Clave | Marcadores | Valor por defecto (inglés) |
| --- | --- | --- |
| `join_login` | | Welcome back! Please authenticate with /login <password>. |
| `join_register` | | Welcome! Create your account with /register <password> <password>. |
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

## Ejemplo

```toml
# ============================================================
#  CubicLogin - English messages
# ============================================================
#
#  - Legacy color codes use '&' (e.g. &b, &l, &r).
#  - Placeholders: {min}, {max}, {seconds}.

# --- Join and authentication prompts ---
join_login = "&eWelcome back! &fPlease authenticate with &b/login <password>&f."
join_register = "&eWelcome! &fCreate your account with &b/register <password> <password>&f."
blocked = "&cYou must register or log in first: &b/register <password> <password> &cor &b/login <password>&c."
hover_login = "Click to fill in the login command"
hover_register = "Click to fill in the register command"

# --- Login ---
login_locked = "&cToo many failed attempts. Try again in &e{seconds}&c seconds."
login_success = "&aLogin successful."
login_failed = "&cIncorrect username or password."
```
