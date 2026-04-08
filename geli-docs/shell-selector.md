# `src/shell/selector/` — Selector interactivo de comandos

Este módulo proporciona la **interfaz de selección interactiva** que aparece cuando GeliShell tiene múltiples traducciones posibles para un comando y necesita que el usuario elija una.

---

## ¿Cuándo aparece el selector?

Depende del `selector_mode` configurado:
- **`always`** (por defecto): aparece siempre que hay más de una sugerencia disponible
- **`auto`**: aparece solo cuando el comando no tiene una traducción exacta unívoca
- **`once`**: muestra el selector una sola vez por comando, luego recuerda la elección

**Ejemplo:**
```bash
$ list -a
# Si hay sugerencias: ["ls -a", "ls -la", "ls --all"]
# → se abre el selector modal para elegir
```

---

## Ficheros

### `mod.rs` — Trait `CommandSelector`

Define el **contrato del selector**. Cualquier implementación puede mostrar las opciones como quiera (modal, lista en línea, etc.):

```rust
pub trait CommandSelector: Send + Sync {
    fn select(&self, resolved: &ResolvedCommand) -> SelectionResult;
}

pub enum SelectionResult {
    Selected(String),   // el usuario eligió esta opción
    Cancelled,          // el usuario canceló con Esc
}
```

El diseño **Open/Closed** permite añadir nuevas presentaciones (ej. un selector fuzzy) sin modificar el código existente.

### `modal.rs` — `ModalSelector`

**¿Qué hace?** Es la implementación concreta del selector: muestra una lista numerada de sugerencias en el terminal y espera a que el usuario elija con las teclas de cursor o un número.

Controles:
- **↑/↓** — navegar entre opciones
- **Enter** — confirmar selección
- **Esc** — cancelar (el comando se ejecuta con la traducción exacta por defecto)
- **1, 2, 3…** — selección directa por número

---

## `ResolvedCommand`

El selector recibe un `ResolvedCommand` del pipeline de traducción, que contiene:
- El comando exacto recomendado
- La lista de sugerencias alternativas con su tipo (`ExactMatch`, `NativeAlias`, `NativeCommand`, `CrossPlatform`)
- La puntuación de cada sugerencia

---

## Para contribuidores

Para crear un **selector alternativo** (ej. fuzzy finder estilo fzf):
```rust
pub struct FuzzySelector;

impl CommandSelector for FuzzySelector {
    fn select(&self, resolved: &ResolvedCommand) -> SelectionResult {
        // implementa aquí el selector fuzzy
        SelectionResult::Selected(resolved.exact.clone())
    }
}
```

Luego pásalo al handler de comandos en lugar del `ModalSelector` por defecto.
