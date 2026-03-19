# Procesamiento de Texto Complejo

## Intención: Extraer una columna por delimitador
- Bash/Zsh: `awk -F'<separador>' '{print $<indice_columna>}' <ruta_archivo>`
- Fish: `awk -F'<separador>' '{print $<indice_columna>}' <ruta_archivo>`
- PowerShell: `Get-Content <ruta_archivo> | ForEach-Object { ($_ -split '<separador>')[<indice_columna_cero_based>] }`
- CMD: `for /f "tokens=<indice_columna> delims=<separador>" %a in (<ruta_archivo>) do @echo %a`

## Intención: Filtrar filas por regex en una columna
- Bash/Zsh: `awk -F'<separador>' '$<indice_columna> ~ /<regex>/ {print}' <ruta_archivo>`
- Fish: `awk -F'<separador>' '$<indice_columna> ~ /<regex>/ {print}' <ruta_archivo>`
- PowerShell: `Get-Content <ruta_archivo> | Where-Object { ($_ -split '<separador>')[<indice_columna_cero_based>] -match '<regex>' }`
- CMD: `findstr /r "<regex>" <ruta_archivo>`

## Intención: Reemplazar texto usando regex en flujo de salida
- Bash/Zsh: `sed -E 's/<regex>/<reemplazo>/g' <ruta_archivo>`
- Fish: `sed -E 's/<regex>/<reemplazo>/g' <ruta_archivo>`
- PowerShell: `(Get-Content <ruta_archivo>) -replace '<regex>','<reemplazo>'`
- CMD: `powershell -Command "(Get-Content '<ruta_archivo>') -replace '<regex>','<reemplazo>'"`

## Intención: Reemplazar regex y guardar resultado en otro archivo
- Bash/Zsh: `sed -E 's/<regex>/<reemplazo>/g' <ruta_entrada> > <ruta_salida>`
- Fish: `sed -E 's/<regex>/<reemplazo>/g' <ruta_entrada> > <ruta_salida>`
- PowerShell: `(Get-Content <ruta_entrada>) -replace '<regex>','<reemplazo>' | Set-Content <ruta_salida>`
- CMD: `powershell -Command "(Get-Content '<ruta_entrada>') -replace '<regex>','<reemplazo>' | Set-Content '<ruta_salida>'"`

## Intención: Extraer todas las coincidencias regex de un archivo
- Bash/Zsh: `grep -Eo "<regex>" <ruta_archivo>`
- Fish: `grep -Eo "<regex>" <ruta_archivo>`
- PowerShell: `Select-String -Path <ruta_archivo> -Pattern "<regex>" -AllMatches | ForEach-Object { $_.Matches.Value }`
- CMD: `findstr /r "<regex>" <ruta_archivo>`

## Intención: Parsear un campo de JSON con jq
- Bash/Zsh: `jq -r '<filtro_jq>' <ruta_json>`
- Fish: `jq -r '<filtro_jq>' <ruta_json>`
- PowerShell: `Get-Content <ruta_json> | ConvertFrom-Json | Select-Object -ExpandProperty <propiedad>`
- CMD: `jq -r "<filtro_jq>" <ruta_json>`

## Intención: Filtrar objetos de un array JSON por valor
- Bash/Zsh: `jq '.<array>[] | select(.<campo>=="<valor>")' <ruta_json>`
- Fish: `jq '.<array>[] | select(.<campo>=="<valor>")' <ruta_json>`
- PowerShell: `(Get-Content <ruta_json> | ConvertFrom-Json).<array> | Where-Object { $_.<campo> -eq "<valor>" }`
- CMD: `jq ".<array>[] | select(.<campo>==\"<valor>\")" <ruta_json>`

## Intención: Extraer una columna de CSV
- Bash/Zsh: `awk -F',' '{print $<indice_columna>}' <ruta_csv>`
- Fish: `awk -F',' '{print $<indice_columna>}' <ruta_csv>`
- PowerShell: `Import-Csv <ruta_csv> | Select-Object -ExpandProperty <nombre_columna>`
- CMD: `powershell -Command "Import-Csv '<ruta_csv>' | Select-Object -ExpandProperty '<nombre_columna>'"`

## Intención: Filtrar CSV por condición en una columna
- Bash/Zsh: `awk -F',' '$<indice_columna>=="<valor>" {print}' <ruta_csv>`
- Fish: `awk -F',' '$<indice_columna>=="<valor>" {print}' <ruta_csv>`
- PowerShell: `Import-Csv <ruta_csv> | Where-Object { $_.<nombre_columna> -eq "<valor>" }`
- CMD: `powershell -Command "Import-Csv '<ruta_csv>' | Where-Object { $_.<nombre_columna> -eq '<valor>' }"`

# Redirección y Pipes

## Intención: Redirigir stdout a un archivo (sobrescribir)
- Bash/Zsh: `<comando> > <ruta_salida>`
- Fish: `<comando> > <ruta_salida>`
- PowerShell: `<comando> > <ruta_salida>`
- CMD: `<comando> > <ruta_salida>`

## Intención: Redirigir stdout a un archivo (append)
- Bash/Zsh: `<comando> >> <ruta_salida>`
- Fish: `<comando> >> <ruta_salida>`
- PowerShell: `<comando> >> <ruta_salida>`
- CMD: `<comando> >> <ruta_salida>`

## Intención: Redirigir stderr a un archivo
- Bash/Zsh: `<comando> 2> <ruta_error>`
- Fish: `<comando> 2> <ruta_error>`
- PowerShell: `<comando> 2> <ruta_error>`
- CMD: `<comando> 2> <ruta_error>`

## Intención: Unificar stdout y stderr en un único archivo
- Bash/Zsh: `<comando> > <ruta_log> 2>&1`
- Fish: `<comando> > <ruta_log> 2>&1`
- PowerShell: `<comando> > <ruta_log> 2>&1`
- CMD: `<comando> > <ruta_log> 2>&1`

## Intención: Duplicar salida en pantalla y archivo con tee
- Bash/Zsh: `<comando> 2>&1 | tee <ruta_log>`
- Fish: `<comando> 2>&1 | tee <ruta_log>`
- PowerShell: `<comando> 2>&1 | Tee-Object -FilePath <ruta_log>`
- CMD: `powershell -Command "<comando> 2>&1 | Tee-Object -FilePath '<ruta_log>'"`

## Intención: Descartar toda la salida del comando
- Bash/Zsh: `<comando> > /dev/null 2>&1`
- Fish: `<comando> > /dev/null 2>&1`
- PowerShell: `<comando> > $null 2>&1`
- CMD: `<comando> >NUL 2>&1`

## Intención: Volcar logs en tiempo real y persistir copia
- Bash/Zsh: `tail -f <ruta_log> | tee <ruta_copia_log>`
- Fish: `tail -f <ruta_log> | tee <ruta_copia_log>`
- PowerShell: `Get-Content <ruta_log> -Tail <numero_lineas> -Wait | Tee-Object -FilePath <ruta_copia_log>`
- CMD: `powershell -Command "Get-Content '<ruta_log>' -Tail <numero_lineas> -Wait | Tee-Object -FilePath '<ruta_copia_log>'"`

## Intención: Encadenar múltiples transformaciones por pipe
- Bash/Zsh: `<comando_1> | <comando_2> | <comando_3>`
- Fish: `<comando_1> | <comando_2> | <comando_3>`
- PowerShell: `<comando_1> | <comando_2> | <comando_3>`
- CMD: `<comando_1> | <comando_2> | <comando_3>`

# Control de Trabajos y Traps

## Intención: Ejecutar un comando en segundo plano
- Bash/Zsh: `<comando> <argumentos> &`
- Fish: `<comando> <argumentos> &`
- PowerShell: `Start-Job -ScriptBlock { <comando> <argumentos> }`
- CMD: `start "" /b <comando> <argumentos>`

## Intención: Listar trabajos o procesos en segundo plano
- Bash/Zsh: `jobs -l`
- Fish: `jobs`
- PowerShell: `Get-Job`
- CMD: `tasklist`

## Intención: Traer un trabajo al foreground
- Bash/Zsh: `fg %<job_id>`
- Fish: `fg %<job_id>`
- PowerShell: `Receive-Job -Id <job_id> -Wait -AutoRemoveJob`
- CMD: `start "" /wait <comando> <argumentos>`

## Intención: Esperar finalización de un trabajo en background
- Bash/Zsh: `wait %<job_id>`
- Fish: `wait`
- PowerShell: `Wait-Job -Id <job_id>`
- CMD: `powershell -Command "Wait-Process -Id <pid>"`

## Intención: Detener un trabajo o proceso con confirmación interactiva
- Bash/Zsh: `read -p "Confirmar detener job <job_id> [y/N]: " resp && [ "$resp" = "y" ] && kill %<job_id>`
- Fish: `read -l -P "Confirmar detener job <job_id> [y/N]: " resp; test "$resp" = "y"; and kill %<job_id>`
- PowerShell: `Stop-Job -Id <job_id> -Confirm`
- CMD: `set /p resp=Confirmar detener PID <pid> [y/N]: & if /I "%resp%"=="y" taskkill /PID <pid>`

## Intención: Capturar señal SIGINT o SIGTERM en shell Unix
- Bash/Zsh: `trap '<comando_limpieza>' INT TERM`
- Fish: `function <nombre_funcion_trap> --on-signal SIGINT; <comando_limpieza>; end`
- PowerShell: `Register-ObjectEvent -InputObject ([Console]) -EventName CancelKeyPress -Action { <comando_limpieza> }`
- CMD: `powershell -Command "Register-ObjectEvent -InputObject ([Console]) -EventName CancelKeyPress -Action { <comando_limpieza> }"`

## Intención: Ejecutar limpieza automática al cerrar sesión o proceso
- Bash/Zsh: `trap '<comando_limpieza>' EXIT`
- Fish: `function <nombre_funcion_exit> --on-event fish_exit; <comando_limpieza>; end`
- PowerShell: `Register-EngineEvent PowerShell.Exiting -Action { <comando_limpieza> }`
- CMD: `powershell -Command "Register-EngineEvent PowerShell.Exiting -Action { <comando_limpieza> }"`

# Monitorización de Rendimiento

## Intención: Ver uso de disco actual
- Bash/Zsh: `df -h`
- Fish: `df -h`
- PowerShell: `Get-PSDrive -PSProvider FileSystem`
- CMD: `wmic logicaldisk get Caption,FreeSpace,Size`

## Intención: Monitorizar uso de disco en tiempo real
- Bash/Zsh: `watch -n <segundos> df -h`
- Fish: `watch -n <segundos> df -h`
- PowerShell: `while ($true) { Get-PSDrive -PSProvider FileSystem; Start-Sleep -Seconds <segundos> }`
- CMD: `powershell -Command "while ($true) { Get-PSDrive -PSProvider FileSystem; Start-Sleep -Seconds <segundos> }"`

## Intención: Ver procesos con mayor consumo de RAM
- Bash/Zsh: `ps -eo pid,comm,%mem,rss --sort=-%mem | head -n <cantidad>`
- Fish: `ps -eo pid,comm,%mem,rss --sort=-%mem | head -n <cantidad>`
- PowerShell: `Get-Process | Sort-Object -Descending WS | Select-Object -First <cantidad> Name,Id,WS`
- CMD: `wmic process get Name,ProcessId,WorkingSetSize`

## Intención: Ver procesos con mayor consumo de CPU
- Bash/Zsh: `ps -eo pid,comm,%cpu --sort=-%cpu | head -n <cantidad>`
- Fish: `ps -eo pid,comm,%cpu --sort=-%cpu | head -n <cantidad>`
- PowerShell: `Get-Process | Sort-Object -Descending CPU | Select-Object -First <cantidad> Name,Id,CPU`
- CMD: `wmic path Win32_PerfFormattedData_PerfProc_Process get Name,IDProcess,PercentProcessorTime`

## Intención: Monitorizar I/O por proceso
- Bash/Zsh: `iotop -o`
- Fish: `iotop -o`
- PowerShell: `Get-Process | Sort-Object -Descending IOReadBytes | Select-Object -First <cantidad> Name,Id,IOReadBytes,IOWriteBytes`
- CMD: `typeperf "\Process(*)\IO Read Bytes/sec" "\Process(*)\IO Write Bytes/sec" -sc <muestras>`

## Intención: Capturar trazas del sistema o llamadas de proceso
- Bash/Zsh: `strace -p <pid> -f`
- Fish: `strace -p <pid> -f`
- PowerShell: `Get-WinEvent -LogName System -MaxEvents <cantidad_eventos>`
- CMD: `wevtutil qe System /c:<cantidad_eventos> /f:text`

## Intención: Monitorizar conexiones de red en tiempo real
- Bash/Zsh: `watch -n <segundos> ss -s`
- Fish: `watch -n <segundos> ss -s`
- PowerShell: `Get-Counter '\TCPv4\Connections Established' -Continuous`
- CMD: `netstat -ano <intervalo_segundos>`

# Gestión de Servicios y Daemons

## Intención: Listar todos los servicios del sistema
- Bash/Zsh: `systemctl list-units --type=service --all`
- Fish: `systemctl list-units --type=service --all`
- PowerShell: `Get-Service`
- CMD: `sc query type= service state= all`

## Intención: Consultar estado de un servicio específico
- Bash/Zsh: `systemctl status <servicio>`
- Fish: `systemctl status <servicio>`
- PowerShell: `Get-Service -Name <servicio>`
- CMD: `sc query <servicio>`

## Intención: Iniciar un servicio
- Bash/Zsh: `sudo systemctl start <servicio>`
- Fish: `sudo systemctl start <servicio>`
- PowerShell: `Start-Service -Name <servicio>`
- CMD: `sc start <servicio>`

## Intención: Detener un servicio con confirmación interactiva
- Bash/Zsh: `read -p "Confirmar detener servicio <servicio> [y/N]: " resp && [ "$resp" = "y" ] && sudo systemctl stop <servicio>`
- Fish: `read -l -P "Confirmar detener servicio <servicio> [y/N]: " resp; test "$resp" = "y"; and sudo systemctl stop <servicio>`
- PowerShell: `Stop-Service -Name <servicio> -Confirm`
- CMD: `set /p resp=Confirmar detener servicio <servicio> [y/N]: & if /I "%resp%"=="y" sc stop <servicio>`

## Intención: Reiniciar un servicio
- Bash/Zsh: `sudo systemctl restart <servicio>`
- Fish: `sudo systemctl restart <servicio>`
- PowerShell: `Restart-Service -Name <servicio>`
- CMD: `sc stop <servicio> & sc start <servicio>`

## Intención: Habilitar servicio al arranque
- Bash/Zsh: `sudo systemctl enable <servicio>`
- Fish: `sudo systemctl enable <servicio>`
- PowerShell: `Set-Service -Name <servicio> -StartupType Automatic`
- CMD: `sc config <servicio> start= auto`

## Intención: Deshabilitar servicio del arranque
- Bash/Zsh: `sudo systemctl disable <servicio>`
- Fish: `sudo systemctl disable <servicio>`
- PowerShell: `Set-Service -Name <servicio> -StartupType Disabled`
- CMD: `sc config <servicio> start= disabled`

## Intención: Ver logs de un servicio en tiempo real
- Bash/Zsh: `journalctl -u <servicio> -f`
- Fish: `journalctl -u <servicio> -f`
- PowerShell: `Get-WinEvent -LogName System | Where-Object { $_.ProviderName -like "*<servicio>*" }`
- CMD: `wevtutil qe System /q:"*[System[Provider[@Name='<servicio>']]]" /f:text`

# Redes Nivel 3

## Intención: Listar reglas actuales del firewall local
- Bash/Zsh: `sudo ufw status numbered`
- Fish: `sudo ufw status numbered`
- PowerShell: `Get-NetFirewallRule`
- CMD: `netsh advfirewall firewall show rule name=all`

## Intención: Abrir un puerto TCP en firewall local
- Bash/Zsh: `sudo ufw allow <puerto>/tcp`
- Fish: `sudo ufw allow <puerto>/tcp`
- PowerShell: `New-NetFirewallRule -DisplayName "<nombre_regla>" -Direction Inbound -Action Allow -Protocol TCP -LocalPort <puerto>`
- CMD: `netsh advfirewall firewall add rule name="<nombre_regla>" dir=in action=allow protocol=TCP localport=<puerto>`

## Intención: Eliminar una regla de firewall con confirmación interactiva
- Bash/Zsh: `read -p "Confirmar eliminación de regla para puerto <puerto> [y/N]: " resp && [ "$resp" = "y" ] && sudo ufw delete allow <puerto>/tcp`
- Fish: `read -l -P "Confirmar eliminación de regla para puerto <puerto> [y/N]: " resp; test "$resp" = "y"; and sudo ufw delete allow <puerto>/tcp`
- PowerShell: `Remove-NetFirewallRule -DisplayName "<nombre_regla>" -Confirm`
- CMD: `set /p resp=Confirmar eliminación de regla <nombre_regla> [y/N]: & if /I "%resp%"=="y" netsh advfirewall firewall delete rule name="<nombre_regla>"`

## Intención: Mostrar tabla de enrutamiento local
- Bash/Zsh: `ip route show`
- Fish: `ip route show`
- PowerShell: `Get-NetRoute`
- CMD: `route print`

## Intención: Añadir una ruta estática
- Bash/Zsh: `sudo ip route add <red_destino>/<prefijo> via <gateway> dev <interfaz>`
- Fish: `sudo ip route add <red_destino>/<prefijo> via <gateway> dev <interfaz>`
- PowerShell: `New-NetRoute -DestinationPrefix "<red_destino>/<prefijo>" -InterfaceAlias "<interfaz>" -NextHop <gateway>`
- CMD: `route add <red_destino> mask <mascara> <gateway>`

## Intención: Eliminar una ruta estática con confirmación interactiva
- Bash/Zsh: `read -p "Confirmar eliminación de ruta <red_destino>/<prefijo> [y/N]: " resp && [ "$resp" = "y" ] && sudo ip route del <red_destino>/<prefijo>`
- Fish: `read -l -P "Confirmar eliminación de ruta <red_destino>/<prefijo> [y/N]: " resp; test "$resp" = "y"; and sudo ip route del <red_destino>/<prefijo>`
- PowerShell: `Remove-NetRoute -DestinationPrefix "<red_destino>/<prefijo>" -Confirm`
- CMD: `set /p resp=Confirmar eliminación de ruta <red_destino> [y/N]: & if /I "%resp%"=="y" route delete <red_destino>`

## Intención: Resolver DNS detallado para registros A, AAAA y MX
- Bash/Zsh: `dig <dominio> A +short && dig <dominio> AAAA +short && dig <dominio> MX +short`
- Fish: `dig <dominio> A +short; and dig <dominio> AAAA +short; and dig <dominio> MX +short`
- PowerShell: `Resolve-DnsName <dominio> -Type A,AAAA,MX`
- CMD: `nslookup -type=A <dominio> & nslookup -type=AAAA <dominio> & nslookup -type=MX <dominio>`

## Intención: Ejecutar resolución DNS con servidor específico
- Bash/Zsh: `dig @<dns_servidor> <dominio>`
- Fish: `dig @<dns_servidor> <dominio>`
- PowerShell: `Resolve-DnsName <dominio> -Server <dns_servidor> -DnsOnly -NoHostsFile`
- CMD: `nslookup <dominio> <dns_servidor>`
