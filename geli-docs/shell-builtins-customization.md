# `src/shell/builtins/customization/` — Comandos personalizados del usuario

Este módulo carga y ejecuta los **comandos personalizados** que el usuario define en su `config.toml`.

---

## ¿Qué son los comandos personalizados?

Son atajos o alias avanzados que el usuario puede definir sin necesidad de editar los TOML de comandos del proyecto. Se definen en la configuración personal:

```toml
# ~/.config/geliShell/config.toml
[customization]
[[customization.custom_commands]]
name = "dev"
template = "cd ~/projects && code ."

[[customization.custom_commands]]
name = "deploy"
template = "cargo build --release && scp target/release/mi-app server:/opt/"
```

Una vez definidos, se pueden usar directamente en el REPL como si fueran comandos normales:
```bash
dev      # ejecuta: cd ~/projects && code .
deploy   # ejecuta: cargo build --release && scp ...
```

---

## `mod.rs`
**¿Qué hace?** Gestiona el registro y ejecución de comandos personalizados:
- Los comandos personalizados se registran dinámicamente desde la configuración al arrancar
- Cuando el usuario escribe el nombre de un comando personalizado, este módulo expande la plantilla (`template`) y la devuelve para que el executor la procese
- Si la configuración cambia (desde el menú de config), los comandos se recargan automáticamente

---

## Para usuarios: cómo configurar comandos personalizados

1. Abre el menú de configuración con **Ctrl+Alt+S** o escribe `geli-config`
2. Ve a la sección "Customization"
3. Añade tus comandos, o edita `~/.config/geliShell/config.toml` directamente

**Limitaciones actuales:**
- El `template` se ejecuta tal cual — no interpola variables de la shell en tiempo de expansión (usa `$HOME` si necesitas rutas absolutas)
- No soportan argumentos posicionales por ahora (el comando se ejecuta con el template completo)

---

## Para contribuidores

- Para añadir **interpolación de argumentos** (ej. `template = "git checkout {branch}"`) → extiende `mod.rs` con lógica de sustitución de placeholders
- Los comandos personalizados se cargan en `BuiltinRegistry` como entradas del `CommandMap`
