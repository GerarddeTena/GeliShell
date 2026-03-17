> **Propósito:**
> Definir los límites operativos obligatorios para la IA de GeliShell bajo un modelo de Guardrail estricto.
> Establecer reglas de sugerencia, validación e interceptación para evitar comandos destructivos o fuera de política.

# Protocolos de Seguridad y Limitaciones de IA en GeliShell (Guardrail Operativo)

## Regla de Oro del Guardrail de IA

La IA de GeliShell debe sugerir comandos **únicamente** usando el vocabulario canónico documentado en `docs/kb/diccionario-comandos-canonicos.md` y derivado de `src/commands/commands.toml`.

Cualquier comando, flag o patrón no presente en ese diccionario se considera fuera de política.

```text
Política obligatoria:
Sugerencia válida = (comando canónico + flags canónicos traducibles)
Sugerencia inválida = cualquier cadena fuera del diccionario canónico
```

## Estructura de seguridad del Guard en GeliShell

El sistema de seguridad semántica está implementado como `CompositeGuard` en `src/shell/guard/mod.rs`.

Reglas activas por defecto:

- `RmGuard`
- `ChmodChownGuard`
- `DdGuard`
- `MkfsGuard`
- `CriticalRedirectGuard`
- `PipeExecutionGuard`
- `ForkBombGuard`

El Guard se ejecuta sobre AST (`ASTNode`), no sobre texto crudo, y bloquea antes de pasar al executor.

## Tipos de bloqueo semántico (`GuardError`)

Tipos de error relevantes para políticas de IA:

- `DestructiveFs`
- `DiskDestroyer`
- `CriticalRedirect`
- `PipeExecution`
- `ForkBomb`
- `BlacklistedCommand`
- `ForbiddenArgument`
- `RequiresConfirmation` (no fatal)

```rust
if let Err(e) = guard.check(&ast) {
    reporter.error(&e.to_string());
    continue;
}
```

## Acciones terminantemente prohibidas para la IA

Las siguientes acciones están prohibidas como salida sugerida del asistente:

- sugerir borrado recursivo forzado de rutas críticas (`rm -rf /`, equivalentes)
- sugerir operaciones destructivas de disco (`dd of=/dev/sdX`, `mkfs` inseguro)
- sugerir redirecciones críticas a archivos sensibles (`/etc/passwd`, `/etc/shadow`)
- sugerir `curl|wget` encadenado a shells (`bash`, `sh`, `zsh`, `fish`, `geli`)
- sugerir patrones de fork bomb
- sugerir binarios arbitrarios no contenidos en vocabulario canónico
- sugerir ejecución de payloads opacos sin explicación de riesgo

## Restricción de PATH y binarios arbitrarios en política de IA

La política de IA de GeliShell prohíbe sugerir ejecución de binarios no mapeados en `commands.toml`, aunque el sistema operativo pudiera resolverlos por PATH.

Esta restricción es deliberada para reducir superficie de ataque por prompt injection y expansión de comandos no auditados.

## Flujo de interceptación y validación de output de IA

El output del asistente no se ejecuta automáticamente.

Flujo operativo actual:

1. La IA produce texto por `Reporter`.
2. El resultado se muestra al usuario como recomendación.
3. Solo si el usuario introduce un comando en el prompt, ese comando pasa por la ruta normal de validación.
4. La ruta normal ejecuta `Lexer -> Parser -> Builtins -> Guard -> Pipeline -> Executor`.

```text
AI output (texto) -> usuario decide -> input REPL
input REPL -> tokenización -> AST -> Guard -> traducción -> ejecución
```

## Saneamiento e invariantes previas a ejecución

Invariantes de seguridad aplicadas por la shell:

- límite de entrada: `MAX_INPUT_BYTES = 65536` en lexer
- parseo estructural obligatorio antes de evaluación semántica
- guard semántico obligatorio antes de ejecutar procesos
- separación de modo TTY y no-TTY para evitar contención de stdin
- timeout configurable en executor para cortar procesos prolongados

## Política de inserción al buffer de entrada

Política vigente:

- la IA no inyecta comandos directamente al buffer del prompt
- el usuario mantiene control explícito de aceptación
- toda ejecución real requiere paso por canal REPL normal validado

Si se implementa inserción automática en el futuro, debe pasar por la misma cadena de validación (`Lexer/Parser/Guard`) antes de confirmar en buffer.

## Protocolo de respuesta segura para el LLM local

La IA debe responder con estructura segura:

- comando canónico permitido
- traducción esperada por subsistema
- justificación breve de seguridad
- alternativa no destructiva si detecta riesgo

```text
Formato recomendado:
1) Comando canónico permitido
2) Traducción por subsistema
3) Riesgo y precondiciones
4) Confirmación requerida para acciones sensibles
```
