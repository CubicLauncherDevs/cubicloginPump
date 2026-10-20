# CubicLogin

[English](README.en.md) | **Español**

Plugin de autenticación para el servidor de Minecraft [Pumpkin](https://github.com/Pumpkin-MC/Pumpkin).

CubicLogin guarda las cuentas en una base de datos redb cifrada, protege las contraseñas con
Argon2id y mantiene a los jugadores sin autenticar en un estado seguro de "limbo" hasta que inicien
sesión o se registren.

## Características

- Hash de contraseñas con Argon2id y coste configurable.
- Base de datos de cuentas cifrada con AES-GCM (redb), con modo sin cifrado solo si se configura.
- Limitación de intentos de inicio de sesión y bloqueos temporales.
- Limbo completo para jugadores sin autenticar: chat, comandos, movimiento, interacción con bloques,
  inventario, uso de objetos y la mayoría de acciones están bloqueadas.
- Inicio de sesión automático tras un registro correcto.
- Pantalla de autenticación oscura con título de login/registro, cuenta atrás visible y expulsión
  automática al agotarse el tiempo.
- Aviso al entrar con sugerencias clicables de `/login` y `/register`.
- Mensajes en inglés y español, configurables por clave, además de idiomas personalizados.
- Configuración TOML con migración automática desde el formato JSON anterior.

## Requisitos

- Un servidor Pumpkin con soporte de plugins WASM.
- Rust con el target `wasm32-wasip2`.
- Una clave de base de datos estable de 32 bytes, proporcionada mediante una variable de entorno.

## Compilación

El repositorio fija el target en `.cargo/config.toml` y la toolchain en `rust-toolchain.toml`.

```bash
rustup target add wasm32-wasip2
cargo build --release
```

El plugin se genera en:

```text
target/wasm32-wasip2/release/cubiclogin_pump.wasm
```

Con Nix:

```bash
nix shell nixpkgs#rustup nixpkgs#gcc nixpkgs#wasm-tools \
  -c rustup run stable cargo build --release
```

## Instalación

1. Copia `cubiclogin_pump.wasm` en el directorio `plugins/` del servidor.
2. Concede los permisos de datos del plugin y proporciona la clave de cifrado.
3. Inicia el servidor.

Ejemplo de configuración de Pumpkin:

```toml
[plugins.overrides.CubicLogin]
allowed_permissions = ["fs.read.data", "fs.write.data"]

[plugins.overrides.CubicLogin.environment]
CUBICLOGIN_DB_KEY = "<64 caracteres hexadecimales>"
```

Genera una clave con:

```bash
openssl rand -hex 32
```

> **Mantén la clave de la base de datos estable.** Si se pierde o se cambia, las cuentas existentes
> no podrán descifrarse.

En el primer arranque el plugin crea:

```text
plugins/data/CubicLogin/
├── accounts.redb
├── config.toml
└── lang/
    ├── en.toml
    └── es.toml
```

## Comandos

| Comando | Descripción |
| --- | --- |
| `/register <contraseña> <confirmar_contraseña>` | Crea una cuenta. Si el registro es correcto, el jugador inicia sesión automáticamente. |
| `/login <contraseña>` | Autentica una cuenta existente. |
| `/cubiclogin reload` | Recarga la configuración y los idiomas sin reiniciar. Requiere el permiso `CubicLogin:admin`. |

Los cambios en `encryption`, `db_key_env` y `pepper_env` requieren reiniciar el servidor.

## Configuración

La configuración está en `plugins/data/CubicLogin/config.toml` y se carga al iniciar el servidor.
Consulta [docs/configuration.es.md](docs/configuration.es.md) para la referencia completa, los modos
de cifrado y las notas de migración.

```toml
language = "en"
prefix = "&b[CubicLogin]&r "
require_login_on_join = true
min_password_length = 8
max_password_length = 64
session_timeout_seconds = 60
max_failed_attempts = 5
lockout_seconds = 300
encryption = "required"
db_key_env = "CUBICLOGIN_DB_KEY"
```

## Idiomas

Los mensajes para jugadores se cargan desde `plugins/data/CubicLogin/lang/<código>.toml` y se
seleccionan con la opción `language`. Los idiomas incluidos son inglés (`en`) y español (`es`). Las
claves que falten usan el texto inglés, y los idiomas personalizados solo necesitan las claves que
quieran sobrescribir.

Consulta [docs/languages.es.md](docs/languages.es.md) para la referencia de claves, marcadores y
códigos de color.

## Seguridad

- Usa una clave de 32 bytes única y aleatoria por servidor.
- Guarda `accounts.redb` y la clave juntos en las copias de seguridad.
- El pepper opcional de Argon2 (`CUBICLOGIN_PEPPER`) añade un secreto del servidor al hash de
  contraseñas. Cambiarlo o perderlo invalida los hashes existentes.
- No uses `encryption = "off"` en servidores de producción.

## Desarrollo

```bash
cargo fmt
cargo clippy --all-targets --target x86_64-unknown-linux-gnu -- -D warnings
cargo clippy --target wasm32-wasip2 -- -D warnings
cargo test --target x86_64-unknown-linux-gnu
cargo build --release
wasm-tools validate target/wasm32-wasip2/release/cubiclogin_pump.wasm
```

## Licencia

GPL-3.0-or-later. Consulta [LICENSE](LICENSE).
