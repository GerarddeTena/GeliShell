# `src/shell/i18n/` — Internacionalización

Este módulo gestiona las **traducciones de todos los mensajes del sistema** que ve el usuario. Permite que GeliShell muestre su interfaz en el idioma preferido del usuario.

---

## Idiomas soportados

| Código | Idioma | Archivo |
|--------|--------|---------|
| `en` | Inglés (por defecto) | `locales/en.toml` |
| `es` | Español | `locales/es.toml` |

---

## `mod.rs` — Funcionamiento

### Inicialización
Al arrancar, GeliShell detecta el idioma y lo inicializa:
```rust
let lang = detect_language(&config.behavior.language);
init_i18n(&lang);
```

### Detección de idioma (prioridad decreciente)
1. Variable de entorno `GELISHELL_LANG` → ej. `GELISHELL_LANG=es`
2. Campo `language` en `config.toml` → `[behavior] language = "es"`
3. Variable `$LANG` del sistema (solo en Unix) → ej. `LANG=es_ES.UTF-8`
4. Por defecto: inglés

### Uso en código
El macro `t!` es la forma principal de obtener traducciones:

```rust
// Sin parámetros
t!("repl.goodbye")
// → "¡Hasta luego!" (en español)

// Con parámetros interpolados
t!("executor.spawning", command = "ls", subsystem = "bash")
// → "Ejecutando: ls [bash]"
```

También disponibles como funciones directas:
```rust
// String simple
let msg = translate("repl.goodbye");

// Con parámetros
let msg = t_with("error.cd_failed", &[("path", "/ruta"), ("error", "no existe")]);
```

### Fallback automático
Si una clave no existe en el idioma activo, busca en inglés. Si tampoco existe en inglés, devuelve la propia clave como texto (facilita detectar traducciones faltantes).

---

## Estructura de los archivos de localización

Los archivos `locales/en.toml` y `locales/es.toml` usan TOML con tablas anidadas. Las claves se aplanan con puntos:

```toml
# locales/es.toml
[repl]
goodbye = "¡Hasta luego!"
input_error = "Error de entrada: {error}"

[builtin.cd]
error = "cd: {path}: {error}"

[executor]
spawning = "Ejecutando: {command} [{subsystem}]"
finished = "Proceso terminado con código {code}"
```

Esto genera las claves:
- `repl.goodbye`
- `repl.input_error`
- `builtin.cd.error`
- `executor.spawning`
- `executor.finished`

---

## Para contribuidores: añadir un nuevo idioma

1. Crea `locales/fr.toml` (o el código ISO del idioma)
2. Copia `locales/en.toml` como base y traduce los valores
3. Registra el nuevo idioma en `supported_languages()`:
```rust
pub fn supported_languages() -> &'static [&'static str] {
    &["en", "es", "fr"]   // añade "fr"
}
```
4. Añade el caso en `load_locale()`:
```rust
"fr" => parse_locale(LOCALE_FR),
```
5. Incluye el archivo con `include_str!`:
```rust
static LOCALE_FR: &str = include_str!("../../../locales/fr.toml");
```

---

## Para contribuidores: añadir una nueva clave de traducción

1. Añade la clave en `locales/en.toml` (siempre primero el inglés)
2. Añade la traducción en `locales/es.toml`
3. Usa `t!("tu.nueva.clave")` en el código

Los tests de `i18n/mod.rs` verifican que los archivos de localización parsean correctamente y que las claves críticas existen.
