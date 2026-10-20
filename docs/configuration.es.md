# Configuración

[English](configuration.md) | **Español**

CubicLogin lee su configuración desde:

```text
plugins/data/CubicLogin/config.toml
```

El archivo se carga una vez al iniciar el plugin. Reinicia el servidor después de modificarlo.

## Migración desde JSON

Las versiones anteriores usaban `config.json`. Al iniciar:

1. Si existe `config.toml`, se usa ese archivo.
2. Si solo existe `config.json`, se convierte a `config.toml` y el original se renombra a
   `config.json.bak`.
3. Si no existe ninguno, se genera un `config.toml` comentado con los valores por defecto.

Cuando una nueva versión del plugin añade opciones, las claves que falten se añaden automáticamente.
Los valores y comentarios existentes se conservan.

## Recarga en caliente

Usa `/cubiclogin reload` para volver a leer `config.toml` y el archivo de idioma activo sin
reiniciar el servidor. El comando requiere el permiso `CubicLogin:admin` (nivel de op 4 por defecto).

Se aplica de inmediato:

- `language` y `prefix`
- `require_login_on_join`
- los límites de longitud de contraseña y la política de bloqueo
- los parámetros de Argon2 para los nuevos hashes de contraseña

Las sesiones autenticadas se conservan al recargar.

Los cambios en `encryption`, `db_key_env` o `pepper_env` se rechazan con un mensaje que indica que
hay que reiniciar. Si el archivo de configuración no es válido, se mantiene la configuración
anterior.

## Referencia

| Clave | Tipo | Valor por defecto | Descripción |
| --- | --- | --- | --- |
| `language` | string | `"en"` | Código de idioma para los mensajes de los jugadores. Incluidos: `en`, `es`. |
| `prefix` | string | `"&b[CubicLogin]&r "` | Prefijo añadido a todos los mensajes para jugadores. Admite códigos de color con `&`. |
| `require_login_on_join` | boolean | `true` | Requiere autenticación después de entrar. |
| `min_password_length` | integer | `8` | Longitud mínima de la contraseña. |
| `max_password_length` | integer | `64` | Longitud máxima de la contraseña. |
| `session_timeout_seconds` | integer | `300` | Reservado para futuras sesiones; actualmente sin uso. |
| `max_failed_attempts` | integer | `5` | Intentos de inicio de sesión fallidos antes de un bloqueo temporal. `0` desactiva los bloqueos. |
| `lockout_seconds` | integer | `300` | Duración del bloqueo en segundos. |
| `argon2_memory_kib` | integer | `19456` | Coste de memoria de Argon2id en KiB. |
| `argon2_iterations` | integer | `2` | Iteraciones de Argon2id. |
| `argon2_parallelism` | integer | `1` | Paralelismo de Argon2id. |
| `encryption` | string | `"required"` | Modo de cifrado de la base de datos: `required`, `auto` u `off`. |
| `db_key_env` | string | `"CUBICLOGIN_DB_KEY"` | Variable de entorno que contiene la clave de la base de datos. |
| `pepper_env` | string | `"CUBICLOGIN_PEPPER"` | Variable de entorno opcional con un pepper de Argon2. |

## Ejemplo

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

## Modos de cifrado

| Modo | Comportamiento |
| --- | --- |
| `required` | La variable de entorno indicada por `db_key_env` debe existir y contener una clave hexadecimal válida de 64 caracteres. Si no, el arranque falla. |
| `auto` | Usa la clave cuando está presente y es válida; en caso contrario guarda los registros sin cifrar y registra una advertencia. |
| `off` | Nunca cifra los registros almacenados. No recomendado fuera de pruebas locales. |

La clave debe ser de 32 bytes codificada como 64 caracteres hexadecimales:

```bash
openssl rand -hex 32
```

```bash
od -An -tx1 -N32 /dev/urandom | tr -d ' \n'
```

> Cambiar la clave de la base de datos hace que las cuentas existentes no puedan leerse. Guarda la
> clave y `accounts.redb` juntos en las copias de seguridad.

## Pepper de Argon2

Si `pepper_env` indica una variable de entorno no vacía, su valor se usa como secreto de Argon2.
Es opcional, pero añade un secreto del servidor al hash de contraseñas. Cambiar o perder el pepper
invalida los hashes de contraseñas existentes.

## Permisos y entorno

CubicLogin necesita acceso de lectura y escritura a su carpeta de datos:

```toml
[plugins.overrides.CubicLogin]
allowed_permissions = ["fs.read.data", "fs.write.data"]

[plugins.overrides.CubicLogin.environment]
CUBICLOGIN_DB_KEY = "<64 caracteres hexadecimales>"
# CUBICLOGIN_PEPPER = "<pepper opcional>"
```

La sección `environment` funciona incluso cuando el servidor está configurado con
`inherit_env = false`.
