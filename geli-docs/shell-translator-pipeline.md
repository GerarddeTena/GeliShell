# `src/shell/translator/pipeline/` — El pipeline de traducción

El pipeline es el **orquestador de la traducción**. Toma un `ASTNode` (el árbol sintáctico del comando) y lo convierte en un string listo para ejecutar en el subsistema nativo, pasándolo por una serie de pasos ordenados.

---

## Estructura

```
pipeline/
├── mod.rs       ← TranslationPipeline (orquestador)
├── context.rs   ← TranslationContext (estado compartido entre pasos)
├── step.rs      ← Trait TranslationStep + StepResult
└── steps/       ← Implementaciones de cada paso
```

---

## `mod.rs` — `TranslationPipeline`

**¿Qué hace?** Define y ejecuta la secuencia de pasos de traducción en orden.

Pasos por defecto (en orden de ejecución):
1. `NodeDecomposer` — descompone el AST en fragmentos de comandos
2. `CommandResolver` — busca cada comando en el mapa canónico
3. `FlagResolver` — traduce los flags del comando
4. `VariableExpander` — expande variables de entorno a la sintaxis del subsistema
5. `SubsystemMapper` — convierte cada fragmento al comando nativo final

**Punto de entrada:**
```rust
pipeline.run(&ast_node, &reporter) → Result<String, TranslationError>
```

**Modo traza** (solo en debug builds):
```rust
pipeline.run_with_trace(&ast_node, &reporter) → (String, Vec<StepSnapshot>)
```
Los snapshots muestran el estado del contexto después de cada paso, útil para depurar.

**Modo con comando resuelto** (para el selector):
```rust
pipeline.run_resolving(&ast_node, &reporter) → (String, Option<ResolvedCommand>)
```
Devuelve también el `ResolvedCommand` del primer fragmento para que el REPL pueda abrir el selector modal si es necesario.

---

## `context.rs` — `TranslationContext`

**¿Qué hace?** Es el **estado compartido** que fluye entre todos los pasos del pipeline. Cada paso puede leer y modificar el contexto.

Campos principales:
- `fragments: Vec<CommandFragment>` — los fragmentos de comandos en construcción
- `subsystem: &Subsystem` — el subsistema destino
- `map: &CommandMap` — el mapa de comandos para lookups
- `output: Option<String>` — si un paso produce output final, lo almacena aquí
- `snapshots: Vec<StepSnapshot>` — historial de estados (solo debug)

`CommandFragment` representa un fragmento de comando en construcción:
- `name` — nombre del comando
- `args` — argumentos ya procesados
- `operator` — operador que lo une con el siguiente (`&&`, `||`, `|`, `;`)
- `resolved` — el `ResolvedCommand` si ya fue resuelto

---

## `step.rs` — Trait `TranslationStep`

Contrato que cada paso del pipeline debe implementar:
```rust
pub trait TranslationStep: Send + Sync {
    fn name(&self) -> &'static str;
    fn process(&self, ctx: &mut TranslationContext, reporter: &dyn Reporter)
        -> Result<StepResult, PipelineError>;
}

pub enum StepResult {
    Continue,           // el paso terminó, continúa con el siguiente
    Done(String),       // el paso produjo output final, no hace falta continuar
}
```

`PipelineError` puede ser:
- `Fatal(String)` — el pipeline no puede continuar
- `Degraded(String)` — hay un resultado parcial disponible (se puede mostrar con advertencia)

---

## Para contribuidores

Para añadir un **nuevo paso al pipeline**:
1. Crea un archivo en `steps/`
2. Implementa `TranslationStep`
3. Añade el paso en `TranslationPipeline::with_resolver()` en el orden correcto
4. El pipeline no hace break early salvo que un paso devuelva `Done` o `Fatal`

> ⚠️ El único lugar donde se hace `match` sobre `ASTNode` es `NodeDecomposer`. Todos los demás pasos trabajan con `CommandFragment` — respeta esta restricción.
