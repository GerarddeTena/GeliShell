> **Propósito:**
> Proveer contexto arquitectónico de alto nivel sobre cómo interactúan TUI, configuración, historial y autocompletado.
> Mantener una visión de diseño orientada a RAG sin dependencia de detalles de línea o implementación tutorial.

# Arquitectura Interna y Subsistemas de GeliShell (Contexto General)

## Topología de componentes del sistema

GeliShell organiza su runtime en módulos coordinados:

- **Core REPL:** ciclo principal de entrada y despacho
- **TUI Layer:** menús interactivos en alternate screen
- **Config Layer:** persistencia de estado en `config.toml`
- **History Layer:** historial de comandos (`history.txt`) y rutas `g_jump`
- **Completion Layer:** sugerencias en línea y ghost text
- **Translation Layer:** mapeo canónico -> comando nativo por subsistema
- **Execution Layer:** proceso async con streaming, timeout y modo TTY
- **Assistant Layer:** RAG local + bootstrap de modelo GGUF

## Flujo de datos de una interacción de usuario

Flujo nominal de entrada:

1. Usuario escribe en prompt (raw mode).
2. `repl_input` gestiona edición, cursor, historial y sugerencia ghost.
3. Core REPL clasifica trigger especial o comando normal.
4. Comando normal sigue parseo, validación de seguridad, traducción y ejecución.
5. Resultados y eventos se muestran por `Reporter`.

```text
Input usuario -> ReplInputAction -> Core REPL -> Guard/Pipeline/Executor -> Output
```

## Interacción entre TUI y estado persistente

La capa TUI no es aislada; se alimenta de `ShellConfig` y devuelve cambios estructurados.

Relación principal:

- `geli-config-me` abre un panel visual
- el panel devuelve `UpdatedVisual(VisualConfig)` o cierre
- el core aplica visuales y persiste `config.toml`

El estado visual activo se reutiliza inmediatamente para prompt, colores de terminal y ghost text.

## Modelo de configuración unificado (`config.toml`)

`ShellConfig` concentra múltiples dominios:

- comportamiento del selector
- override de subsistema
- políticas de ejecución
- configuración visual
- comandos custom
- ajustes del asistente local

El diseño permite que la shell arranque con defaults seguros si falla parseo de TOML.

## Historial persistente y memoria de uso

GeliShell mantiene dos memorias independientes:

- **historial de prompt:** `history.txt` para comandos cronológicos
- **historial de `g_jump`:** `g_history.toml` para rutas con scoring de frecencia

Ambas memorias alimentan UX:

- navegación `Up/Down`
- sugerencias predictivas por prefijo
- autocompletado de rutas en `g <prefijo>`

## Motor de autocompletado y priorización

La capa de completions aplica prioridad de fuentes:

1. historial reciente del usuario
2. rutas de `g_jump` cuando el prefijo empieza con `g `
3. pool de comandos nativos/canónicos/custom

El texto sugerido se renderiza como ghost text y se acepta con `Tab` o `Right`.

```text
Prefijo -> history match -> g_jump match -> command pool match
```

## Arquitectura de subsistemas de comando

El traductor mantiene una separación explícita:

- **Comando canónico:** nombre abstracto de GeliShell
- **Comando nativo:** equivalente por subsistema (`bash`, `zsh`, `fish`, `powershell`, `cmd`)

La detección de subsistema usa prioridad:

1. `GELI_SUBSYSTEM`
2. detección por entorno
3. default por plataforma

Esta estrategia evita acoplar comandos al sistema operativo de forma rígida.

## Integración de TUI con alternate screen y raw mode

Paneles principales (`help`, `config`, `assistant`) comparten contrato visual:

- activar raw mode
- entrar a alternate screen
- renderizar siempre desde `(0,0)`
- limpiar frame completo por iteración
- restaurar terminal al salir

Este contrato mantiene la shell limpia y consistente tras cerrar paneles.

## Arquitectura del asistente local (RAG + modelo)

El subsistema de asistente combina:

- bootstrap de modelo GGUF en `~/.config/geliShell/models/`
- recuperación documental local desde `~/.config/geliShell/docs/`
- menú de parámetros predefinidos con filtro reactivo

El pipeline del asistente es:

1. verificar/cargar modelo
2. recuperar contexto local por RAG
3. generar respuesta
4. liberar recursos al finalizar interacción

## Principio de no bloqueo y continuidad de UX

GeliShell prioriza no bloquear el loop de interacción:

- I/O persistente en async (`tokio::fs`)
- generación del asistente en tareas separadas
- streaming concurrente de stdout/stderr en executor
- separación de modo TTY para procesos interactivos

El objetivo de diseño es latencia estable en prompt aun con tareas pesadas en segundo plano lógico.
[sha256-4a188102020e9c9530b687fd6400f775c45e90a0d7baafe65bd0a36963fbb7ba](../../../.ollama/models/blobs/sha256-4a188102020e9c9530b687fd6400f775c45e90a0d7baafe65bd0a36963fbb7ba)
## Contrato de compatibilidad para componentes futuros

Reglas de evolución arquitectónica:

- nuevas capacidades deben integrarse como módulo aislado y registrable
- el flujo REPL debe conservar checkpoints de seguridad (`Guard`)
- cualquier sugerencia automatizada debe conservar dependencia del diccionario canónico
- cambios de UI deben preservar alternate screen + restauración limpia

Este contrato asegura consistencia entre extensiones futuras y comportamiento operativo actual.
