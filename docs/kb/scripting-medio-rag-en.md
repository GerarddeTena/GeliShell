# CLI Control Flow

## Intención: Execute a for loop over an explicit list
- Bash/Zsh: `for item in <space_separated_list>; do <action_command> "$item"; done`
- Fish: `for item in <space_separated_list>; <action_command> $item; end`
- PowerShell: `foreach ($item in <list>) { <action_command> $item }`
- CMD: `for %i in (<list>) do <action_command> %i`

## Intención: Execute a for loop over a numeric range
- Bash/Zsh: `for i in $(seq <start> <end>); do <action_command> "$i"; done`
- Fish: `for i in (seq <start> <end>); <action_command> $i; end`
- PowerShell: `<start>..<end> | ForEach-Object { <action_command> $_ }`
- CMD: `for /l %i in (<start>,1,<end>) do <action_command> %i`

## Intención: Execute a periodic while loop
- Bash/Zsh: `while true; do <action_command>; sleep <seconds>; done`
- Fish: `while true; <action_command>; sleep <seconds>; end`
- PowerShell: `while ($true) { <action_command>; Start-Sleep -Seconds <seconds> }`
- CMD: `:loop & <action_command> & timeout /t <seconds> >nul & goto loop`

## Intención: Iterate over command output
- Bash/Zsh: `for item in $(<generator_command>); do <action_command> "$item"; done`
- Fish: `for item in (<generator_command>); <action_command> $item; end`
- PowerShell: `<generator_command> | ForEach-Object { <action_command> $_ }`
- CMD: `for /f %i in ('<generator_command>') do <action_command> %i`

## Intención: Iterate found files and execute action
- Bash/Zsh: `find <base_path> -type f -name "<pattern>" -print0 | xargs -0 -I {} <action_command> "{}"`
- Fish: `find <base_path> -type f -name "<pattern>" -print0 | xargs -0 -I {} <action_command> "{}"`
- PowerShell: `Get-ChildItem -Path <base_path> -Recurse -File -Filter "<pattern>" | ForEach-Object { <action_command> $_.FullName }`
- CMD: `for /r <base_path> %f in (<pattern>) do <action_command> "%f"`

## Intención: Execute an action N times
- Bash/Zsh: `for i in $(seq 1 <repetitions>); do <action_command>; done`
- Fish: `for i in (seq 1 <repetitions>); <action_command>; end`
- PowerShell: `1..<repetitions> | ForEach-Object { <action_command> }`
- CMD: `for /l %i in (1,1,<repetitions>) do <action_command>`

# CLI Conditionals

## Intención: Execute a command only if the previous one succeeded
- Bash/Zsh: `<command_1> && <command_2>`
- Fish: `<command_1>; and <command_2>`
- PowerShell: `<command_1>; if ($?) { <command_2> }`
- CMD: `<command_1> && <command_2>`

## Intención: Execute an alternative command if the previous one fails
- Bash/Zsh: `<command_1> || <command_2>`
- Fish: `<command_1>; or <command_2>`
- PowerShell: `<command_1>; if (-not $?) { <command_2> }`
- CMD: `<command_1> || <command_2>`

## Intención: Check file existence before operating
- Bash/Zsh: `[ -f <file_path> ] && <exists_command> || <does_not_exist_command>`
- Fish: `test -f <file_path>; and <exists_command>; or <does_not_exist_command>`
- PowerShell: `if (Test-Path -Path <file_path> -PathType Leaf) { <exists_command> } else { <does_not_exist_command> }`
- CMD: `if exist <file_path> (<exists_command>) else (<does_not_exist_command>)`

## Intención: Check directory existence before operating
- Bash/Zsh: `[ -d <directory_path> ] && <exists_command> || <does_not_exist_command>`
- Fish: `test -d <directory_path>; and <exists_command>; or <does_not_exist_command>`
- PowerShell: `if (Test-Path -Path <directory_path> -PathType Container) { <exists_command> } else { <does_not_exist_command> }`
- CMD: `if exist <directory_path>\ (<exists_command>) else (<does_not_exist_command>)`

## Intención: Check if a variable is defined
- Bash/Zsh: `[ -n "$(printenv <variable_name>)" ] && <defined_command> || <empty_command>`
- Fish: `set -q <variable_name>; and <defined_command>; or <empty_command>`
- PowerShell: `if ($env:<variable_name>) { <defined_command> } else { <empty_command> }`
- CMD: `if defined <variable_name> (<defined_command>) else (<empty_command>)`

## Intención: Compare two numeric values
- Bash/Zsh: `[ <value_1> -gt <value_2> ] && <greater_command> || <not_greater_command>`
- Fish: `test <value_1> -gt <value_2>; and <greater_command>; or <not_greater_command>`
- PowerShell: `if (<value_1> -gt <value_2>) { <greater_command> } else { <not_greater_command> }`
- CMD: `if <value_1> GTR <value_2> (<greater_command>) else (<not_greater_command>)`

# Compression and Packaging

## Intención: Create tar file
- Bash/Zsh: `tar -cvf <tar_file> <source_path>`
- Fish: `tar -cvf <tar_file> <source_path>`
- PowerShell: `tar -cvf <tar_file> <source_path>`
- CMD: `tar -cvf <tar_file> <source_path>`

## Intención: Extract tar file
- Bash/Zsh: `tar -xvf <tar_file> -C <destination_path>`
- Fish: `tar -xvf <tar_file> -C <destination_path>`
- PowerShell: `tar -xvf <tar_file> -C <destination_path>`
- CMD: `tar -xvf <tar_file> -C <destination_path>`

## Intención: Create tar.gz file
- Bash/Zsh: `tar -czvf <targz_file> <source_path>`
- Fish: `tar -czvf <targz_file> <source_path>`
- PowerShell: `tar -czvf <targz_file> <source_path>`
- CMD: `tar -czvf <targz_file> <source_path>`

## Intención: Extract tar.gz file
- Bash/Zsh: `tar -xzvf <targz_file> -C <destination_path>`
- Fish: `tar -xzvf <targz_file> -C <destination_path>`
- PowerShell: `tar -xzvf <targz_file> -C <destination_path>`
- CMD: `tar -xzvf <targz_file> -C <destination_path>`

## Intención: Compress file or directory to zip
- Bash/Zsh: `zip -r <zip_file> <source_path>`
- Fish: `zip -r <zip_file> <source_path>`
- PowerShell: `Compress-Archive -Path <source_path> -DestinationPath <zip_file>`
- CMD: `powershell -Command "Compress-Archive -Path '<source_path>' -DestinationPath '<zip_file>'"`

## Intención: Extract zip to current directory
- Bash/Zsh: `unzip <zip_file>`
- Fish: `unzip <zip_file>`
- PowerShell: `Expand-Archive -Path <zip_file> -DestinationPath .`
- CMD: `powershell -Command "Expand-Archive -Path '<zip_file>' -DestinationPath '.'"`

## Intención: Extract zip to specific directory
- Bash/Zsh: `unzip <zip_file> -d <destination_path>`
- Fish: `unzip <zip_file> -d <destination_path>`
- PowerShell: `Expand-Archive -Path <zip_file> -DestinationPath <destination_path>`
- CMD: `powershell -Command "Expand-Archive -Path '<zip_file>' -DestinationPath '<destination_path>'"`

## Intención: List compressed file contents without extracting
- Bash/Zsh: `tar -tvf <tar_or_targz_file>`
- Fish: `tar -tvf <tar_or_targz_file>`
- PowerShell: `tar -tvf <tar_or_targz_file>`
- CMD: `tar -tvf <tar_or_targz_file>`

# Advanced Search

## Intención: Find files by extension
- Bash/Zsh: `find <base_path> -type f -name "*.<extension>"`
- Fish: `find <base_path> -type f -name "*.<extension>"`
- PowerShell: `Get-ChildItem -Path <base_path> -Recurse -File -Filter "*.<extension>"`
- CMD: `for /r <base_path> %f in (*.<extension>) do @echo %f`

## Intención: Find files larger than a size
- Bash/Zsh: `find <base_path> -type f -size +<size>`
- Fish: `find <base_path> -type f -size +<size>`
- PowerShell: `Get-ChildItem -Path <base_path> -Recurse -File | Where-Object { $_.Length -gt <size_bytes> }`
- CMD: `forfiles /p <base_path> /s /m *.* /c "cmd /c if @fsize GEQ <size_bytes> echo @path"`

## Intención: Find files modified in the last N days
- Bash/Zsh: `find <base_path> -type f -mtime -<days>`
- Fish: `find <base_path> -type f -mtime -<days>`
- PowerShell: `Get-ChildItem -Path <base_path> -Recurse -File | Where-Object { $_.LastWriteTime -gt (Get-Date).AddDays(-<days>) }`
- CMD: `forfiles /p <base_path> /s /d -<days> /c "cmd /c echo @path"`

## Intención: Find files older than N days
- Bash/Zsh: `find <base_path> -type f -mtime +<days>`
- Fish: `find <base_path> -type f -mtime +<days>`
- PowerShell: `Get-ChildItem -Path <base_path> -Recurse -File | Where-Object { $_.LastWriteTime -lt (Get-Date).AddDays(-<days>) }`
- CMD: `forfiles /p <base_path> /s /d -<days> /c "cmd /c echo @path"`

## Intención: Execute a command on each found file
- Bash/Zsh: `find <base_path> -type f -name "<pattern>" -print0 | xargs -0 -I {} <action_command> "{}"`
- Fish: `find <base_path> -type f -name "<pattern>" -print0 | xargs -0 -I {} <action_command> "{}"`
- PowerShell: `Get-ChildItem -Path <base_path> -Recurse -File -Filter "<pattern>" | ForEach-Object { <action_command> $_.FullName }`
- CMD: `for /r <base_path> %f in (<pattern>) do <action_command> "%f"`

## Intención: Search by name and show size and date
- Bash/Zsh: `find <base_path> -type f -name "<pattern>" -exec ls -lh {} \;`
- Fish: `find <base_path> -type f -name "<pattern>" -exec ls -lh {} \;`
- PowerShell: `Get-ChildItem -Path <base_path> -Recurse -File -Filter "<pattern>" | Select-Object FullName,Length,LastWriteTime`
- CMD: `for /r <base_path> %f in (<pattern>) do @for %s in ("%f") do @echo %f %~zs %~ts`

## Intención: Search by extension and sort by modification date
- Bash/Zsh: `find <base_path> -type f -name "*.<extension>" -printf "%T@ %p\n" | sort -nr`
- Fish: `find <base_path> -type f -name "*.<extension>" -printf "%T@ %p\n" | sort -nr`
- PowerShell: `Get-ChildItem -Path <base_path> -Recurse -File -Filter "*.<extension>" | Sort-Object LastWriteTime -Descending`
- CMD: `for /f "delims=" %f in ('dir <base_path>\*.<extension> /s /b') do @echo %~tf %f`

# Networking Level 2

## Intención: Execute HTTP GET request with headers
- Bash/Zsh: `curl -X GET "<url>" -H "Authorization: Bearer <token>" -H "Accept: application/json"`
- Fish: `curl -X GET "<url>" -H "Authorization: Bearer <token>" -H "Accept: application/json"`
- PowerShell: `Invoke-RestMethod -Method Get -Uri "<url>" -Headers @{ Authorization = "Bearer <token>"; Accept = "application/json" }`
- CMD: `curl -X GET "<url>" -H "Authorization: Bearer <token>" -H "Accept: application/json"`

## Intención: Execute HTTP POST request with JSON
- Bash/Zsh: `curl -X POST "<url>" -H "Content-Type: application/json" -H "Authorization: Bearer <token>" -d '<json_body>'`
- Fish: `curl -X POST "<url>" -H "Content-Type: application/json" -H "Authorization: Bearer <token>" -d '<json_body>'`
- PowerShell: `Invoke-RestMethod -Method Post -Uri "<url>" -Headers @{ Authorization = "Bearer <token>" } -ContentType "application/json" -Body '<json_body>'`
- CMD: `curl -X POST "<url>" -H "Content-Type: application/json" -H "Authorization: Bearer <token>" -d "<json_body>"`

## Intención: Execute HTTP PUT request with JSON
- Bash/Zsh: `curl -X PUT "<url>" -H "Content-Type: application/json" -H "Authorization: Bearer <token>" -d '<json_body>'`
- Fish: `curl -X PUT "<url>" -H "Content-Type: application/json" -H "Authorization: Bearer <token>" -d '<json_body>'`
- PowerShell: `Invoke-RestMethod -Method Put -Uri "<url>" -Headers @{ Authorization = "Bearer <token>" } -ContentType "application/json" -Body '<json_body>'`
- CMD: `curl -X PUT "<url>" -H "Content-Type: application/json" -H "Authorization: Bearer <token>" -d "<json_body>"`

## Intención: Save HTTP response to a file
- Bash/Zsh: `curl -X GET "<url>" -H "Accept: application/json" -o <response_path>`
- Fish: `curl -X GET "<url>" -H "Accept: application/json" -o <response_path>`
- PowerShell: `Invoke-WebRequest -Uri "<url>" -Headers @{ Accept = "application/json" } -OutFile <response_path>`
- CMD: `curl -X GET "<url>" -H "Accept: application/json" -o <response_path>`

## Intención: List local ports in LISTEN state
- Bash/Zsh: `ss -lntup`
- Fish: `ss -lntup`
- PowerShell: `Get-NetTCPConnection -State Listen`
- CMD: `netstat -ano | findstr LISTENING`

## Intención: Verify if a specific local port is open
- Bash/Zsh: `nc -zv 127.0.0.1 <port>`
- Fish: `nc -zv 127.0.0.1 <port>`
- PowerShell: `Test-NetConnection -ComputerName 127.0.0.1 -Port <port>`
- CMD: `powershell -Command "Test-NetConnection -ComputerName 127.0.0.1 -Port <port>"`

## Intención: Scan a basic local TCP port range
- Bash/Zsh: `for p in $(seq <port_start> <port_end>); do (echo >/dev/tcp/127.0.0.1/$p) >/dev/null 2>&1 && echo $p; done`
- Fish: `for p in (seq <port_start> <port_end>); nc -z 127.0.0.1 $p; and echo $p; end`
- PowerShell: `<port_start>..<port_end> | ForEach-Object { if ((Test-NetConnection -ComputerName 127.0.0.1 -Port $_ -WarningAction SilentlyContinue).TcpTestSucceeded) { $_ } }`
- CMD: `for /l %p in (<port_start>,1,<port_end>) do @powershell -Command "if ((Test-NetConnection -ComputerName 127.0.0.1 -Port %p -WarningAction SilentlyContinue).TcpTestSucceeded) { Write-Output %p }"`

## Intención: Correlate open ports with processes
- Bash/Zsh: `lsof -i -P -n | grep LISTEN`
- Fish: `lsof -i -P -n | grep LISTEN`
- PowerShell: `Get-NetTCPConnection -State Listen | Select-Object LocalAddress,LocalPort,OwningProcess`
- CMD: `netstat -ano`

# Permissions and Ownership

## Intención: View file or directory permissions
- Bash/Zsh: `ls -l <path>`
- Fish: `ls -l <path>`
- PowerShell: `Get-Acl <path> | Format-List`
- CMD: `icacls <path>`

## Intención: Change permissions using numeric mode
- Bash/Zsh: `chmod <octal_mode> <path>`
- Fish: `chmod <octal_mode> <path>`
- PowerShell: `icacls <path> /grant <user>:(RX)`
- CMD: `icacls <path> /grant <user>:(RX)`

## Intención: Add execute permission to owner
- Bash/Zsh: `chmod u+x <path>`
- Fish: `chmod u+x <path>`
- PowerShell: `icacls <path> /grant <user>:(X)`
- CMD: `icacls <path> /grant <user>:(X)`

## Intención: Change file or directory owner
- Bash/Zsh: `chown <user>:<group> <path>`
- Fish: `chown <user>:<group> <path>`
- PowerShell: `icacls <path> /setowner <user>`
- CMD: `icacls <path> /setowner <user>`

## Intención: Change file or directory group owner
- Bash/Zsh: `chgrp <group> <path>`
- Fish: `chgrp <group> <path>`
- PowerShell: `icacls <path> /grant <group>:(M)`
- CMD: `icacls <path> /grant <group>:(M)`

## Intención: Grant Modify permission via ACL
- Bash/Zsh: `setfacl -m u:<user>:rwX <path>`
- Fish: `setfacl -m u:<user>:rwX <path>`
- PowerShell: `$acl=Get-Acl <path>; $rule=New-Object System.Security.AccessControl.FileSystemAccessRule('<user>','Modify','Allow'); $acl.SetAccessRule($rule); Set-Acl <path> $acl`
- CMD: `icacls <path> /grant <user>:(M)`

## Intención: Revoke ACL permissions with interactive confirmation
- Bash/Zsh: `read -p "Confirm revoke ACL of <user> on <path> [y/N]: " resp && [ "$resp" = "y" ] && setfacl -x u:<user> <path>`
- Fish: `read -l -P "Confirm revoke ACL of <user> on <path> [y/N]: " resp; test "$resp" = "y"; and setfacl -x u:<user> <path>`
- PowerShell: `if ((Read-Host "Confirm revoke ACL of <user> on <path> [y/N]") -eq "y") { icacls <path> /remove <user> }`
- CMD: `set /p resp=Confirm revoke ACL of <user> on <path> [y/N]: & if /I "%resp%"=="y" icacls <path> /remove <user>`
