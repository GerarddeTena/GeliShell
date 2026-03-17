> **Propósito:**
> Definir el vocabulario operativo canónico de GeliShell derivado exclusivamente de `commands.toml`.
> Restringir las sugerencias del LLM a comandos y flags explícitamente mapeados en el diccionario canónico.

# Diccionario de Comandos Canónicos de GeliShell (Derivado de commands.toml)

## Regla operativa del vocabulario canónico para IA

La IA debe construir sugerencias únicamente con comandos y flags presentes en este diccionario. Cualquier instrucción fuera de este vocabulario se considera no autorizada para autocompletado y recomendación automática.

```toml
[[commands]]
name = "list"
translate = { bash = { exact = "ls" }, powershell = { exact = "Get-ChildItem" } }
[[commands.flags]]
canonical = "--all"
bash = "-a"
powershell = "-Force"
```

## Categoría: filesystem

- **Comando Canónico:** `list`
  - **Descripción:** List directory contents
  - **Traducción por defecto:** bash: `ls`, zsh: `ls`, fish: `ls`, powershell: `Get-ChildItem`, cmd: `dir`
  - **Flags soportados:**
    - `--all` -> bash: `-a`, zsh: `-a`, fish: `-a`, powershell: `-Force`, cmd: `/a`
    - `--long` -> bash: `-l`, zsh: `-l`, fish: `-l`, powershell: `| Format-List`
    - `--human-readable` -> bash: `-lh`, zsh: `-lh`, fish: `-lh`
    - `--recursive` -> bash: `-R`, zsh: `-R`, fish: `-R`, powershell: `-Recurse`, cmd: `/s`

- **Comando Canónico:** `change-dir`
  - **Descripción:** Change the current working directory
  - **Traducción por defecto:** bash: `cd`, zsh: `cd`, fish: `cd`, powershell: `Set-Location`, cmd: `cd`
  - **Flags soportados:**
    - `--home` -> bash: `~`, zsh: `~`, fish: `~`, powershell: `$HOME`, cmd: `%USERPROFILE%`
    - `--back` -> bash: `-`, zsh: `-`, fish: `-`, powershell: `-`

- **Comando Canónico:** `print-dir`
  - **Descripción:** Print the current working directory path
  - **Traducción por defecto:** bash: `pwd`, zsh: `pwd`, fish: `pwd`, powershell: `Get-Location`, cmd: `cd`
  - **Flags soportados:** Ninguno

- **Comando Canónico:** `make-dir`
  - **Descripción:** Create a new directory
  - **Traducción por defecto:** bash: `mkdir`, zsh: `mkdir`, fish: `mkdir`, powershell: `New-Item -ItemType Directory`, cmd: `mkdir`
  - **Flags soportados:**
    - `--parents` -> bash: `-p`, zsh: `-p`, fish: `-p`, powershell: `-Force`

- **Comando Canónico:** `remove`
  - **Descripción:** Remove files or directories
  - **Traducción por defecto:** bash: `rm`, zsh: `rm`, fish: `rm`, powershell: `Remove-Item`, cmd: `del`
  - **Flags soportados:**
    - `--recursive` -> bash: `-r`, zsh: `-r`, fish: `-r`, powershell: `-Recurse`, cmd: `/s`
    - `--force` -> bash: `-f`, zsh: `-f`, fish: `-f`, powershell: `-Force`, cmd: `/f`
    - `--interactive` -> bash: `-i`, zsh: `-i`, fish: `-i`, powershell: `-Confirm`

- **Comando Canónico:** `copy`
  - **Descripción:** Copy files or directories
  - **Traducción por defecto:** bash: `cp`, zsh: `cp`, fish: `cp`, powershell: `Copy-Item`, cmd: `copy`
  - **Flags soportados:**
    - `--recursive` -> bash: `-r`, zsh: `-r`, fish: `-r`, powershell: `-Recurse`, cmd: `/s`
    - `--verbose` -> bash: `-v`, zsh: `-v`, fish: `-v`, powershell: `-Verbose`
    - `--preserve` -> bash: `-p`, zsh: `-p`, fish: `-p`

- **Comando Canónico:** `move`
  - **Descripción:** Move or rename files and directories
  - **Traducción por defecto:** bash: `mv`, zsh: `mv`, fish: `mv`, powershell: `Move-Item`, cmd: `move`
  - **Flags soportados:**
    - `--force` -> bash: `-f`, zsh: `-f`, fish: `-f`, powershell: `-Force`, cmd: `/y`
    - `--verbose` -> bash: `-v`, zsh: `-v`, fish: `-v`, powershell: `-Verbose`

- **Comando Canónico:** `find`
  - **Descripción:** Search for files in a directory hierarchy
  - **Traducción por defecto:** bash: `find`, zsh: `find`, fish: `find`, powershell: `Get-ChildItem -Recurse -Filter`, cmd: `dir /s /b`
  - **Flags soportados:**
    - `--name` -> bash: `-name`, zsh: `-name`, fish: `-name`, powershell: `-Filter`
    - `--type-file` -> bash: `-type f`, zsh: `-type f`, fish: `-type f`, powershell: `-File`
    - `--type-dir` -> bash: `-type d`, zsh: `-type d`, fish: `-type d`, powershell: `-Directory`

## Categoría: file-ops

- **Comando Canónico:** `read`
  - **Descripción:** Display the contents of a file
  - **Traducción por defecto:** bash: `cat`, zsh: `cat`, fish: `cat`, powershell: `Get-Content`, cmd: `type`
  - **Flags soportados:**
    - `--number-lines` -> bash: `-n`, zsh: `-n`, fish: `-n`

- **Comando Canónico:** `read-paged`
  - **Descripción:** View file contents page by page
  - **Traducción por defecto:** bash: `less`, zsh: `less`, fish: `less`, powershell: `more`, cmd: `more`
  - **Flags soportados:** Ninguno

- **Comando Canónico:** `create-file`
  - **Descripción:** Create an empty file or update its timestamp
  - **Traducción por defecto:** bash: `touch`, zsh: `touch`, fish: `touch`, powershell: `New-Item -ItemType File`, cmd: `type nul >`
  - **Flags soportados:** Ninguno

- **Comando Canónico:** `tail`
  - **Descripción:** Display the last lines of a file
  - **Traducción por defecto:** bash: `tail`, zsh: `tail`, fish: `tail`, powershell: `Get-Content -Tail`, cmd: `more`
  - **Flags soportados:**
    - `--lines` -> bash: `-n`, zsh: `-n`, fish: `-n`, powershell: `-Tail`
    - `--follow` -> bash: `-f`, zsh: `-f`, fish: `-f`, powershell: `-Wait`

- **Comando Canónico:** `head`
  - **Descripción:** Display the first lines of a file
  - **Traducción por defecto:** bash: `head`, zsh: `head`, fish: `head`, powershell: `Get-Content -TotalCount`, cmd: `more +1`
  - **Flags soportados:**
    - `--lines` -> bash: `-n`, zsh: `-n`, fish: `-n`, powershell: `-TotalCount`

## Categoría: process

- **Comando Canónico:** `list-processes`
  - **Descripción:** List running processes
  - **Traducción por defecto:** bash: `ps`, zsh: `ps`, fish: `ps`, powershell: `Get-Process`, cmd: `tasklist`
  - **Flags soportados:**
    - `--all` -> bash: `aux`, zsh: `aux`, fish: `aux`

- **Comando Canónico:** `kill-process`
  - **Descripción:** Terminate a process by PID
  - **Traducción por defecto:** bash: `kill`, zsh: `kill`, fish: `kill`, powershell: `Stop-Process -Id`, cmd: `taskkill /PID`
  - **Flags soportados:**
    - `--force` -> bash: `-9`, zsh: `-9`, fish: `-9`, powershell: `-Force`, cmd: `/F`
    - `--by-name` -> bash: `killall`, zsh: `killall`, fish: `killall`, powershell: `Stop-Process -Name`, cmd: `taskkill /IM`

- **Comando Canónico:** `which`
  - **Descripción:** Locate a command executable in PATH
  - **Traducción por defecto:** bash: `which`, zsh: `which`, fish: `which`, powershell: `Get-Command`, cmd: `where`
  - **Flags soportados:** Ninguno

- **Comando Canónico:** `run-background`
  - **Descripción:** Run a command in the background
  - **Traducción por defecto:** bash: `nohup {cmd} &`, zsh: `nohup {cmd} &`, fish: `{cmd} &`, powershell: `Start-Process {cmd}`, cmd: `start {cmd}`
  - **Flags soportados:** Ninguno

## Categoría: network

- **Comando Canónico:** `ping`
  - **Descripción:** Send ICMP echo requests to a host
  - **Traducción por defecto:** bash: `ping`, zsh: `ping`, fish: `ping`, powershell: `Test-Connection`, cmd: `ping`
  - **Flags soportados:**
    - `--count` -> bash: `-c`, zsh: `-c`, fish: `-c`, powershell: `-Count`, cmd: `-n`

- **Comando Canónico:** `http-get`
  - **Descripción:** Make an HTTP GET request
  - **Traducción por defecto:** bash: `curl -X GET`, zsh: `curl -X GET`, fish: `curl -X GET`, powershell: `Invoke-WebRequest -Method GET`, cmd: `curl -X GET`
  - **Flags soportados:**
    - `--header` -> bash: `-H`, zsh: `-H`, fish: `-H`, powershell: `-Headers`, cmd: `-H`
    - `--output` -> bash: `-o`, zsh: `-o`, fish: `-o`, powershell: `-OutFile`, cmd: `-o`

- **Comando Canónico:** `download`
  - **Descripción:** Download a file from a URL
  - **Traducción por defecto:** bash: `wget`, zsh: `wget`, fish: `wget`, powershell: `Invoke-WebRequest -OutFile`, cmd: `curl -O`
  - **Flags soportados:**
    - `--output` -> bash: `-O`, zsh: `-O`, fish: `-O`, powershell: `-OutFile`, cmd: `-o`

## Categoría: text

- **Comando Canónico:** `search`
  - **Descripción:** Search text patterns in files
  - **Traducción por defecto:** bash: `grep`, zsh: `grep`, fish: `grep`, powershell: `Select-String`, cmd: `findstr`
  - **Flags soportados:**
    - `--recursive` -> bash: `-r`, zsh: `-r`, fish: `-r`, powershell: `-Path ** -Recurse`, cmd: `/s`
    - `--ignore-case` -> bash: `-i`, zsh: `-i`, fish: `-i`, cmd: `/i`
    - `--line-number` -> bash: `-n`, zsh: `-n`, fish: `-n`, cmd: `/n`
    - `--invert` -> bash: `-v`, zsh: `-v`, fish: `-v`, powershell: `-NotMatch`

- **Comando Canónico:** `count`
  - **Descripción:** Count lines, words, or characters in input
  - **Traducción por defecto:** bash: `wc`, zsh: `wc`, fish: `wc`, powershell: `Measure-Object`, cmd: `find /c /v \`
  - **Flags soportados:**
    - `--lines` -> bash: `-l`, zsh: `-l`, fish: `-l`, powershell: `-Line`
    - `--words` -> bash: `-w`, zsh: `-w`, fish: `-w`, powershell: `-Word`

- **Comando Canónico:** `sort`
  - **Descripción:** Sort lines of text
  - **Traducción por defecto:** bash: `sort`, zsh: `sort`, fish: `sort`, powershell: `Sort-Object`, cmd: `sort`
  - **Flags soportados:**
    - `--reverse` -> bash: `-r`, zsh: `-r`, fish: `-r`, powershell: `-Descending`, cmd: `/r`
    - `--unique` -> bash: `-u`, zsh: `-u`, fish: `-u`, powershell: `-Unique`

- **Comando Canónico:** `unique`
  - **Descripción:** Remove or report duplicate lines
  - **Traducción por defecto:** bash: `uniq`, zsh: `uniq`, fish: `uniq`, powershell: `Get-Unique`, cmd: `sort /unique`
  - **Flags soportados:**
    - `--count` -> bash: `-c`, zsh: `-c`, fish: `-c`

- **Comando Canónico:** `replace`
  - **Descripción:** Replace text in a stream
  - **Traducción por defecto:** bash: `sed`, zsh: `sed`, fish: `sed`, powershell: `ForEach-Object { $_ -replace`, cmd: `powershell -c \`
  - **Flags soportados:** Ninguno

## Categoría: system

- **Comando Canónico:** `clear`
  - **Descripción:** Clear the terminal screen
  - **Traducción por defecto:** bash: `clear`, zsh: `clear`, fish: `clear`, powershell: `Clear-Host`, cmd: `cls`
  - **Flags soportados:** Ninguno

- **Comando Canónico:** `whoami`
  - **Descripción:** Print the current logged-in user
  - **Traducción por defecto:** bash: `whoami`, zsh: `whoami`, fish: `whoami`, powershell: `$env:USERNAME`, cmd: `whoami`
  - **Flags soportados:** Ninguno

- **Comando Canónico:** `env-get`
  - **Descripción:** Print all or a specific environment variable
  - **Traducción por defecto:** bash: `printenv`, zsh: `printenv`, fish: `set`, powershell: `Get-ChildItem Env:`, cmd: `set`
  - **Flags soportados:** Ninguno

- **Comando Canónico:** `env-set`
  - **Descripción:** Set an environment variable for the current session
  - **Traducción por defecto:** bash: `export`, zsh: `export`, fish: `set -x`, powershell: `$env:`, cmd: `set`
  - **Flags soportados:** Ninguno

- **Comando Canónico:** `add-to-path`
  - **Descripción:** Add a directory to the PATH environment variable
  - **Traducción por defecto:** bash: `export PATH=\`, zsh: `export PATH=\`, fish: `fish_add_path {dir}`, powershell: `$env:PATH += ';{dir}'`, cmd: `setx PATH \`
  - **Flags soportados:** Ninguno

- **Comando Canónico:** `disk-usage`
  - **Descripción:** Show disk space usage
  - **Traducción por defecto:** bash: `df`, zsh: `df`, fish: `df`, powershell: `Get-PSDrive`, cmd: `wmic logicaldisk get size,freespace,caption`
  - **Flags soportados:**
    - `--human-readable` -> bash: `-h`, zsh: `-h`, fish: `-h`

- **Comando Canónico:** `dir-size`
  - **Descripción:** Show the size of a directory
  - **Traducción por defecto:** bash: `du -sh`, zsh: `du -sh`, fish: `du -sh`, powershell: `Get-ChildItem -Recurse | Measure-Object -Property Length -Sum`, cmd: `powershell -c \`
  - **Flags soportados:** Ninguno

- **Comando Canónico:** `history`
  - **Descripción:** Show command history
  - **Traducción por defecto:** bash: `history`, zsh: `history`, fish: `history`, powershell: `Get-History`, cmd: `doskey /history`
  - **Flags soportados:**
    - `--clear` -> bash: `-c`, zsh: `-c`, fish: `--delete --prefix ''`, powershell: `Clear-History`

- **Comando Canónico:** `alias`
  - **Descripción:** Create a command alias
  - **Traducción por defecto:** bash: `alias`, zsh: `alias`, fish: `alias`, powershell: `Set-Alias`, cmd: `doskey`
  - **Flags soportados:** Ninguno

- **Comando Canónico:** `permissions`
  - **Descripción:** Change file permissions
  - **Traducción por defecto:** bash: `chmod`, zsh: `chmod`, fish: `chmod`, powershell: `icacls`, cmd: `icacls`
  - **Flags soportados:**
    - `--recursive` -> bash: `-R`, zsh: `-R`, fish: `-R`, powershell: `/T`, cmd: `/T`

- **Comando Canónico:** `link`
  - **Descripción:** Create a symbolic or hard link
  - **Traducción por defecto:** bash: `ln`, zsh: `ln`, fish: `ln`, powershell: `New-Item -ItemType SymbolicLink`, cmd: `mklink`
  - **Flags soportados:**
    - `--symbolic` -> bash: `-s`, zsh: `-s`, fish: `-s`, powershell: `-ItemType SymbolicLink`

## Categoría: dev

- **Comando Canónico:** `git-status`
  - **Descripción:** Show git working tree status
  - **Traducción por defecto:** bash: `git status`, zsh: `git status`, fish: `git status`, powershell: `git status`, cmd: `git status`
  - **Flags soportados:** Ninguno

- **Comando Canónico:** `git-log`
  - **Descripción:** Show git commit history
  - **Traducción por defecto:** bash: `git log`, zsh: `git log`, fish: `git log`, powershell: `git log`, cmd: `git log`
  - **Flags soportados:**
    - `--oneline` -> bash: `--oneline`, zsh: `--oneline`, fish: `--oneline`, powershell: `--oneline`, cmd: `--oneline`

- **Comando Canónico:** `archive`
  - **Descripción:** Create or extract a compressed archive
  - **Traducción por defecto:** bash: `tar`, zsh: `tar`, fish: `tar`, powershell: `Compress-Archive`, cmd: `tar`
  - **Flags soportados:**
    - `--extract` -> bash: `-xzf`, zsh: `-xzf`, fish: `-xzf`, powershell: `-DestinationPath`, cmd: `-xzf`
    - `--create` -> bash: `-czf`, zsh: `-czf`, fish: `-czf`, powershell: `-Path`, cmd: `-czf`
