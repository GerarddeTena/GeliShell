# `src/shell/builtins/` — Comandos integrados de la shell

Los **builtins** son comandos que GeliShell ejecuta **directamente**, sin lanzar un proceso externo. Son necesarios porque ciertos comandos (como `cd`) deben modificar el estado interno del proceso de la shell, algo que un proceso hijo no puede hacer.

---

## Estructura

```
builtins/
├── mod.rs           ← BuiltinRegistry + trait Builtin
├── cd.rs            ← Cambio de directorio
├── clear.rs         ← Limpia la pantalla
├── exit.rs          ← Sale de la shell
├── export.rs        ← Define variables de entorno
├── unset.rs         ← Elimina variables de entorno
├── source.rs        ← Ejecuta un script en el contexto actual
├── history.rs       ← Muestra/limpia el historial de comandos
├── gerisabet.rs     ← Builtin del asistente IA
├── g_jump/          ← Navegación inteligente de directorios
└── customization/   ← Comandos personalizados del usuario
```

---

## `mod.rs` — El registro de builtins

Define dos piezas clave:

### Trait `Builtin`
Contrato que todo builtin debe implementar:
```rust
pub trait Builtin: Send + Sync {
    fn name(&self) -> &'static str;   // nombre del comando
    fn execute(&self, args: &[String], reporter: &dyn Reporter) -> BuiltinResult;
}
```

### Enum `BuiltinResult`
Lo que un builtin puede devolver:
- `Handled` — el comando se procesó correctamente, continúa el REPL
- `NotABuiltin` — no es un builtin, pásalo al translator/executor
- `Exit(i32)` — termina el proceso con este código de salida

### `BuiltinRegistry`
Colección de todos los builtins registrados. El REPL llama a `try_execute(ast, reporter)` y el registro encuentra y ejecuta el builtin correcto.

---

## Comandos disponibles

### `cd` — Cambiar directorio
```bash
cd /ruta/al/directorio
cd ..
cd ~
cd -          # vuelve al directorio anterior
```
Actualiza `PWD` y `OLDPWD` en el entorno. Registra la visita en el historial de `g`.

### `clear` — Limpiar pantalla
```bash
clear
```
Limpia la terminal. También se activa con **Ctrl+L**.

### `exit` — Salir
```bash
exit
exit 0        # código de salida específico
```
También se activa con **Ctrl+D**.

### `export` — Variables de entorno
```bash
export MI_VAR=valor
export PATH=$PATH:/nueva/ruta
```
Define variables de entorno para el proceso actual y sus hijos.

### `unset` — Eliminar variables
```bash
unset MI_VAR
```

### `source` — Ejecutar script
```bash
source ~/.bashrc
source ./mi-script.sh
```
Ejecuta el archivo indicado en el contexto de la shell actual.

### `history` — Historial de comandos
```bash
history           # muestra todo el historial
history --clear   # limpia el historial
```

### `gerisabet` — Asistente IA (builtin)
```bash
gerisabet ¿cómo listo ficheros ocultos?
```
Alias builtin para invocar el asistente desde el REPL.

### `g` — Navegación inteligente
Ver documentación completa en [`g_jump/README.md`](g_jump/README.md).

---

## Para contribuidores: añadir un nuevo builtin

1. Crea un nuevo archivo, p.ej. `alias.rs`
2. Implementa el trait `Builtin`:
```rust
pub struct AliasBuiltin;

impl Builtin for AliasBuiltin {
    fn name(&self) -> &'static str { "alias" }

    fn execute(&self, args: &[String], reporter: &dyn Reporter) -> BuiltinResult {
        // tu lógica aquí
        BuiltinResult::Handled
    }
}
```
3. Registra el builtin en `mod.rs` dentro de `BuiltinRegistry::new()`:
```rust
Box::new(alias::AliasBuiltin),
```
4. Declara el módulo en `mod.rs`: `pub mod alias;`
5. Escribe tests unitarios usando `SilentReporter`
