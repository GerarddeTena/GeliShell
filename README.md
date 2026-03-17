# GeliShell: instalación y uso de la base de datos vectorial `docs.db` (RAG local)

## Qué es `docs.db` y para qué sirve en GeliShell

`docs.db` es un artefacto SQLite precomputado que contiene la documentación vectorizada de GeliShell para casos de recuperación semántica (RAG).

Se genera en máquina de build/desarrollo y se distribuye como base de conocimiento portable para acelerar búsquedas y reducir cómputo en cliente.

## Componentes del proyecto relacionados con la BDD vectorial

- `src/bin/build_docs_db.rs`: herramienta interna de ingesta, chunking, embeddings e inserción SQL.
- `docs/kb/*.md`: fuente documental que se transforma en chunks vectoriales.
- `src/shell/assistant/mod.rs`: orquestador del asistente local.
- `src/shell/assistant/rag.rs`: recuperación textual local por archivos.
- `src/shell/assistant/qwen.rs`: runtime de modelo local y eventos de bootstrap.

Estado actual importante:

- El asistente runtime actual recupera contexto desde archivos en `~/.config/geliShell/docs/`.
- `docs.db` queda preparado como índice vectorial preconstruido para integración RAG vectorial local o distribución de conocimiento.

## De qué se nutre la `docs.db`

La base se alimenta de Markdown en `docs/kb` (por ejemplo):

- `diccionario-comandos-canonicos.md`
- `protocolos-seguridad-limitaciones-guardrail.md`
- `arquitectura-interna-subsistemas-contexto-general.md`

Pipeline de datos:

```text
docs/kb/*.md -> chunking por #/## -> texto chunk -> embedding (768) -> SQLite (metadata + vector)
```

## Requisitos de instalación en la máquina que construye la BDD

### 1) Rust/Cargo

Necesitas toolchain Rust para ejecutar el binario interno:

```powershell
cargo --version
```

### 2) Motor de embeddings local (`nomic-embed-text`)

El binario usa endpoint local compatible con Ollama (`/api/embed` y fallback `/api/embeddings`).

```powershell
ollama pull nomic-embed-text
ollama serve
```

### 3) Extensión `sqlite-vec` (solo en máquina de build o en runtime vectorial local)

Descarga la build de `sqlite-vec` adecuada a tu SO/arquitectura desde su release oficial y conserva la ruta del binario (`vec0.dll` en Windows).

```powershell
$env:GELI_SQLITE_VEC_PATH = "C:\tools\sqlite-vec\vec0.dll"
```

## Ejecución paso a paso para construir `docs.db`

### Ver ayuda del binario

```powershell
cargo run --bin build_docs_db -- --help
```

### Build estándar

```powershell
$env:GELI_SQLITE_VEC_PATH = "C:\tools\sqlite-vec\vec0.dll"
cargo run --bin build_docs_db -- `
  --docs-dir docs\kb `
  --db-path docs.db `
  --batch-size 16 `
  --model nomic-embed-text `
  --ollama-url http://127.0.0.1:11434
```

### Qué hace el comando anterior

1. Lee todos los `.md` de `docs\kb`.
2. Fragmenta por cabeceras `#` y `##`.
3. Crea chunks con `id` determinista (SHA-256), `source`, `text`.
4. Genera embeddings en batch.
5. Valida que cada embedding tenga dimensión `768` (assert de seguridad).
6. Inserta metadatos y vectores en transacción atómica.

## Esquema de la base de datos

La herramienta crea:

```sql
CREATE TABLE docs_metadata (
  id TEXT PRIMARY KEY,
  fuente TEXT NOT NULL,
  texto_completo TEXT NOT NULL
);

CREATE VIRTUAL TABLE vec_docs USING vec0(
  id TEXT,
  embedding float[768]
);
```

Inserción:

```sql
BEGIN TRANSACTION;
-- INSERT docs_metadata
-- INSERT vec_docs
COMMIT;
```

## Tolerancia a fallos y validaciones de seguridad del pipeline

- Si falla un batch de embeddings, el pipeline hace fallback chunk-a-chunk.
- Si un chunk falla, se registra `WARN` y se continúa con el resto.
- Si un embedding no mide 768, se descarta y no se inserta.
- Si no se genera ningún embedding válido, el proceso termina con error explícito.

## Ejemplo de verificación rápida de contenido

```sql
SELECT COUNT(*) AS total_chunks FROM docs_metadata;
SELECT fuente, COUNT(*) AS chunks_por_fuente
FROM docs_metadata
GROUP BY fuente
ORDER BY chunks_por_fuente DESC;
```

## Cómo usar `docs.db` en un flujo RAG

Flujo recomendado:

1. Construir `docs.db` en entorno controlado de desarrollo/CI.
2. Versionar o adjuntar `docs.db` como artefacto.
3. Distribuir `docs.db` al cliente/aplicación.
4. Resolver recuperación semántica local (si hay `sqlite-vec` en runtime) o en servicio backend.
5. Pasar chunks recuperados al LLM local como contexto.

## Ventajas de exportar `docs.db` preconstruida en vez de entregar el `.dll` al cliente

### Ventajas operativas

- Menor fricción de instalación en cliente (no compilar embeddings ni reconstruir índice).
- Artefacto determinista y versionable (`docs.db` por release).
- Menos variabilidad por entorno (SO/arquitectura/toolchains).
- Arranque más rápido: el conocimiento ya está vectorizado.

### Ventajas de seguridad y cumplimiento

- Reduce la necesidad de ejecutar binarios nativos de extensión en el cliente final.
- Menor superficie de ataque asociada a carga dinámica de DLLs.
- Separación clara entre entorno de build (interno) y entorno de consumo (cliente).

### Diferencia clave frente a distribuir directamente `sqlite-vec.dll`

Distribuir solo la DLL obliga al cliente a:

- instalar y cargar extensión nativa correctamente,
- resolver compatibilidad de ABI/arquitectura,
- y construir/gestionar su propio índice vectorial.

Distribuir `docs.db` ya construida traslada ese costo al pipeline de build, que es más controlable y repetible.

## Nota de arquitectura sobre runtime actual

Actualmente, el módulo `assistant/rag.rs` recupera documentos desde filesystem (`~/.config/geliShell/docs/`).

La `docs.db` vectorial queda como base precomputada para integración vectorial local o para servir recuperación desde un componente dedicado.

## Troubleshooting

### Error: `sqlite-vec extension could not be loaded`

- Verifica ruta de la extensión.
- Exporta `GELI_SQLITE_VEC_PATH`.
- Comprueba arquitectura (x64/x86) compatible con tu SQLite/rusqlite.

### Error: `no embeddings were generated successfully`

- Verifica que Ollama esté activo.
- Verifica que exista el modelo `nomic-embed-text`.
- Revisa conectividad al `--ollama-url`.

### Error de dimensión de embedding

- Usa un modelo que devuelva 768 dimensiones para mantener compatibilidad del esquema `float[768]`.
