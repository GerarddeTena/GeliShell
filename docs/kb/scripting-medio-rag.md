# Flujo de Control CLI

## Intención: Ejecutar un bucle for sobre una lista explícita
- Bash/Zsh: `for item in <lista_espaciada>; do <comando_accion> "$item"; done`
- Fish: `for item in <lista_espaciada>; <comando_accion> $item; end`
- PowerShell: `foreach ($item in <lista>) { <comando_accion> $item }`
- CMD: `for %i in (<lista>) do <comando_accion> %i`

## Intención: Ejecutar un bucle for sobre un rango numérico
- Bash/Zsh: `for i in $(seq <inicio> <fin>); do <comando_accion> "$i"; done`
- Fish: `for i in (seq <inicio> <fin>); <comando_accion> $i; end`
- PowerShell: `<inicio>..<fin> | ForEach-Object { <comando_accion> $_ }`
- CMD: `for /l %i in (<inicio>,1,<fin>) do <comando_accion> %i`

## Intención: Ejecutar un bucle while periódico
- Bash/Zsh: `while true; do <comando_accion>; sleep <segundos>; done`
- Fish: `while true; <comando_accion>; sleep <segundos>; end`
- PowerShell: `while ($true) { <comando_accion>; Start-Sleep -Seconds <segundos> }`
- CMD: `:loop & <comando_accion> & timeout /t <segundos> >nul & goto loop`

## Intención: Iterar sobre la salida de un comando
- Bash/Zsh: `for item in $(<comando_generador>); do <comando_accion> "$item"; done`
- Fish: `for item in (<comando_generador>); <comando_accion> $item; end`
- PowerShell: `<comando_generador> | ForEach-Object { <comando_accion> $_ }`
- CMD: `for /f %i in ('<comando_generador>') do <comando_accion> %i`

## Intención: Iterar archivos encontrados y ejecutar acción
- Bash/Zsh: `find <ruta_base> -type f -name "<patron>" -print0 | xargs -0 -I {} <comando_accion> "{}"`
- Fish: `find <ruta_base> -type f -name "<patron>" -print0 | xargs -0 -I {} <comando_accion> "{}"`
- PowerShell: `Get-ChildItem -Path <ruta_base> -Recurse -File -Filter "<patron>" | ForEach-Object { <comando_accion> $_.FullName }`
- CMD: `for /r <ruta_base> %f in (<patron>) do <comando_accion> "%f"`

## Intención: Ejecutar una acción N veces
- Bash/Zsh: `for i in $(seq 1 <repeticiones>); do <comando_accion>; done`
- Fish: `for i in (seq 1 <repeticiones>); <comando_accion>; end`
- PowerShell: `1..<repeticiones> | ForEach-Object { <comando_accion> }`
- CMD: `for /l %i in (1,1,<repeticiones>) do <comando_accion>`

# Condicionales CLI

## Intención: Ejecutar un comando solo si el anterior fue exitoso
- Bash/Zsh: `<comando_1> && <comando_2>`
- Fish: `<comando_1>; and <comando_2>`
- PowerShell: `<comando_1>; if ($?) { <comando_2> }`
- CMD: `<comando_1> && <comando_2>`

## Intención: Ejecutar un comando alternativo si el anterior falla
- Bash/Zsh: `<comando_1> || <comando_2>`
- Fish: `<comando_1>; or <comando_2>`
- PowerShell: `<comando_1>; if (-not $?) { <comando_2> }`
- CMD: `<comando_1> || <comando_2>`

## Intención: Comprobar existencia de archivo antes de operar
- Bash/Zsh: `[ -f <ruta_archivo> ] && <comando_si_existe> || <comando_si_no_existe>`
- Fish: `test -f <ruta_archivo>; and <comando_si_existe>; or <comando_si_no_existe>`
- PowerShell: `if (Test-Path -Path <ruta_archivo> -PathType Leaf) { <comando_si_existe> } else { <comando_si_no_existe> }`
- CMD: `if exist <ruta_archivo> (<comando_si_existe>) else (<comando_si_no_existe>)`

## Intención: Comprobar existencia de directorio antes de operar
- Bash/Zsh: `[ -d <ruta_directorio> ] && <comando_si_existe> || <comando_si_no_existe>`
- Fish: `test -d <ruta_directorio>; and <comando_si_existe>; or <comando_si_no_existe>`
- PowerShell: `if (Test-Path -Path <ruta_directorio> -PathType Container) { <comando_si_existe> } else { <comando_si_no_existe> }`
- CMD: `if exist <ruta_directorio>\ (<comando_si_existe>) else (<comando_si_no_existe>)`

## Intención: Comprobar si una variable está definida
- Bash/Zsh: `[ -n "$(printenv <nombre_variable>)" ] && <comando_si_definida> || <comando_si_vacia>`
- Fish: `set -q <nombre_variable>; and <comando_si_definida>; or <comando_si_vacia>`
- PowerShell: `if ($env:<nombre_variable>) { <comando_si_definida> } else { <comando_si_vacia> }`
- CMD: `if defined <nombre_variable> (<comando_si_definida>) else (<comando_si_vacia>)`

## Intención: Comparar dos valores numéricos
- Bash/Zsh: `[ <valor_1> -gt <valor_2> ] && <comando_si_mayor> || <comando_si_no_mayor>`
- Fish: `test <valor_1> -gt <valor_2>; and <comando_si_mayor>; or <comando_si_no_mayor>`
- PowerShell: `if (<valor_1> -gt <valor_2>) { <comando_si_mayor> } else { <comando_si_no_mayor> }`
- CMD: `if <valor_1> GTR <valor_2> (<comando_si_mayor>) else (<comando_si_no_mayor>)`

# Compresión y Empaquetado

## Intención: Crear archivo tar
- Bash/Zsh: `tar -cvf <archivo_tar> <ruta_origen>`
- Fish: `tar -cvf <archivo_tar> <ruta_origen>`
- PowerShell: `tar -cvf <archivo_tar> <ruta_origen>`
- CMD: `tar -cvf <archivo_tar> <ruta_origen>`

## Intención: Extraer archivo tar
- Bash/Zsh: `tar -xvf <archivo_tar> -C <ruta_destino>`
- Fish: `tar -xvf <archivo_tar> -C <ruta_destino>`
- PowerShell: `tar -xvf <archivo_tar> -C <ruta_destino>`
- CMD: `tar -xvf <archivo_tar> -C <ruta_destino>`

## Intención: Crear archivo tar.gz
- Bash/Zsh: `tar -czvf <archivo_targz> <ruta_origen>`
- Fish: `tar -czvf <archivo_targz> <ruta_origen>`
- PowerShell: `tar -czvf <archivo_targz> <ruta_origen>`
- CMD: `tar -czvf <archivo_targz> <ruta_origen>`

## Intención: Extraer archivo tar.gz
- Bash/Zsh: `tar -xzvf <archivo_targz> -C <ruta_destino>`
- Fish: `tar -xzvf <archivo_targz> -C <ruta_destino>`
- PowerShell: `tar -xzvf <archivo_targz> -C <ruta_destino>`
- CMD: `tar -xzvf <archivo_targz> -C <ruta_destino>`

## Intención: Comprimir archivo o directorio en zip
- Bash/Zsh: `zip -r <archivo_zip> <ruta_origen>`
- Fish: `zip -r <archivo_zip> <ruta_origen>`
- PowerShell: `Compress-Archive -Path <ruta_origen> -DestinationPath <archivo_zip>`
- CMD: `powershell -Command "Compress-Archive -Path '<ruta_origen>' -DestinationPath '<archivo_zip>'"`

## Intención: Extraer zip en el directorio actual
- Bash/Zsh: `unzip <archivo_zip>`
- Fish: `unzip <archivo_zip>`
- PowerShell: `Expand-Archive -Path <archivo_zip> -DestinationPath .`
- CMD: `powershell -Command "Expand-Archive -Path '<archivo_zip>' -DestinationPath '.'"`

## Intención: Extraer zip en un directorio específico
- Bash/Zsh: `unzip <archivo_zip> -d <ruta_destino>`
- Fish: `unzip <archivo_zip> -d <ruta_destino>`
- PowerShell: `Expand-Archive -Path <archivo_zip> -DestinationPath <ruta_destino>`
- CMD: `powershell -Command "Expand-Archive -Path '<archivo_zip>' -DestinationPath '<ruta_destino>'"`

## Intención: Listar contenido de un archivo comprimido sin extraer
- Bash/Zsh: `tar -tvf <archivo_tar_o_targz>`
- Fish: `tar -tvf <archivo_tar_o_targz>`
- PowerShell: `tar -tvf <archivo_tar_o_targz>`
- CMD: `tar -tvf <archivo_tar_o_targz>`

# Búsqueda Avanzada

## Intención: Encontrar archivos por extensión
- Bash/Zsh: `find <ruta_base> -type f -name "*.<extension>"`
- Fish: `find <ruta_base> -type f -name "*.<extension>"`
- PowerShell: `Get-ChildItem -Path <ruta_base> -Recurse -File -Filter "*.<extension>"`
- CMD: `for /r <ruta_base> %f in (*.<extension>) do @echo %f`

## Intención: Encontrar archivos mayores a un tamaño
- Bash/Zsh: `find <ruta_base> -type f -size +<tamano>`
- Fish: `find <ruta_base> -type f -size +<tamano>`
- PowerShell: `Get-ChildItem -Path <ruta_base> -Recurse -File | Where-Object { $_.Length -gt <tamano_bytes> }`
- CMD: `forfiles /p <ruta_base> /s /m *.* /c "cmd /c if @fsize GEQ <tamano_bytes> echo @path"`

## Intención: Encontrar archivos modificados en los últimos N días
- Bash/Zsh: `find <ruta_base> -type f -mtime -<dias>`
- Fish: `find <ruta_base> -type f -mtime -<dias>`
- PowerShell: `Get-ChildItem -Path <ruta_base> -Recurse -File | Where-Object { $_.LastWriteTime -gt (Get-Date).AddDays(-<dias>) }`
- CMD: `forfiles /p <ruta_base> /s /d -<dias> /c "cmd /c echo @path"`

## Intención: Encontrar archivos más antiguos que N días
- Bash/Zsh: `find <ruta_base> -type f -mtime +<dias>`
- Fish: `find <ruta_base> -type f -mtime +<dias>`
- PowerShell: `Get-ChildItem -Path <ruta_base> -Recurse -File | Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-<dias>) }`
- CMD: `forfiles /p <ruta_base> /s /d -<dias> /c "cmd /c echo @path"`

## Intención: Ejecutar un comando sobre cada archivo encontrado
- Bash/Zsh: `find <ruta_base> -type f -name "<patron>" -print0 | xargs -0 -I {} <comando_accion> "{}"`
- Fish: `find <ruta_base> -type f -name "<patron>" -print0 | xargs -0 -I {} <comando_accion> "{}"`
- PowerShell: `Get-ChildItem -Path <ruta_base> -Recurse -File -Filter "<patron>" | ForEach-Object { <comando_accion> $_.FullName }`
- CMD: `for /r <ruta_base> %f in (<patron>) do <comando_accion> "%f"`

## Intención: Buscar por nombre y mostrar tamaño y fecha
- Bash/Zsh: `find <ruta_base> -type f -name "<patron>" -exec ls -lh {} \;`
- Fish: `find <ruta_base> -type f -name "<patron>" -exec ls -lh {} \;`
- PowerShell: `Get-ChildItem -Path <ruta_base> -Recurse -File -Filter "<patron>" | Select-Object FullName,Length,LastWriteTime`
- CMD: `for /r <ruta_base> %f in (<patron>) do @for %s in ("%f") do @echo %f %~zs %~ts`

## Intención: Buscar por extensión y ordenar por fecha de modificación
- Bash/Zsh: `find <ruta_base> -type f -name "*.<extension>" -printf "%T@ %p\n" | sort -nr`
- Fish: `find <ruta_base> -type f -name "*.<extension>" -printf "%T@ %p\n" | sort -nr`
- PowerShell: `Get-ChildItem -Path <ruta_base> -Recurse -File -Filter "*.<extension>" | Sort-Object LastWriteTime -Descending`
- CMD: `for /f "delims=" %f in ('dir <ruta_base>\*.<extension> /s /b') do @echo %~tf %f`

# Redes Nivel 2

## Intención: Ejecutar petición HTTP GET con cabeceras
- Bash/Zsh: `curl -X GET "<url>" -H "Authorization: Bearer <token>" -H "Accept: application/json"`
- Fish: `curl -X GET "<url>" -H "Authorization: Bearer <token>" -H "Accept: application/json"`
- PowerShell: `Invoke-RestMethod -Method Get -Uri "<url>" -Headers @{ Authorization = "Bearer <token>"; Accept = "application/json" }`
- CMD: `curl -X GET "<url>" -H "Authorization: Bearer <token>" -H "Accept: application/json"`

## Intención: Ejecutar petición HTTP POST con JSON
- Bash/Zsh: `curl -X POST "<url>" -H "Content-Type: application/json" -H "Authorization: Bearer <token>" -d '<json_body>'`
- Fish: `curl -X POST "<url>" -H "Content-Type: application/json" -H "Authorization: Bearer <token>" -d '<json_body>'`
- PowerShell: `Invoke-RestMethod -Method Post -Uri "<url>" -Headers @{ Authorization = "Bearer <token>" } -ContentType "application/json" -Body '<json_body>'`
- CMD: `curl -X POST "<url>" -H "Content-Type: application/json" -H "Authorization: Bearer <token>" -d "<json_body>"`

## Intención: Ejecutar petición HTTP PUT con JSON
- Bash/Zsh: `curl -X PUT "<url>" -H "Content-Type: application/json" -H "Authorization: Bearer <token>" -d '<json_body>'`
- Fish: `curl -X PUT "<url>" -H "Content-Type: application/json" -H "Authorization: Bearer <token>" -d '<json_body>'`
- PowerShell: `Invoke-RestMethod -Method Put -Uri "<url>" -Headers @{ Authorization = "Bearer <token>" } -ContentType "application/json" -Body '<json_body>'`
- CMD: `curl -X PUT "<url>" -H "Content-Type: application/json" -H "Authorization: Bearer <token>" -d "<json_body>"`

## Intención: Guardar respuesta HTTP en un archivo
- Bash/Zsh: `curl -X GET "<url>" -H "Accept: application/json" -o <ruta_respuesta>`
- Fish: `curl -X GET "<url>" -H "Accept: application/json" -o <ruta_respuesta>`
- PowerShell: `Invoke-WebRequest -Uri "<url>" -Headers @{ Accept = "application/json" } -OutFile <ruta_respuesta>`
- CMD: `curl -X GET "<url>" -H "Accept: application/json" -o <ruta_respuesta>`

## Intención: Listar puertos locales en estado LISTEN
- Bash/Zsh: `ss -lntup`
- Fish: `ss -lntup`
- PowerShell: `Get-NetTCPConnection -State Listen`
- CMD: `netstat -ano | findstr LISTENING`

## Intención: Verificar si un puerto local específico está abierto
- Bash/Zsh: `nc -zv 127.0.0.1 <puerto>`
- Fish: `nc -zv 127.0.0.1 <puerto>`
- PowerShell: `Test-NetConnection -ComputerName 127.0.0.1 -Port <puerto>`
- CMD: `powershell -Command "Test-NetConnection -ComputerName 127.0.0.1 -Port <puerto>"`

## Intención: Escanear un rango local básico de puertos TCP
- Bash/Zsh: `for p in $(seq <puerto_inicio> <puerto_fin>); do (echo >/dev/tcp/127.0.0.1/$p) >/dev/null 2>&1 && echo $p; done`
- Fish: `for p in (seq <puerto_inicio> <puerto_fin>); nc -z 127.0.0.1 $p; and echo $p; end`
- PowerShell: `<puerto_inicio>..<puerto_fin> | ForEach-Object { if ((Test-NetConnection -ComputerName 127.0.0.1 -Port $_ -WarningAction SilentlyContinue).TcpTestSucceeded) { $_ } }`
- CMD: `for /l %p in (<puerto_inicio>,1,<puerto_fin>) do @powershell -Command "if ((Test-NetConnection -ComputerName 127.0.0.1 -Port %p -WarningAction SilentlyContinue).TcpTestSucceeded) { Write-Output %p }"`

## Intención: Relacionar puertos abiertos con procesos
- Bash/Zsh: `lsof -i -P -n | grep LISTEN`
- Fish: `lsof -i -P -n | grep LISTEN`
- PowerShell: `Get-NetTCPConnection -State Listen | Select-Object LocalAddress,LocalPort,OwningProcess`
- CMD: `netstat -ano`

# Permisos y Propiedad

## Intención: Ver permisos de un archivo o directorio
- Bash/Zsh: `ls -l <ruta>`
- Fish: `ls -l <ruta>`
- PowerShell: `Get-Acl <ruta> | Format-List`
- CMD: `icacls <ruta>`

## Intención: Cambiar permisos usando modo numérico
- Bash/Zsh: `chmod <modo_octal> <ruta>`
- Fish: `chmod <modo_octal> <ruta>`
- PowerShell: `icacls <ruta> /grant <usuario>:(RX)`
- CMD: `icacls <ruta> /grant <usuario>:(RX)`

## Intención: Añadir permiso de ejecución al usuario propietario
- Bash/Zsh: `chmod u+x <ruta>`
- Fish: `chmod u+x <ruta>`
- PowerShell: `icacls <ruta> /grant <usuario>:(X)`
- CMD: `icacls <ruta> /grant <usuario>:(X)`

## Intención: Cambiar propietario de archivo o directorio
- Bash/Zsh: `chown <usuario>:<grupo> <ruta>`
- Fish: `chown <usuario>:<grupo> <ruta>`
- PowerShell: `icacls <ruta> /setowner <usuario>`
- CMD: `icacls <ruta> /setowner <usuario>`

## Intención: Cambiar grupo propietario de archivo o directorio
- Bash/Zsh: `chgrp <grupo> <ruta>`
- Fish: `chgrp <grupo> <ruta>`
- PowerShell: `icacls <ruta> /grant <grupo>:(M)`
- CMD: `icacls <ruta> /grant <grupo>:(M)`

## Intención: Otorgar permiso Modify mediante ACL
- Bash/Zsh: `setfacl -m u:<usuario>:rwX <ruta>`
- Fish: `setfacl -m u:<usuario>:rwX <ruta>`
- PowerShell: `$acl=Get-Acl <ruta>; $rule=New-Object System.Security.AccessControl.FileSystemAccessRule('<usuario>','Modify','Allow'); $acl.SetAccessRule($rule); Set-Acl <ruta> $acl`
- CMD: `icacls <ruta> /grant <usuario>:(M)`

## Intención: Revocar permisos ACL con confirmación interactiva
- Bash/Zsh: `read -p "Confirmar revocar ACL de <usuario> en <ruta> [y/N]: " resp && [ "$resp" = "y" ] && setfacl -x u:<usuario> <ruta>`
- Fish: `read -l -P "Confirmar revocar ACL de <usuario> en <ruta> [y/N]: " resp; test "$resp" = "y"; and setfacl -x u:<usuario> <ruta>`
- PowerShell: `if ((Read-Host "Confirmar revocar ACL de <usuario> en <ruta> [y/N]") -eq "y") { icacls <ruta> /remove <usuario> }`
- CMD: `set /p resp=Confirmar revocar ACL de <usuario> en <ruta> [y/N]: & if /I "%resp%"=="y" icacls <ruta> /remove <usuario>`
