# Filesystem Operations

## Intención: Create a directory
- Bash/Zsh: `mkdir -p <directory_path>`
- Fish: `mkdir -p <directory_path>`
- PowerShell: `New-Item -ItemType Directory -Path <directory_path>`
- CMD: `mkdir <directory_path>`

## Intención: Create an empty file
- Bash/Zsh: `touch <file_path>`
- Fish: `touch <file_path>`
- PowerShell: `New-Item -ItemType File -Path <file_path>`
- CMD: `type nul > <file_path>`

## Intención: List detailed directory contents
- Bash/Zsh: `ls -la <directory_path>`
- Fish: `ls -la <directory_path>`
- PowerShell: `Get-ChildItem -Force -Path <directory_path>`
- CMD: `dir <directory_path>`

## Intención: Copy a file
- Bash/Zsh: `cp <source_path> <destination_path>`
- Fish: `cp <source_path> <destination_path>`
- PowerShell: `Copy-Item -Path <source_path> -Destination <destination_path>`
- CMD: `copy <source_path> <destination_path>`

## Intención: Copy directory recursively
- Bash/Zsh: `cp -r <source_directory> <destination_directory>`
- Fish: `cp -r <source_directory> <destination_directory>`
- PowerShell: `Copy-Item -Path <source_directory> -Destination <destination_directory> -Recurse`
- CMD: `xcopy <source_directory> <destination_directory> /E /I`

## Intención: Copy preserving attributes and timestamps
- Bash/Zsh: `cp -a <source_path> <destination_path>`
- Fish: `cp -a <source_path> <destination_path>`
- PowerShell: `robocopy <source_directory> <destination_directory> /E /COPY:DATS`
- CMD: `robocopy <source_directory> <destination_directory> /E /COPY:DATS`

## Intención: Move or rename file or directory
- Bash/Zsh: `mv <source_path> <destination_path>`
- Fish: `mv <source_path> <destination_path>`
- PowerShell: `Move-Item -Path <source_path> -Destination <destination_path>`
- CMD: `move <source_path> <destination_path>`

## Intención: Create symbolic link to a file
- Bash/Zsh: `ln -s <target_file_path> <link_path>`
- Fish: `ln -s <target_file_path> <link_path>`
- PowerShell: `New-Item -ItemType SymbolicLink -Path <link_path> -Target <target_file_path>`
- CMD: `mklink <link_path> <target_file_path>`

## Intención: Create symbolic link to a directory
- Bash/Zsh: `ln -s <target_directory> <link_path>`
- Fish: `ln -s <target_directory> <link_path>`
- PowerShell: `New-Item -ItemType SymbolicLink -Path <link_path> -Target <target_directory>`
- CMD: `mklink /D <link_path> <target_directory>`

## Intención: Create hard link to a file
- Bash/Zsh: `ln <target_file_path> <hard_link_path>`
- Fish: `ln <target_file_path> <hard_link_path>`
- PowerShell: `New-Item -ItemType HardLink -Path <hard_link_path> -Target <target_file_path>`
- CMD: `mklink /H <hard_link_path> <target_file_path>`

## Intención: Display file or directory metadata
- Bash/Zsh: `stat <path>`
- Fish: `stat <path>`
- PowerShell: `Get-Item <path> | Format-List *`
- CMD: `dir /q <path>`

## Intención: Resolve destination of a symbolic link
- Bash/Zsh: `readlink <link_path>`
- Fish: `readlink <link_path>`
- PowerShell: `(Get-Item <link_path>).Target`
- CMD: `fsutil reparsepoint query <link_path>`

## Intención: Delete a file with interactive confirmation
- Bash/Zsh: `rm -i <file_path>`
- Fish: `rm -i <file_path>`
- PowerShell: `Remove-Item -Path <file_path> -Confirm`
- CMD: `del /p <file_path>`

## Intención: Delete directory recursively with interactive confirmation
- Bash/Zsh: `rm -r -i <directory_path>`
- Fish: `rm -r -i <directory_path>`
- PowerShell: `Remove-Item -Path <directory_path> -Recurse -Confirm`
- CMD: `rmdir /s <directory_path>`

## Intención: View total directory size
- Bash/Zsh: `du -sh <directory_path>`
- Fish: `du -sh <directory_path>`
- PowerShell: `(Get-ChildItem -Path <directory_path> -Recurse | Measure-Object -Property Length -Sum).Sum`
- CMD: `powershell -Command "(Get-ChildItem -Path '<directory_path>' -Recurse | Measure-Object -Property Length -Sum).Sum"`

# Text Visualization

## Intención: Display full file content
- Bash/Zsh: `cat <file_path>`
- Fish: `cat <file_path>`
- PowerShell: `Get-Content <file_path>`
- CMD: `type <file_path>`

## Intención: Paginate a text file
- Bash/Zsh: `less <file_path>`
- Fish: `less <file_path>`
- PowerShell: `Get-Content <file_path> | more`
- CMD: `more < <file_path>`

## Intención: View first N lines of a file
- Bash/Zsh: `head -n <line_count> <file_path>`
- Fish: `head -n <line_count> <file_path>`
- PowerShell: `Get-Content <file_path> -TotalCount <line_count>`
- CMD: `powershell -Command "Get-Content '<file_path>' -TotalCount <line_count>"`

## Intención: View last N lines of a file
- Bash/Zsh: `tail -n <line_count> <file_path>`
- Fish: `tail -n <line_count> <file_path>`
- PowerShell: `Get-Content <file_path> -Tail <line_count>`
- CMD: `powershell -Command "Get-Content '<file_path>' -Tail <line_count>"`

## Intención: Follow a log in real time
- Bash/Zsh: `tail -f <log_path>`
- Fish: `tail -f <log_path>`
- PowerShell: `Get-Content <log_path> -Tail <line_count> -Wait`
- CMD: `powershell -Command "Get-Content '<log_path>' -Tail <line_count> -Wait"`

## Intención: Concatenate multiple files to standard output
- Bash/Zsh: `cat <file_path_1> <file_path_2> <file_path_n>`
- Fish: `cat <file_path_1> <file_path_2> <file_path_n>`
- PowerShell: `Get-Content <file_path_1>,<file_path_2>,<file_path_n>`
- CMD: `type <file_path_1> <file_path_2> <file_path_n>`

## Intención: Display numbered lines from a file
- Bash/Zsh: `nl -ba <file_path>`
- Fish: `nl -ba <file_path>`
- PowerShell: `$i=0; Get-Content <file_path> | ForEach-Object { $i++; "{0}`t{1}" -f $i, $_ }`
- CMD: `findstr /n "^" <file_path>`

## Intención: Count lines, words and bytes in a file
- Bash/Zsh: `wc <file_path>`
- Fish: `wc <file_path>`
- PowerShell: `Get-Content <file_path> | Measure-Object -Line -Word -Character`
- CMD: `for /f %c in ('type <file_path> ^| find /v /c ""') do @echo %c`

# Basic User Management

## Intención: View current user
- Bash/Zsh: `whoami`
- Fish: `whoami`
- PowerShell: `whoami`
- CMD: `whoami`

## Intención: View current user IDs and groups
- Bash/Zsh: `id`
- Fish: `id`
- PowerShell: `whoami /groups`
- CMD: `whoami /groups`

## Intención: Change current user password
- Bash/Zsh: `passwd`
- Fish: `passwd`
- PowerShell: `Set-LocalUser -Name <user> -Password (Read-Host -AsSecureString)`
- CMD: `net user <user> *`

## Intención: List active sessions on the system
- Bash/Zsh: `w`
- Fish: `w`
- PowerShell: `query user`
- CMD: `query user`

## Intención: View recent login history
- Bash/Zsh: `last -n <record_count>`
- Fish: `last -n <record_count>`
- PowerShell: `Get-WinEvent -FilterHashtable @{LogName='Security';Id=4624} -MaxEvents <record_count>`
- CMD: `wevtutil qe Security /q:"*[System[(EventID=4624)]]" /c:<record_count> /f:text`

## Intención: Execute a command as another user
- Bash/Zsh: `su - <user> -c "<command>"`
- Fish: `su - <user> -c "<command>"`
- PowerShell: `Start-Process powershell -Credential <user> -ArgumentList "-Command <command>"`
- CMD: `runas /user:<user> "<command>"`

# Basic Processes

## Intención: List all active processes
- Bash/Zsh: `ps aux`
- Fish: `ps aux`
- PowerShell: `Get-Process`
- CMD: `tasklist`

## Intención: Search processes by name
- Bash/Zsh: `ps aux | grep "<process_name>"`
- Fish: `ps aux | grep "<process_name>"`
- PowerShell: `Get-Process -Name <process_name>`
- CMD: `tasklist | findstr /i "<process_name>"`

## Intención: View process details by PID
- Bash/Zsh: `ps -fp <pid>`
- Fish: `ps -fp <pid>`
- PowerShell: `Get-Process -Id <pid> | Format-List *`
- CMD: `tasklist /fi "PID eq <pid>"`

## Intención: Terminate a process by PID with interactive confirmation
- Bash/Zsh: `read -p "Confirm termination of PID <pid> [y/N]: " resp && [ "$resp" = "y" ] && kill -TERM <pid>`
- Fish: `read -l -P "Confirm termination of PID <pid> [y/N]: " resp; test "$resp" = "y"; and kill -TERM <pid>`
- PowerShell: `Stop-Process -Id <pid> -Confirm`
- CMD: `set /p resp=Confirm termination of PID <pid> [y/N]: & if /I "%resp%"=="y" taskkill /PID <pid>`

## Intención: Terminate processes by name with interactive confirmation
- Bash/Zsh: `read -p "Confirm termination of <process_name> [y/N]: " resp && [ "$resp" = "y" ] && pkill -TERM -x "<process_name>"`
- Fish: `read -l -P "Confirm termination of <process_name> [y/N]: " resp; test "$resp" = "y"; and pkill -TERM -x "<process_name>"`
- PowerShell: `Stop-Process -Name <process_name> -Confirm`
- CMD: `set /p resp=Confirm termination of <process_name> [y/N]: & if /I "%resp%"=="y" taskkill /IM <process_name>.exe`

## Intención: View hierarchical process tree
- Bash/Zsh: `ps -ef --forest`
- Fish: `ps -ef --forest`
- PowerShell: `Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,Name`
- CMD: `wmic process get ProcessId,ParentProcessId,Name`

## Intención: Wait for process completion
- Bash/Zsh: `wait <pid>`
- Fish: `wait <pid>`
- PowerShell: `Wait-Process -Id <pid>`
- CMD: `powershell -Command "Wait-Process -Id <pid>"`

# Basic Networking

## Intención: Verify ICMP connectivity to a host
- Bash/Zsh: `ping -c <packet_count> <host_or_ip>`
- Fish: `ping -c <packet_count> <host_or_ip>`
- PowerShell: `Test-Connection -Count <packet_count> -ComputerName <host_or_ip>`
- CMD: `ping -n <packet_count> <host_or_ip>`

## Intención: Trace network route to a host
- Bash/Zsh: `traceroute <host_or_ip>`
- Fish: `traceroute <host_or_ip>`
- PowerShell: `tracert <host_or_ip>`
- CMD: `tracert <host_or_ip>`

## Intención: Download a file via HTTP/HTTPS
- Bash/Zsh: `curl -L "<url>" -o <output_file_path>`
- Fish: `curl -L "<url>" -o <output_file_path>`
- PowerShell: `Invoke-WebRequest -Uri "<url>" -OutFile <output_file_path>`
- CMD: `curl -L "<url>" -o <output_file_path>`

## Intención: Get local IPv4 addresses
- Bash/Zsh: `hostname -I`
- Fish: `hostname -I`
- PowerShell: `Get-NetIPAddress -AddressFamily IPv4 | Select-Object -ExpandProperty IPAddress`
- CMD: `ipconfig`

## Intención: List network interfaces and status
- Bash/Zsh: `ip addr show`
- Fish: `ip addr show`
- PowerShell: `Get-NetAdapter`
- CMD: `netsh interface show interface`

## Intención: Resolve a DNS name to IP
- Bash/Zsh: `nslookup <domain>`
- Fish: `nslookup <domain>`
- PowerShell: `Resolve-DnsName <domain>`
- CMD: `nslookup <domain>`

## Intención: View active network connections
- Bash/Zsh: `netstat -tulpn`
- Fish: `netstat -tulpn`
- PowerShell: `Get-NetTCPConnection`
- CMD: `netstat -ano`

# Environment Variables and PATH

## Intención: Set a non-exported session variable
- Bash/Zsh: `<variable_name>=<variable_value>`
- Fish: `set <variable_name> <variable_value>`
- PowerShell: `$<variable_name> = "<variable_value>"`
- CMD: `set <variable_name>=<variable_value>`

## Intención: Export an environment variable for child processes
- Bash/Zsh: `export <variable_name>="<variable_value>"`
- Fish: `set -x <variable_name> "<variable_value>"`
- PowerShell: `$env:<variable_name> = "<variable_value>"`
- CMD: `set <variable_name>=<variable_value>`

## Intención: Read the value of an environment variable
- Bash/Zsh: `printenv <variable_name>`
- Fish: `printenv <variable_name>`
- PowerShell: `(Get-Item -Path Env:<variable_name>).Value`
- CMD: `echo %<variable_name>%`

## Intención: Delete a variable from the session
- Bash/Zsh: `read -p "Confirm deletion of variable <variable_name> [y/N]: " resp && [ "$resp" = "y" ] && unset <variable_name>`
- Fish: `read -l -P "Confirm deletion of variable <variable_name> [y/N]: " resp; test "$resp" = "y"; and set -e <variable_name>`
- PowerShell: `Remove-Item -Path Env:<variable_name> -Confirm`
- CMD: `set /p resp=Confirm deletion of variable <variable_name> [y/N]: & if /I "%resp%"=="y" set <variable_name>=`

## Intención: List all environment variables
- Bash/Zsh: `printenv`
- Fish: `printenv`
- PowerShell: `Get-ChildItem Env:`
- CMD: `set`

## Intención: Add a directory to the current session PATH
- Bash/Zsh: `export PATH="$PATH:<directory_path>"`
- Fish: `fish_add_path <directory_path>`
- PowerShell: `$env:PATH += ";<directory_path>"`
- CMD: `set PATH=%PATH%;<directory_path>`

## Intención: Persist a directory in the user PATH
- Bash/Zsh: `echo 'export PATH="$PATH:<directory_path>"' >> <rc_file_path>`
- Fish: `set -U fish_user_paths <directory_path> $fish_user_paths`
- PowerShell: `[Environment]::SetEnvironmentVariable('Path', $env:Path + ';<directory_path>', 'User')`
- CMD: `setx PATH "%PATH%;<directory_path>"`

## Intención: Display the current PATH
- Bash/Zsh: `echo "$PATH"`
- Fish: `echo $PATH`
- PowerShell: `$env:PATH`
- CMD: `echo %PATH%`
