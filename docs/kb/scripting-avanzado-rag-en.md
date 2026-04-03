# Complex Text Processing

## Intención: Extract a column by delimiter
- Bash/Zsh: `awk -F'<delimiter>' '{print $<column_index>}' <file_path>`
- Fish: `awk -F'<delimiter>' '{print $<column_index>}' <file_path>`
- PowerShell: `Get-Content <file_path> | ForEach-Object { ($_ -split '<delimiter>')[<column_index_zero_based>] }`
- CMD: `for /f "tokens=<column_index> delims=<delimiter>" %a in (<file_path>) do @echo %a`

## Intención: Filter rows by regex in a column
- Bash/Zsh: `awk -F'<delimiter>' '$<column_index> ~ /<regex>/ {print}' <file_path>`
- Fish: `awk -F'<delimiter>' '$<column_index> ~ /<regex>/ {print}' <file_path>`
- PowerShell: `Get-Content <file_path> | Where-Object { ($_ -split '<delimiter>')[<column_index_zero_based>] -match '<regex>' }`
- CMD: `findstr /r "<regex>" <file_path>`

## Intención: Replace text using regex in output stream
- Bash/Zsh: `sed -E 's/<regex>/<replacement>/g' <file_path>`
- Fish: `sed -E 's/<regex>/<replacement>/g' <file_path>`
- PowerShell: `(Get-Content <file_path>) -replace '<regex>','<replacement>'`
- CMD: `powershell -Command "(Get-Content '<file_path>') -replace '<regex>','<replacement>'"`

## Intención: Replace regex and save result to another file
- Bash/Zsh: `sed -E 's/<regex>/<replacement>/g' <input_path> > <output_path>`
- Fish: `sed -E 's/<regex>/<replacement>/g' <input_path> > <output_path>`
- PowerShell: `(Get-Content <input_path>) -replace '<regex>','<replacement>' | Set-Content <output_path>`
- CMD: `powershell -Command "(Get-Content '<input_path>') -replace '<regex>','<replacement>' | Set-Content '<output_path>'"`

## Intención: Extract all regex matches from a file
- Bash/Zsh: `grep -Eo "<regex>" <file_path>`
- Fish: `grep -Eo "<regex>" <file_path>`
- PowerShell: `Select-String -Path <file_path> -Pattern "<regex>" -AllMatches | ForEach-Object { $_.Matches.Value }`
- CMD: `findstr /r "<regex>" <file_path>`

## Intención: Parse a JSON field with jq
- Bash/Zsh: `jq -r '<jq_filter>' <json_path>`
- Fish: `jq -r '<jq_filter>' <json_path>`
- PowerShell: `Get-Content <json_path> | ConvertFrom-Json | Select-Object -ExpandProperty <property>`
- CMD: `jq -r "<jq_filter>" <json_path>`

## Intención: Filter JSON array objects by value
- Bash/Zsh: `jq '.<array>[] | select(.<field>=="<value>")' <json_path>`
- Fish: `jq '.<array>[] | select(.<field>=="<value>")' <json_path>`
- PowerShell: `(Get-Content <json_path> | ConvertFrom-Json).<array> | Where-Object { $_.<field> -eq "<value>" }`
- CMD: `jq ".<array>[] | select(.<field>==\"<value>\")" <json_path>`

## Intención: Extract a column from CSV
- Bash/Zsh: `awk -F',' '{print $<column_index>}' <csv_path>`
- Fish: `awk -F',' '{print $<column_index>}' <csv_path>`
- PowerShell: `Import-Csv <csv_path> | Select-Object -ExpandProperty <column_name>`
- CMD: `powershell -Command "Import-Csv '<csv_path>' | Select-Object -ExpandProperty '<column_name>'"`

## Intención: Filter CSV by condition in a column
- Bash/Zsh: `awk -F',' '$<column_index>=="<value>" {print}' <csv_path>`
- Fish: `awk -F',' '$<column_index>=="<value>" {print}' <csv_path>`
- PowerShell: `Import-Csv <csv_path> | Where-Object { $_.<column_name> -eq "<value>" }`
- CMD: `powershell -Command "Import-Csv '<csv_path>' | Where-Object { $_.<column_name> -eq '<value>' }"`

# Redirection and Pipes

## Intención: Redirect stdout to a file (overwrite)
- Bash/Zsh: `<command> > <output_path>`
- Fish: `<command> > <output_path>`
- PowerShell: `<command> > <output_path>`
- CMD: `<command> > <output_path>`

## Intención: Redirect stdout to a file (append)
- Bash/Zsh: `<command> >> <output_path>`
- Fish: `<command> >> <output_path>`
- PowerShell: `<command> >> <output_path>`
- CMD: `<command> >> <output_path>`

## Intención: Redirect stderr to a file
- Bash/Zsh: `<command> 2> <error_path>`
- Fish: `<command> 2> <error_path>`
- PowerShell: `<command> 2> <error_path>`
- CMD: `<command> 2> <error_path>`

## Intención: Merge stdout and stderr into a single file
- Bash/Zsh: `<command> > <log_path> 2>&1`
- Fish: `<command> > <log_path> 2>&1`
- PowerShell: `<command> > <log_path> 2>&1`
- CMD: `<command> > <log_path> 2>&1`

## Intención: Duplicate output to screen and file with tee
- Bash/Zsh: `<command> 2>&1 | tee <log_path>`
- Fish: `<command> 2>&1 | tee <log_path>`
- PowerShell: `<command> 2>&1 | Tee-Object -FilePath <log_path>`
- CMD: `powershell -Command "<command> 2>&1 | Tee-Object -FilePath '<log_path>'"`

## Intención: Discard all command output
- Bash/Zsh: `<command> > /dev/null 2>&1`
- Fish: `<command> > /dev/null 2>&1`
- PowerShell: `<command> > $null 2>&1`
- CMD: `<command> >NUL 2>&1`

## Intención: Dump logs in real time and persist copy
- Bash/Zsh: `tail -f <log_path> | tee <copy_log_path>`
- Fish: `tail -f <log_path> | tee <copy_log_path>`
- PowerShell: `Get-Content <log_path> -Tail <line_count> -Wait | Tee-Object -FilePath <copy_log_path>`
- CMD: `powershell -Command "Get-Content '<log_path>' -Tail <line_count> -Wait | Tee-Object -FilePath '<copy_log_path>'"`

## Intención: Chain multiple transformations via pipe
- Bash/Zsh: `<command_1> | <command_2> | <command_3>`
- Fish: `<command_1> | <command_2> | <command_3>`
- PowerShell: `<command_1> | <command_2> | <command_3>`
- CMD: `<command_1> | <command_2> | <command_3>`

# Job Control and Traps

## Intención: Run a command in the background
- Bash/Zsh: `<command> <arguments> &`
- Fish: `<command> <arguments> &`
- PowerShell: `Start-Job -ScriptBlock { <command> <arguments> }`
- CMD: `start "" /b <command> <arguments>`

## Intención: List background jobs or processes
- Bash/Zsh: `jobs -l`
- Fish: `jobs`
- PowerShell: `Get-Job`
- CMD: `tasklist`

## Intención: Bring a job to the foreground
- Bash/Zsh: `fg %<job_id>`
- Fish: `fg %<job_id>`
- PowerShell: `Receive-Job -Id <job_id> -Wait -AutoRemoveJob`
- CMD: `start "" /wait <command> <arguments>`

## Intención: Wait for background job completion
- Bash/Zsh: `wait %<job_id>`
- Fish: `wait`
- PowerShell: `Wait-Job -Id <job_id>`
- CMD: `powershell -Command "Wait-Process -Id <pid>"`

## Intención: Stop a job or process with interactive confirmation
- Bash/Zsh: `read -p "Confirm stop job <job_id> [y/N]: " resp && [ "$resp" = "y" ] && kill %<job_id>`
- Fish: `read -l -P "Confirm stop job <job_id> [y/N]: " resp; test "$resp" = "y"; and kill %<job_id>`
- PowerShell: `Stop-Job -Id <job_id> -Confirm`
- CMD: `set /p resp=Confirm stop PID <pid> [y/N]: & if /I "%resp%"=="y" taskkill /PID <pid>`

## Intención: Capture SIGINT or SIGTERM signal in Unix shell
- Bash/Zsh: `trap '<cleanup_command>' INT TERM`
- Fish: `function <trap_function_name> --on-signal SIGINT; <cleanup_command>; end`
- PowerShell: `Register-ObjectEvent -InputObject ([Console]) -EventName CancelKeyPress -Action { <cleanup_command> }`
- CMD: `powershell -Command "Register-ObjectEvent -InputObject ([Console]) -EventName CancelKeyPress -Action { <cleanup_command> }"`

## Intención: Execute automatic cleanup on session or process close
- Bash/Zsh: `trap '<cleanup_command>' EXIT`
- Fish: `function <exit_function_name> --on-event fish_exit; <cleanup_command>; end`
- PowerShell: `Register-EngineEvent PowerShell.Exiting -Action { <cleanup_command> }`
- CMD: `powershell -Command "Register-EngineEvent PowerShell.Exiting -Action { <cleanup_command> }"`

# Performance Monitoring

## Intención: View current disk usage
- Bash/Zsh: `df -h`
- Fish: `df -h`
- PowerShell: `Get-PSDrive -PSProvider FileSystem`
- CMD: `wmic logicaldisk get Caption,FreeSpace,Size`

## Intención: Monitor disk usage in real time
- Bash/Zsh: `watch -n <seconds> df -h`
- Fish: `watch -n <seconds> df -h`
- PowerShell: `while ($true) { Get-PSDrive -PSProvider FileSystem; Start-Sleep -Seconds <seconds> }`
- CMD: `powershell -Command "while ($true) { Get-PSDrive -PSProvider FileSystem; Start-Sleep -Seconds <seconds> }"`

## Intención: View processes with highest RAM consumption
- Bash/Zsh: `ps -eo pid,comm,%mem,rss --sort=-%mem | head -n <count>`
- Fish: `ps -eo pid,comm,%mem,rss --sort=-%mem | head -n <count>`
- PowerShell: `Get-Process | Sort-Object -Descending WS | Select-Object -First <count> Name,Id,WS`
- CMD: `wmic process get Name,ProcessId,WorkingSetSize`

## Intención: View processes with highest CPU consumption
- Bash/Zsh: `ps -eo pid,comm,%cpu --sort=-%cpu | head -n <count>`
- Fish: `ps -eo pid,comm,%cpu --sort=-%cpu | head -n <count>`
- PowerShell: `Get-Process | Sort-Object -Descending CPU | Select-Object -First <count> Name,Id,CPU`
- CMD: `wmic path Win32_PerfFormattedData_PerfProc_Process get Name,IDProcess,PercentProcessorTime`

## Intención: Monitor I/O by process
- Bash/Zsh: `iotop -o`
- Fish: `iotop -o`
- PowerShell: `Get-Process | Sort-Object -Descending IOReadBytes | Select-Object -First <count> Name,Id,IOReadBytes,IOWriteBytes`
- CMD: `typeperf "\Process(*)\IO Read Bytes/sec" "\Process(*)\IO Write Bytes/sec" -sc <samples>`

## Intención: Capture system traces or process calls
- Bash/Zsh: `strace -p <pid> -f`
- Fish: `strace -p <pid> -f`
- PowerShell: `Get-WinEvent -LogName System -MaxEvents <event_count>`
- CMD: `wevtutil qe System /c:<event_count> /f:text`

## Intención: Monitor network connections in real time
- Bash/Zsh: `watch -n <seconds> ss -s`
- Fish: `watch -n <seconds> ss -s`
- PowerShell: `Get-Counter '\TCPv4\Connections Established' -Continuous`
- CMD: `netstat -ano <interval_seconds>`

# Service and Daemon Management

## Intención: List all system services
- Bash/Zsh: `systemctl list-units --type=service --all`
- Fish: `systemctl list-units --type=service --all`
- PowerShell: `Get-Service`
- CMD: `sc query type= service state= all`

## Intención: Query specific service status
- Bash/Zsh: `systemctl status <service>`
- Fish: `systemctl status <service>`
- PowerShell: `Get-Service -Name <service>`
- CMD: `sc query <service>`

## Intención: Start a service
- Bash/Zsh: `sudo systemctl start <service>`
- Fish: `sudo systemctl start <service>`
- PowerShell: `Start-Service -Name <service>`
- CMD: `sc start <service>`

## Intención: Stop a service with interactive confirmation
- Bash/Zsh: `read -p "Confirm stop service <service> [y/N]: " resp && [ "$resp" = "y" ] && sudo systemctl stop <service>`
- Fish: `read -l -P "Confirm stop service <service> [y/N]: " resp; test "$resp" = "y"; and sudo systemctl stop <service>`
- PowerShell: `Stop-Service -Name <service> -Confirm`
- CMD: `set /p resp=Confirm stop service <service> [y/N]: & if /I "%resp%"=="y" sc stop <service>`

## Intención: Restart a service
- Bash/Zsh: `sudo systemctl restart <service>`
- Fish: `sudo systemctl restart <service>`
- PowerShell: `Restart-Service -Name <service>`
- CMD: `sc stop <service> & sc start <service>`

## Intención: Enable service at startup
- Bash/Zsh: `sudo systemctl enable <service>`
- Fish: `sudo systemctl enable <service>`
- PowerShell: `Set-Service -Name <service> -StartupType Automatic`
- CMD: `sc config <service> start= auto`

## Intención: Disable service from startup
- Bash/Zsh: `sudo systemctl disable <service>`
- Fish: `sudo systemctl disable <service>`
- PowerShell: `Set-Service -Name <service> -StartupType Disabled`
- CMD: `sc config <service> start= disabled`

## Intención: View service logs in real time
- Bash/Zsh: `journalctl -u <service> -f`
- Fish: `journalctl -u <service> -f`
- PowerShell: `Get-WinEvent -LogName System | Where-Object { $_.ProviderName -like "*<service>*" }`
- CMD: `wevtutil qe System /q:"*[System[Provider[@Name='<service>']]]" /f:text`

# Networking Level 3

## Intención: List current local firewall rules
- Bash/Zsh: `sudo ufw status numbered`
- Fish: `sudo ufw status numbered`
- PowerShell: `Get-NetFirewallRule`
- CMD: `netsh advfirewall firewall show rule name=all`

## Intención: Open a TCP port in local firewall
- Bash/Zsh: `sudo ufw allow <port>/tcp`
- Fish: `sudo ufw allow <port>/tcp`
- PowerShell: `New-NetFirewallRule -DisplayName "<rule_name>" -Direction Inbound -Action Allow -Protocol TCP -LocalPort <port>`
- CMD: `netsh advfirewall firewall add rule name="<rule_name>" dir=in action=allow protocol=TCP localport=<port>`

## Intención: Delete a firewall rule with interactive confirmation
- Bash/Zsh: `read -p "Confirm deletion of rule for port <port> [y/N]: " resp && [ "$resp" = "y" ] && sudo ufw delete allow <port>/tcp`
- Fish: `read -l -P "Confirm deletion of rule for port <port> [y/N]: " resp; test "$resp" = "y"; and sudo ufw delete allow <port>/tcp`
- PowerShell: `Remove-NetFirewallRule -DisplayName "<rule_name>" -Confirm`
- CMD: `set /p resp=Confirm deletion of rule <rule_name> [y/N]: & if /I "%resp%"=="y" netsh advfirewall firewall delete rule name="<rule_name>"`

## Intención: Display local routing table
- Bash/Zsh: `ip route show`
- Fish: `ip route show`
- PowerShell: `Get-NetRoute`
- CMD: `route print`

## Intención: Add a static route
- Bash/Zsh: `sudo ip route add <destination_network>/<prefix> via <gateway> dev <interface>`
- Fish: `sudo ip route add <destination_network>/<prefix> via <gateway> dev <interface>`
- PowerShell: `New-NetRoute -DestinationPrefix "<destination_network>/<prefix>" -InterfaceAlias "<interface>" -NextHop <gateway>`
- CMD: `route add <destination_network> mask <mask> <gateway>`

## Intención: Delete a static route with interactive confirmation
- Bash/Zsh: `read -p "Confirm deletion of route <destination_network>/<prefix> [y/N]: " resp && [ "$resp" = "y" ] && sudo ip route del <destination_network>/<prefix>`
- Fish: `read -l -P "Confirm deletion of route <destination_network>/<prefix> [y/N]: " resp; test "$resp" = "y"; and sudo ip route del <destination_network>/<prefix>`
- PowerShell: `Remove-NetRoute -DestinationPrefix "<destination_network>/<prefix>" -Confirm`
- CMD: `set /p resp=Confirm deletion of route <destination_network> [y/N]: & if /I "%resp%"=="y" route delete <destination_network>`

## Intención: Detailed DNS resolution for A, AAAA and MX records
- Bash/Zsh: `dig <domain> A +short && dig <domain> AAAA +short && dig <domain> MX +short`
- Fish: `dig <domain> A +short; and dig <domain> AAAA +short; and dig <domain> MX +short`
- PowerShell: `Resolve-DnsName <domain> -Type A,AAAA,MX`
- CMD: `nslookup -type=A <domain> & nslookup -type=AAAA <domain> & nslookup -type=MX <domain>`

## Intención: Execute DNS resolution with specific server
- Bash/Zsh: `dig @<dns_server> <domain>`
- Fish: `dig @<dns_server> <domain>`
- PowerShell: `Resolve-DnsName <domain> -Server <dns_server> -DnsOnly -NoHostsFile`
- CMD: `nslookup <domain> <dns_server>`
