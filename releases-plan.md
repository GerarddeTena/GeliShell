# GeliShell — Plan de Releases en GitHub

## Índice

1. [Comparativa de estrategias](#1-comparativa-de-estrategias)
2. [Estrategia recomendada: Tags en el repo actual](#2-estrategia-recomendada-tags-en-el-repo-actual)
3. [Qué debe contener cada release](#3-qué-debe-contener-cada-release)
4. [El problema de docs.db en CI](#4-el-problema-de-docsdb-en-ci)
5. [Workflow de GitHub Actions](#5-workflow-de-github-actions)
6. [Proceso manual para la primera release](#6-proceso-manual-para-la-primera-release)
7. [Convención de tags y ramas](#7-convención-de-tags-y-ramas)
8. [Checklist previa a cada release](#8-checklist-previa-a-cada-release)

---

## 1. Comparativa de estrategias

| Estrategia | Pros | Contras | Veredicto |
|---|---|---|---|
| **Tags en el repo actual** (`GelarddeTena/GeliShell`) | Sin fragmentación, historial unificado, bootstrap ya apunta aquí | Ninguno relevante | ✅ **Recomendado** |
| **Repo separado solo para releases** (ej. `GeliShell/releases`) | Historial de releases limpio y separado | Doble mantenimiento, bootstrap.rs debe apuntar a otro repo, confusión para contribuidores | ❌ Descartado |
| **Rama `releases` en el mismo repo** | Aisla artefactos del árbol de código | Las branches no son el mecanismo correcto para distribuir binarios; GitHub Releases es exactamente para eso | ❌ Descartado |
| **cargo-dist** | Automatización completa de firma y distribución | Añade complejidad antes de tener una release estable; evaluar en v0.3+ | ⏳ Futuro (P5) |

**Conclusión:** GitHub Releases nativo sobre el repo actual, disparado por un tag `v*.*.*` en `main`.

---

## 2. Estrategia recomendada: Tags en el repo actual

El flujo completo es:

```
dev  →  PR  →  main  →  git tag v0.1.0  →  push tag  →  GitHub Actions  →  Release
```

1. Todo el desarrollo ocurre en `dev` (ya es el flujo actual).
2. Cuando hay una versión lista, se hace merge a `main` mediante PR.
3. Se crea y pushea un tag semántico sobre `main`.
4. El workflow de release (`release.yml`) se dispara automáticamente.
5. El workflow compila la matriz de plataformas, empaqueta, genera checksums y crea el GitHub Release.

**Por qué `main` y no `dev`:**  
Los tags de release deben apuntar a código estable. `dev` es la rama de trabajo activo. `main` es el estado publicable. Esto ya es el modelo que sugiere el `.github/workflows/lint.yml` existente (protege `main` y `dev`).

---

## 3. Qué debe contener cada release

### Assets obligatorios

| Asset | Descripción |
|---|---|
| `geli-windows-x86_64.zip` | `geli.exe` + `gerisabet.exe` para Windows 64-bit |
| `geli-linux-x86_64.tar.gz` | `geli` + `gerisabet` para Linux x86_64 |
| `geli-linux-aarch64.tar.gz` | `geli` + `gerisabet` para Linux ARM64 (Raspberry Pi 4/5, Ampere) |
| `geli-macos-x86_64.tar.gz` | `geli` + `gerisabet` para Intel Mac |
| `geli-macos-aarch64.tar.gz` | `geli` + `gerisabet` para Apple Silicon |
| `checksums.txt` | SHA-256 de **todos** los assets anteriores + docs.db |

### Assets opcionales (pero muy recomendados)

| Asset | Descripción |
|---|---|
| `docs.db` | Base RAG pre-generada. Si presente, `bootstrap.rs` la descarga automáticamente al primer arranque. Si ausente, el core funciona; el asistente RAG queda desactivado hasta que el usuario genere su propia `docs.db`. |

### Formato `checksums.txt`

`bootstrap.rs` espera el formato estándar GNU:
```
<sha256hex>  <filename>
```
(dos espacios entre hash y nombre). Ejemplo:
```
a3f1...  geli-windows-x86_64.zip
b7c2...  geli-linux-x86_64.tar.gz
9de4...  docs.db
```

> **CRÍTICO:** El asset de `docs.db` debe llamarse exactamente `docs.db` — así lo busca `bootstrap.rs` en `download_docs_db_if_absent()`.

---

## 4. El problema de docs.db en CI

`docs.db` se genera con:
```bash
cargo run --bin build_docs_db --features dev-tools
```
Este binario necesita:
- **Ollama corriendo** en `http://127.0.0.1:11434`
- **Modelo de embeddings descargado** (`nomic-embed-text` por defecto)
- Los archivos Markdown de `geli-docs/`

Esto hace **inviable** generar `docs.db` en un runner estándar de GitHub Actions.

### Opciones para gestionar docs.db en releases

| Opción | Cuándo usarla |
|---|---|
| **A) Build local + upload manual** | ✅ Para v0.1.0 y mientras `geli-docs/` no cambia frecuentemente. Genera `docs.db` localmente y lo añades manualmente a la release desde la UI de GitHub. |
| **B) Self-hosted runner con Ollama** | Cuando tengas un servidor propio o VM donde puedas instalar Ollama. El workflow detecta el runner por label y corre `build_docs_db` allí. |
| **C) Docs.db versionada en repo separado** | Si `geli-docs/` crece mucho. Mantienes un repo `GeliShell-docs` con su propio CI que publica `docs.db` en sus releases. `bootstrap.rs` apunta allí. |

**Recomendación para v0.1.0:** Opción A. Es seguro, predecible y no requiere infraestructura adicional.

---

## 5. Workflow de GitHub Actions

Crea el archivo `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - "v[0-9]+.[0-9]+.[0-9]+"   # dispara en v0.1.0, v1.2.3, etc.
      - "v[0-9]+.[0-9]+.[0-9]+-*" # dispara en v0.1.0-beta.1, etc.

permissions:
  contents: write   # necesario para crear GitHub Releases y subir assets

jobs:
  build:
    name: Build (${{ matrix.target }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            archive: zip
            asset_name: geli-windows-x86_64

          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            archive: tar.gz
            asset_name: geli-linux-x86_64

          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
            archive: tar.gz
            asset_name: geli-linux-aarch64
            cross: true   # usa cross-rs para cross-compilation

          - target: x86_64-apple-darwin
            os: macos-13   # Intel Mac runner
            archive: tar.gz
            asset_name: geli-macos-x86_64

          - target: aarch64-apple-darwin
            os: macos-latest   # Apple Silicon runner
            archive: tar.gz
            asset_name: geli-macos-aarch64

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.target }}

      # Solo para cross-compilation (Linux aarch64)
      - name: Install cross
        if: matrix.cross == true
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: Build (native)
        if: matrix.cross != true
        run: cargo build --release --locked --target ${{ matrix.target }}

      - name: Build (cross)
        if: matrix.cross == true
        run: cross build --release --locked --target ${{ matrix.target }}

      # ── Empaquetar ────────────────────────────────────────────
      - name: Package (zip — Windows)
        if: matrix.archive == 'zip'
        shell: pwsh
        run: |
          $dir = "${{ matrix.asset_name }}"
          New-Item -ItemType Directory $dir
          Copy-Item "target/${{ matrix.target }}/release/geli.exe"      "$dir/"
          Copy-Item "target/${{ matrix.target }}/release/gerisabet.exe" "$dir/"
          Compress-Archive -Path "$dir/*" -DestinationPath "${{ matrix.asset_name }}.zip"

      - name: Package (tar.gz — Unix)
        if: matrix.archive == 'tar.gz'
        run: |
          dir="${{ matrix.asset_name }}"
          mkdir "$dir"
          cp "target/${{ matrix.target }}/release/geli"      "$dir/"
          cp "target/${{ matrix.target }}/release/gerisabet" "$dir/"
          tar -czf "${{ matrix.asset_name }}.tar.gz" "$dir/"

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.asset_name }}
          path: ${{ matrix.asset_name }}.${{ matrix.archive }}
          retention-days: 1

  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts/

      - name: Flatten artifacts
        run: find artifacts/ -type f | xargs -I{} mv {} ./

      # ── Generar checksums.txt ──────────────────────────────────
      - name: Generate checksums
        run: |
          sha256sum geli-*.zip geli-*.tar.gz > checksums.txt
          # Si docs.db fue subido manualmente antes del tag, inclúyelo:
          [[ -f docs.db ]] && sha256sum docs.db >> checksums.txt
          cat checksums.txt

      # ── Crear Release en GitHub ────────────────────────────────
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ github.ref_name }}
          name: "GeliShell ${{ github.ref_name }}"
          draft: false
          prerelease: ${{ contains(github.ref_name, '-') }}
          generate_release_notes: true
          files: |
            geli-windows-x86_64.zip
            geli-linux-x86_64.tar.gz
            geli-linux-aarch64.tar.gz
            geli-macos-x86_64.tar.gz
            geli-macos-aarch64.tar.gz
            checksums.txt
```

### Notas del workflow

- **`softprops/action-gh-release`** es la acción más usada para crear releases con assets. Alternativa: `gh release create` (CLI de GitHub, viene en todos los runners).
- **`prerelease: ${{ contains(github.ref_name, '-') }}`** marca automáticamente `v0.1.0-beta.1` como pre-release y `v0.1.0` como release estable.
- **`cross-rs`** es la forma más robusta de compilar para `aarch64-unknown-linux-gnu` desde un runner `ubuntu-latest`. No requiere Docker extra; `cross` lo maneja.
- **`generate_release_notes: true`** rellena las notas automáticamente desde los commits convencionales. Como ya usáis `commitlint`, esto funciona muy bien.

---

## 6. Proceso manual para la primera release (v0.1.0)

Para la v0.1.0 no es estrictamente necesario que el workflow esté listo. Puedes hacer el primer release de forma manual y luego automatizar desde v0.2.0.

### Paso a paso

```bash
# 1. Asegúrate de estar en main con todo mergeado
git checkout main
git pull origin main

# 2. Actualiza la versión en Cargo.toml
#    version = "0.1.0"  (ya está)

# 3. Compila en release para todas las plataformas disponibles
cargo build --release --locked

# 4. (Opcional pero recomendado) Genera docs.db localmente
#    Requiere: ollama serve + ollama pull nomic-embed-text
cargo run --bin build_docs_db --features dev-tools

# 5. Empaqueta los binarios
mkdir -p dist/geli-windows-x86_64
cp target/release/geli.exe         dist/geli-windows-x86_64/
cp target/release/gerisabet.exe    dist/geli-windows-x86_64/
cd dist && zip -r geli-windows-x86_64.zip geli-windows-x86_64/ && cd ..

# 6. Genera checksums
cd dist
sha256sum geli-windows-x86_64.zip > checksums.txt
# Añade los demás artefactos si compilaste en otras plataformas
cd ..

# 7. Crea y pushea el tag
git tag -a v0.1.0 -m "feat: first public release — bootstrap + cargo-install ready"
git push origin v0.1.0

# 8. Crea la release desde gh CLI
gh release create v0.1.0 \
  dist/geli-windows-x86_64.zip \
  dist/checksums.txt \
  --title "GeliShell v0.1.0" \
  --notes "First release. See README for install instructions." \
  --latest

# 9. (Opcional) Sube docs.db por separado si la tienes
gh release upload v0.1.0 ~/.config/geliShell/docs/docs.db
```

> **Sobre `gh` CLI:** Viene preinstalada en Windows con Git for Windows y en GitHub Codespaces. Instálala con `winget install GitHub.cli` si no la tienes.

---

## 7. Convención de tags y ramas

### Ramas

| Rama | Propósito |
|---|---|
| `main` | Código estable. Solo acepta PRs desde `dev`. Los tags de release apuntan aquí. |
| `dev` | Desarrollo activo. PRs de features/fixes llegan aquí. |

### Tags

Usa **SemVer estricto** con prefijo `v`:

| Tag | Significado |
|---|---|
| `v0.1.0` | Primera release pública. Pre-1.0 = API puede cambiar. |
| `v0.1.1` | Patch: fix de bugs sin nuevas features. |
| `v0.2.0` | Minor: nuevas features retrocompatibles (ej: TriggerEngine). |
| `v1.0.0` | API estable. Publicación en crates.io oficial. |
| `v0.2.0-beta.1` | Pre-release. `bootstrap.rs` acepta pre-releases (busca `latest`). |

### Relación con crates.io (futuro)

Cuando llegue el momento de publicar en crates.io:
```bash
cargo publish --dry-run   # verifica que exclude, autobins y features están bien
cargo publish
```
El `Cargo.toml` ya está preparado (`exclude = ["geli-docs/"]`, `autobins = false`, `description` presente, `license` presente, `repository` presente). Solo falta subir y tener cuenta verificada en crates.io.

---

## 8. Checklist previa a cada release

```
[ ] Todos los tests pasan: cargo test --locked
[ ] Clippy limpio: cargo clippy --all-targets --all-features -- -D warnings  
[ ] Versión actualizada en Cargo.toml
[ ] CHANGELOG / release notes redactadas (o activate generate_release_notes)
[ ] Merge de dev → main aprobado por PR
[ ] docs.db generada localmente si geli-docs/ cambió
[ ] Tag creado y pusheado sobre main
[ ] Workflow de release ejecutado sin errores
[ ] Assets verificados: tamaños razonables, checksums.txt presente
[ ] docs.db subida manualmente si se generó
[ ] bootstrap.rs apunta al repo correcto (GerarddeTena/GeliShell)
[ ] README actualizado si cambió la instalación
```

---

## Resumen ejecutivo

| Qué | Cómo |
|---|---|
| **Dónde viven las releases** | GitHub Releases en `GerarddeTena/GeliShell` (mismo repo) |
| **Cuándo se crean** | Al pushear un tag `v*.*.*` sobre `main` |
| **Quién compila** | GitHub Actions — matriz de 5 plataformas |
| **docs.db** | Build local con Ollama + upload manual en v0.1.0; self-hosted runner en el futuro |
| **Verificación de integridad** | `checksums.txt` (SHA-256, formato GNU) — bootstrap.rs lo verifica automáticamente |
| **Instalación del usuario final** | `cargo install geli_shell` (crates.io, futuro) o descargar binario + ejecutar |
| **Bootstrap automático** | Al primer `geli`, se descargan sqlite-vec y docs.db desde la latest release |
