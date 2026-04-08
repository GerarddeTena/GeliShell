# `src/shell/assistant/` — Asistente IA integrado

Este módulo implementa el **asistente de inteligencia artificial** de GeliShell. Combina un modelo de lenguaje local (Qwen) con un sistema de recuperación de documentación (RAG) para responder preguntas sobre la propia shell y sobre comandos.

> 💡 Para usar el asistente escribe `geli-helpme <tu pregunta>` en el REPL, o usa `geli ask "<pregunta>"` desde la terminal.

---

## Ficheros

### `mod.rs` — `AssistantRuntime`
**¿Qué hace?** Es el **punto de entrada público** del asistente. Coordina el modelo LLM y el motor RAG en una única interfaz.

Métodos principales:
- `new(config)` — inicializa el runtime con la configuración del usuario
- `ensure_model_ready(progress)` — descarga el modelo si no existe, lo carga en memoria y notifica el progreso
- `run_parameter(parameter, filter)` — ejecuta el asistente para completar un parámetro de comando
- `run_how_to(subsystem, query)` — responde preguntas de tipo "¿cómo hago X en bash?"
- `sweep_idle_resources()` — descarga el modelo de memoria si lleva demasiado tiempo inactivo (configurable)
- `release_resources()` — libera todo: descarga el modelo y limpia la caché RAG

### `qwen.rs` — Motor del modelo de lenguaje
**¿Qué hace?** Gestiona el ciclo de vida del modelo Qwen (descarga, carga, generación de texto, descarga de memoria).

- **Descarga automática**: si el modelo no está en `~/.config/geliShell/models/`, lo descarga
- **Generación**: toma un prompt en formato ChatML y devuelve texto generado
- Soporta dos variantes configurables: `qwen-0.5b` (ligero, rápido) y `qwen-1.5b` (más capaz)

**Configuración en `config.toml`:**
```toml
[assistant]
model_variant = "qwen-0.5b"    # o "qwen-1.5b"
auto_unload_after_secs = 300   # descarga el modelo tras 5 min de inactividad
```

### `rag.rs` — Motor de recuperación (RAG)
**¿Qué hace?** Implementa **Retrieval-Augmented Generation**: antes de llamar al LLM, busca en la base de datos de documentación (`docs.db`) los fragmentos más relevantes para la consulta, y los incluye en el prompt como contexto.

- Usa `sqlite-vec` para búsqueda vectorial por similitud semántica
- `retrieve_context(query, top_k)` — devuelve los `top_k` fragmentos más relevantes
- `clear_cache()` — limpia la caché de embeddings entre consultas

**¿Dónde está `docs.db`?** En `~/.config/geliShell/docs/docs.db`. Se genera con el binario `build_docs_db` (ver `src/bin/`).

### `suggest.rs` — Construcción de prompts y parseo de respuestas
**¿Qué hace?** Actúa como capa de **traducción entre el mundo GeliShell y el LLM**:

- `build_user_action(parameter, filter)` — construye la instrucción en lenguaje natural
- `build_retrieval_query(parameter, filter)` — genera la query para buscar en RAG
- `build_chatml_prompt(action, rag_context)` — ensambla el prompt final en formato ChatML
- `build_suggestion(generated)` — parsea la respuesta cruda del LLM en un `AssistantSuggestion`
- `build_how_to_chatml_prompt(...)` — versión específica para preguntas how-to
- `parse_how_to_response(generated)` — parsea respuestas how-to en `HowToSuggestion`

### `params.rs` — Tipos de parámetros del asistente
**¿Qué hace?** Define el enum `AssistantParameter` que clasifica qué tipo de ayuda se está pidiendo (ej. completar un flag, sugerir un comando, explicar un error).

---

## Flujo de una consulta

```
Usuario: "geli-helpme ¿cómo hago un git rebase interactivo?"
             │
             ▼
      handlers/assistant.rs
             │  ensure_model_ready() → carga Qwen si no está
             │
             ▼
      AssistantRuntime::run_how_to("bash", "git rebase interactivo")
             │
             ├─ rag.retrieve_context(query, 4) → 4 fragmentos de docs.db
             │
             ├─ suggest.build_how_to_chatml_prompt(...)
             │
             ├─ qwen.generate(prompt) → "git rebase -i HEAD~3"
             │
             └─ suggest.parse_how_to_response(text) → HowToSuggestion
                          │
                          ▼
                   Se muestra en pantalla
```

---

## Para contribuidores

- Para **añadir más documentación** al RAG → añade `.md` a `docs/kb/` y regenera `docs.db`
- Para **cambiar el modelo** → edita las constantes en `qwen.rs`
- El asistente **no requiere conexión a internet** en tiempo de uso — todo es local
- El modelo se descarga solo la primera vez que se invoca
