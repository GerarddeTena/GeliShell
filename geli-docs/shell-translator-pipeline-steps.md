# `src/shell/translator/pipeline/steps/` — Pasos de traducción

Cada archivo de este directorio implementa **un paso específico** de la cadena de traducción. Los pasos se ejecutan en secuencia, y cada uno transforma el `TranslationContext` un poco más hasta producir el comando nativo final.

---

## Pasos y su responsabilidad

### 1. `node_decomposer.rs` — `NodeDecomposer`

**Responsabilidad**: Es el **único lugar** en todo el código que hace `match` sobre `ASTNode`. Descompone el árbol sintáctico en una lista plana de `CommandFragment`, preservando los operadores entre ellos.

```
ASTNode::And(
  Command("list"),
  Command("clear")
)
→ [
    CommandFragment { name: "list", operator: Some(And) },
    CommandFragment { name: "clear", operator: None }
  ]
```

También maneja pipelines (`|`), secuencias (`;`), operadores OR (`||`) y procesos en background (`&`).

### 2. `command_resolver.rs` — `CommandResolver`

**Responsabilidad**: Para cada fragmento, busca el nombre del comando en el `CommandMap`.

Estrategias (en orden):
1. **Lookup canónico directo**: `"list"` → encontrado, es un comando canónico ✓
2. **Reverse lookup**: `"ls"` no es canónico, pero es la traducción bash de `"list"` → normaliza a `"list"` ✓
3. **Pass-through**: `"git"` no está en el mapa → se pasa tal cual (como comando nativo del OS)

Cuando encuentra por reverse lookup, emite un trace en el reporter: `"reverse lookup: ls → list"`.

### 3. `flag_resolver.rs` — `FlagResolver`

**Responsabilidad**: Traduce los flags del comando al subsistema activo.

```
Comando: list --all
Subsistema: PowerShell
Flag canónico "--all" → flag.powershell = "-Force"
→ args: ["-Force"]
```

Si un flag no tiene traducción para el subsistema, se pasa tal cual (pass-through). Esto permite que flags específicos de una herramienta funcionen sin necesidad de registrarlos todos.

### 4. `variable_expander.rs` — `VariableExpander`

**Responsabilidad**: Convierte las referencias a variables de entorno a la sintaxis del subsistema.

```
Token::Variable("HOME")
  → bash/zsh/fish: "$HOME"
  → powershell:    "$env:HOME"
  → cmd:           "%HOME%"
```

Esto permite escribir `$HOME` en GeliShell y que funcione correctamente en cualquier subsistema.

### 5. `subsystem_mapper.rs` — `SubsystemMapper`

**Responsabilidad**: Es el **paso final**. Toma cada fragmento ya resuelto y produce el string nativo del subsistema.

Proceso:
1. Consulta `Subsystem::entry(translations)` para obtener el `exact` del subsistema
2. Combina el comando exacto con los args ya procesados
3. Ensambla todos los fragmentos con sus operadores (`&&`, `||`, `|`, `;`, `&`)
4. Genera el `ResolvedCommand` con el exact y las sugerencias para el selector

También maneja el operator syntax específico por subsistema:
- PowerShell: `&&` → `&&`, `;` → `;`, `&` → `Start-Process`
- Cmd: `&&` → `&`, `||` → `&` (cmd no tiene `&&` real)

---

## `mod.rs`
Re-exporta los cinco pasos para que `pipeline/mod.rs` los importe limpiamente.

---

## Ejemplo de transformación completa

```
Input:  "list --all | search foo"
Subsistema: PowerShell

Paso 1 (NodeDecomposer):
  [Fragment("list", args=["--all"], op=Pipe), Fragment("search", args=["foo"])]

Paso 2 (CommandResolver):
  [Fragment("list" → resolved, op=Pipe), Fragment("search" → resolved)]

Paso 3 (FlagResolver):
  [Fragment("list", args=["-Force"], op=Pipe), Fragment("search", args=["foo"])]

Paso 4 (VariableExpander):
  sin variables, sin cambios

Paso 5 (SubsystemMapper):
  "Get-ChildItem -Force | Select-String foo"
```

---

## Para contribuidores

- Cada paso recibe `&mut TranslationContext` y `&dyn Reporter`
- Los pasos no se comunican entre sí directamente — todo pasa por el contexto
- Usa `reporter.info(...)` para emitir trazas de lo que estás haciendo (ayudan al debug)
- Escribe tests con `CommandMap` cargado desde TOML inline (usa `load_from_str`)
