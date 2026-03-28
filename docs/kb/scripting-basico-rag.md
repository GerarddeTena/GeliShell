# Operaciones de Sistema de Archivos (Filesystem)

## Intención: Crear un directorio
- Bash/Zsh: `mkdir -p <ruta_directorio>`
- Fish: `mkdir -p <ruta_directorio>`
- PowerShell: `New-Item -ItemType Directory -Path <ruta_directorio>`
- CMD: `mkdir <ruta_directorio>`

## Intención: Crear un archivo vacío
- Bash/Zsh: `touch <ruta_archivo>`
- Fish: `touch <ruta_archivo>`
- PowerShell: `New-Item -ItemType File -Path <ruta_archivo>`
- CMD: `type nul > <ruta_archivo>`

## Intención: Listar contenido detallado de un directorio
- Bash/Zsh: `ls -la <ruta_directorio>`
- Fish: `ls -la <ruta_directorio>`
- PowerShell: `Get-ChildItem -Force -Path <ruta_directorio>`
- CMD: `dir <ruta_directorio>`

## Intención: Copiar un archivo
- Bash/Zsh: `cp <ruta_origen> <ruta_destino>`
- Fish: `cp <ruta_origen> <ruta_destino>`
- PowerShell: `Copy-Item -Path <ruta_origen> -Destination <ruta_destino>`
- CMD: `copy <ruta_origen> <ruta_destino>`

## Intención: Copiar un directorio recursivamente
- Bash/Zsh: `cp -r <ruta_origen_directorio> <ruta_destino_directorio>`
- Fish: `cp -r <ruta_origen_directorio> <ruta_destino_directorio>`
- PowerShell: `Copy-Item -Path <ruta_origen_directorio> -Destination <ruta_destino_directorio> -Recurse`
- CMD: `xcopy <ruta_origen_directorio> <ruta_destino_directorio> /E /I`

## Intención: Copiar preservando atributos y timestamps
- Bash/Zsh: `cp -a <ruta_origen> <ruta_destino>`
- Fish: `cp -a <ruta_origen> <ruta_destino>`
- PowerShell: `robocopy <ruta_origen_directorio> <ruta_destino_directorio> /E /COPY:DATS`
- CMD: `robocopy <ruta_origen_directorio> <ruta_destino_directorio> /E /COPY:DATS`

## Intención: Mover o renombrar archivo o directorio
- Bash/Zsh: `mv <ruta_origen> <ruta_destino>`
- Fish: `mv <ruta_origen> <ruta_destino>`
- PowerShell: `Move-Item -Path <ruta_origen> -Destination <ruta_destino>`
- CMD: `move <ruta_origen> <ruta_destino>`

## Intención: Crear enlace simbólico a un archivo
- Bash/Zsh: `ln -s <ruta_objetivo_archivo> <ruta_enlace>`
- Fish: `ln -s <ruta_objetivo_archivo> <ruta_enlace>`
- PowerShell: `New-Item -ItemType SymbolicLink -Path <ruta_enlace> -Target <ruta_objetivo_archivo>`
- CMD: `mklink <ruta_enlace> <ruta_objetivo_archivo>`

## Intención: Crear enlace simbólico a un directorio
- Bash/Zsh: `ln -s <ruta_objetivo_directorio> <ruta_enlace>`
- Fish: `ln -s <ruta_objetivo_directorio> <ruta_enlace>`
- PowerShell: `New-Item -ItemType SymbolicLink -Path <ruta_enlace> -Target <ruta_objetivo_directorio>`
- CMD: `mklink /D <ruta_enlace> <ruta_objetivo_directorio>`

## Intención: Crear enlace duro a un archivo
- Bash/Zsh: `ln <ruta_objetivo_archivo> <ruta_enlace_duro>`
- Fish: `ln <ruta_objetivo_archivo> <ruta_enlace_duro>`
- PowerShell: `New-Item -ItemType HardLink -Path <ruta_enlace_duro> -Target <ruta_objetivo_archivo>`
- CMD: `mklink /H <ruta_enlace_duro> <ruta_objetivo_archivo>`

## Intención: Mostrar metadatos de archivo o directorio
- Bash/Zsh: `stat <ruta>`
- Fish: `stat <ruta>`
- PowerShell: `Get-Item <ruta> | Format-List *`
- CMD: `dir /q <ruta>`

## Intención: Resolver destino de un enlace simbólico
- Bash/Zsh: `readlink <ruta_enlace>`
- Fish: `readlink <ruta_enlace>`
- PowerShell: `(Get-Item <ruta_enlace>).Target`
- CMD: `fsutil reparsepoint query <ruta_enlace>`

## Intención: Eliminar un archivo con confirmación interactiva
- Bash/Zsh: `rm -i <ruta_archivo>`
- Fish: `rm -i <ruta_archivo>`
- PowerShell: `Remove-Item -Path <ruta_archivo> -Confirm`
- CMD: `del /p <ruta_archivo>`

## Intención: Eliminar un directorio recursivamente con confirmación interactiva
- Bash/Zsh: `rm -r -i <ruta_directorio>`
- Fish: `rm -r -i <ruta_directorio>`
- PowerShell: `Remove-Item -Path <ruta_directorio> -Recurse -Confirm`
- CMD: `rmdir /s <ruta_directorio>`

## Intención: Ver tamaño total de un directorio
- Bash/Zsh: `du -sh <ruta_directorio>`
- Fish: `du -sh <ruta_directorio>`
- PowerShell: `(Get-ChildItem -Path <ruta_directorio> -Recurse | Measure-Object -Property Length -Sum).Sum`
- CMD: `powershell -Command "(Get-ChildItem -Path '<ruta_directorio>' -Recurse | Measure-Object -Property Length -Sum).Sum"`

# Visualización de Texto

## Intención: Mostrar contenido completo de un archivo
- Bash/Zsh: `cat <ruta_archivo>`
- Fish: `cat <ruta_archivo>`
- PowerShell: `Get-Content <ruta_archivo>`
- CMD: `type <ruta_archivo>`

## Intención: Paginar un archivo de texto
- Bash/Zsh: `less <ruta_archivo>`
- Fish: `less <ruta_archivo>`
- PowerShell: `Get-Content <ruta_archivo> | more`
- CMD: `more < <ruta_archivo>`

## Intención: Ver las primeras N líneas de un archivo
- Bash/Zsh: `head -n <numero_lineas> <ruta_archivo>`
- Fish: `head -n <numero_lineas> <ruta_archivo>`
- PowerShell: `Get-Content <ruta_archivo> -TotalCount <numero_lineas>`
- CMD: `powershell -Command "Get-Content '<ruta_archivo>' -TotalCount <numero_lineas>"`

## Intención: Ver las últimas N líneas de un archivo
- Bash/Zsh: `tail -n <numero_lineas> <ruta_archivo>`
- Fish: `tail -n <numero_lineas> <ruta_archivo>`
- PowerShell: `Get-Content <ruta_archivo> -Tail <numero_lineas>`
- CMD: `powershell -Command "Get-Content '<ruta_archivo>' -Tail <numero_lineas>"`

## Intención: Seguir un log en tiempo real
- Bash/Zsh: `tail -f <ruta_log>`
- Fish: `tail -f <ruta_log>`
- PowerShell: `Get-Content <ruta_log> -Tail <numero_lineas> -Wait`
- CMD: `powershell -Command "Get-Content '<ruta_log>' -Tail <numero_lineas> -Wait"`

## Intención: Concatenar múltiples archivos en salida estándar
- Bash/Zsh: `cat <ruta_archivo_1> <ruta_archivo_2> <ruta_archivo_n>`
- Fish: `cat <ruta_archivo_1> <ruta_archivo_2> <ruta_archivo_n>`
- PowerShell: `Get-Content <ruta_archivo_1>,<ruta_archivo_2>,<ruta_archivo_n>`
- CMD: `type <ruta_archivo_1> <ruta_archivo_2> <ruta_archivo_n>`

## Intención: Mostrar líneas numeradas de un archivo
- Bash/Zsh: `nl -ba <ruta_archivo>`
- Fish: `nl -ba <ruta_archivo>`
- PowerShell: `$i=0; Get-Content <ruta_archivo> | ForEach-Object { $i++; "{0}`t{1}" -f $i, $_ }`
- CMD: `findstr /n "^" <ruta_archivo>`

## Intención: Contar líneas, palabras y bytes de un archivo
- Bash/Zsh: `wc <ruta_archivo>`
- Fish: `wc <ruta_archivo>`
- PowerShell: `Get-Content <ruta_archivo> | Measure-Object -Line -Word -Character`
- CMD: `for /f %c in ('type <ruta_archivo> ^| find /v /c ""') do @echo %c`

# Gestión de Usuarios Básica

## Intención: Ver el usuario actual
- Bash/Zsh: `whoami`
- Fish: `whoami`
- PowerShell: `whoami`
- CMD: `whoami`

## Intención: Ver identificadores y grupos del usuario actual
- Bash/Zsh: `id`
- Fish: `id`
- PowerShell: `whoami /groups`
- CMD: `whoami /groups`

## Intención: Cambiar la contraseña del usuario actual
- Bash/Zsh: `passwd`
- Fish: `passwd`
- PowerShell: `Set-LocalUser -Name <usuario> -Password (Read-Host -AsSecureString)`
- CMD: `net user <usuario> *`

## Intención: Listar sesiones activas en el sistema
- Bash/Zsh: `w`
- Fish: `w`
- PowerShell: `query user`
- CMD: `query user`

## Intención: Ver historial reciente de inicios de sesión
- Bash/Zsh: `last -n <cantidad_registros>`
- Fish: `last -n <cantidad_registros>`
- PowerShell: `Get-WinEvent -FilterHashtable @{LogName='Security';Id=4624} -MaxEvents <cantidad_registros>`
- CMD: `wevtutil qe Security /q:"*[System[(EventID=4624)]]" /c:<cantidad_registros> /f:text`

## Intención: Ejecutar un comando como otro usuario
- Bash/Zsh: `su - <usuario> -c "<comando>"`
- Fish: `su - <usuario> -c "<comando>"`
- PowerShell: `Start-Process powershell -Credential <usuario> -ArgumentList "-Command <comando>"`
- CMD: `runas /user:<usuario> "<comando>"`

# Procesos Básicos

## Intención: Listar todos los procesos activos
- Bash/Zsh: `ps aux`
- Fish: `ps aux`
- PowerShell: `Get-Process`
- CMD: `tasklist`

## Intención: Buscar procesos por nombre
- Bash/Zsh: `ps aux | grep "<nombre_proceso>"`
- Fish: `ps aux | grep "<nombre_proceso>"`
- PowerShell: `Get-Process -Name <nombre_proceso>`
- CMD: `tasklist | findstr /i "<nombre_proceso>"`

## Intención: Ver detalle de un proceso por PID
- Bash/Zsh: `ps -fp <pid>`
- Fish: `ps -fp <pid>`
- PowerShell: `Get-Process -Id <pid> | Format-List *`
- CMD: `tasklist /fi "PID eq <pid>"`

## Intención: Terminar un proceso por PID con confirmación interactiva
- Bash/Zsh: `read -p "Confirmar terminación de PID <pid> [y/N]: " resp && [ "$resp" = "y" ] && kill -TERM <pid>`
- Fish: `read -l -P "Confirmar terminación de PID <pid> [y/N]: " resp; test "$resp" = "y"; and kill -TERM <pid>`
- PowerShell: `Stop-Process -Id <pid> -Confirm`
- CMD: `set /p resp=Confirmar terminación de PID <pid> [y/N]: & if /I "%resp%"=="y" taskkill /PID <pid>`

## Intención: Terminar procesos por nombre con confirmación interactiva
- Bash/Zsh: `read -p "Confirmar terminación de <nombre_proceso> [y/N]: " resp && [ "$resp" = "y" ] && pkill -TERM -x "<nombre_proceso>"`
- Fish: `read -l -P "Confirmar terminación de <nombre_proceso> [y/N]: " resp; test "$resp" = "y"; and pkill -TERM -x "<nombre_proceso>"`
- PowerShell: `Stop-Process -Name <nombre_proceso> -Confirm`
- CMD: `set /p resp=Confirmar terminación de <nombre_proceso> [y/N]: & if /I "%resp%"=="y" taskkill /IM <nombre_proceso>.exe`

## Intención: Ver árbol jerárquico de procesos
- Bash/Zsh: `ps -ef --forest`
- Fish: `ps -ef --forest`
- PowerShell: `Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,Name`
- CMD: `wmic process get ProcessId,ParentProcessId,Name`

## Intención: Esperar la finalización de un proceso
- Bash/Zsh: `wait <pid>`
- Fish: `wait <pid>`
- PowerShell: `Wait-Process -Id <pid>`
- CMD: `powershell -Command "Wait-Process -Id <pid>"`

# Redes Nivel 1

## Intención: Verificar conectividad ICMP con un host
- Bash/Zsh: `ping -c <cantidad_paquetes> <host_o_ip>`
- Fish: `ping -c <cantidad_paquetes> <host_o_ip>`
- PowerShell: `Test-Connection -Count <cantidad_paquetes> -ComputerName <host_o_ip>`
- CMD: `ping -n <cantidad_paquetes> <host_o_ip>`

## Intención: Trazar la ruta de red hacia un host
- Bash/Zsh: `traceroute <host_o_ip>`
- Fish: `traceroute <host_o_ip>`
- PowerShell: `tracert <host_o_ip>`
- CMD: `tracert <host_o_ip>`

## Intención: Descargar un archivo vía HTTP/HTTPS
- Bash/Zsh: `curl -L "<url>" -o <ruta_archivo_salida>`
- Fish: `curl -L "<url>" -o <ruta_archivo_salida>`
- PowerShell: `Invoke-WebRequest -Uri "<url>" -OutFile <ruta_archivo_salida>`
- CMD: `curl -L "<url>" -o <ruta_archivo_salida>`

## Intención: Obtener direcciones IP locales IPv4
- Bash/Zsh: `hostname -I`
- Fish: `hostname -I`
- PowerShell: `Get-NetIPAddress -AddressFamily IPv4 | Select-Object -ExpandProperty IPAddress`
- CMD: `ipconfig`

## Intención: Listar interfaces de red y estado
- Bash/Zsh: `ip addr show`
- Fish: `ip addr show`
- PowerShell: `Get-NetAdapter`
- CMD: `netsh interface show interface`

## Intención: Resolver un nombre DNS a IP
- Bash/Zsh: `nslookup <dominio>`
- Fish: `nslookup <dominio>`
- PowerShell: `Resolve-DnsName <dominio>`
- CMD: `nslookup <dominio>`

## Intención: Ver conexiones de red activas
- Bash/Zsh: `netstat -tulpn`
- Fish: `netstat -tulpn`
- PowerShell: `Get-NetTCPConnection`
- CMD: `netstat -ano`

# Variables de Entorno y PATH

## Intención: Definir una variable de sesión no exportada
- Bash/Zsh: `<nombre_variable>=<valor_variable>`
- Fish: `set <nombre_variable> <valor_variable>`
- PowerShell: `$<nombre_variable> = "<valor_variable>"`
- CMD: `set <nombre_variable>=<valor_variable>`

## Intención: Exportar una variable de entorno para procesos hijo
- Bash/Zsh: `export <nombre_variable>="<valor_variable>"`
- Fish: `set -x <nombre_variable> "<valor_variable>"`
- PowerShell: `$env:<nombre_variable> = "<valor_variable>"`
- CMD: `set <nombre_variable>=<valor_variable>`

## Intención: Leer el valor de una variable de entorno
- Bash/Zsh: `printenv <nombre_variable>`
- Fish: `printenv <nombre_variable>`
- PowerShell: `(Get-Item -Path Env:<nombre_variable>).Value`
- CMD: `echo %<nombre_variable>%`

## Intención: Eliminar una variable de la sesión
- Bash/Zsh: `read -p "Confirmar eliminación de variable <nombre_variable> [y/N]: " resp && [ "$resp" = "y" ] && unset <nombre_variable>`
- Fish: `read -l -P "Confirmar eliminación de variable <nombre_variable> [y/N]: " resp; test "$resp" = "y"; and set -e <nombre_variable>`
- PowerShell: `Remove-Item -Path Env:<nombre_variable> -Confirm`
- CMD: `set /p resp=Confirmar eliminación de variable <nombre_variable> [y/N]: & if /I "%resp%"=="y" set <nombre_variable>=`

## Intención: Listar todas las variables de entorno
- Bash/Zsh: `printenv`
- Fish: `printenv`
- PowerShell: `Get-ChildItem Env:`
- CMD: `set`

## Intención: Añadir un directorio al PATH de la sesión actual
- Bash/Zsh: `export PATH="$PATH:<ruta_directorio>"`
- Fish: `fish_add_path <ruta_directorio>`
- PowerShell: `$env:PATH += ";<ruta_directorio>"`
- CMD: `set PATH=%PATH%;<ruta_directorio>`

## Intención: Persistir un directorio en el PATH del usuario
- Bash/Zsh: `echo 'export PATH="$PATH:<ruta_directorio>"' >> <ruta_archivo_rc>`
- Fish: `set -U fish_user_paths <ruta_directorio> $fish_user_paths`
- PowerShell: `[Environment]::SetEnvironmentVariable('Path', $env:Path + ';<ruta_directorio>', 'User')`
- CMD: `setx PATH "%PATH%;<ruta_directorio>"`

## Intención: Mostrar el PATH actual
- Bash/Zsh: `echo "$PATH"`
- Fish: `echo $PATH`
- PowerShell: `$env:PATH`
- CMD: `echo %PATH%`
