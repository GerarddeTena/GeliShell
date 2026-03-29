# `src/cli/` — Argumentos de línea de comandos

Este directorio gestiona lo que ocurre cuando se invoca `geli` con **argumentos directos** en lugar de abrirse en modo interactivo.

---

## Ficheros

### `gerisabet.rs`
**¿Qué hace?** Implementa el subcomando `geli ask "..."` que permite hacer preguntas al asistente IA directamente desde la terminal, **sin entrar en el REPL**.

**Ejemplo de uso:**
```bash
geli ask "¿cómo listo ficheros ocultos en bash?"
geli ask "how do I create a new git branch?"
```

---

## `cli.rs` (en `src/`)

El módulo raíz `src/cli.rs` actúa como **router de argumentos**. Lee `args[1..]` y despacha según el primer argumento:

| Argumento | Acción |
|-----------|--------|
| `ask <pregunta>` | Llama al asistente IA en modo one-shot |
| `--help` / `-h` | Muestra la ayuda del CLI |
| `--version` / `-v` | Muestra la versión |
| *(ninguno)* | Abre el REPL interactivo |

---

## Para contribuidores

Para añadir un nuevo subcomando CLI:
1. Añade la lógica en un nuevo archivo dentro de `src/cli/`
2. Registra el caso en `src/cli.rs` dentro de `handle_cli_args`
3. El patrón es siempre: parsear args → ejecutar acción → retornar (sin REPL)
