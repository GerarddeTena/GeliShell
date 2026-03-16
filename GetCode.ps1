param (
    [Parameter(Mandatory=$true)]
    [string]$SourcePath
)

# Verificar si la ruta existe
if (-not (Test-Path $SourcePath)) {
    Write-Error "La ruta especificada no existe: $SourcePath"
    exit
}

# Obtener todos los archivos de forma recursiva
# Excluimos carpetas comunes de dependencias o binarios para evitar basura en la consola
$excludeDirs = @("*node_modules*", "*.git*", "*bin*", "*obj*", "*.next*", "*dist*")

$files = Get-ChildItem -Path $SourcePath -Recurse -File | Where-Object { 
    $fullName = $_.FullName
    $shouldExclude = $false
    foreach ($dir in $excludeDirs) {
        if ($fullName -like $dir) { $shouldExclude = $true; break }
    }
    -not $shouldExclude
}

foreach ($file in $files) {
    Write-Host "`n" + ("=" * 80) -ForegroundColor Cyan
    Write-Host "ARCHIVO: $($file.FullName)" -ForegroundColor Yellow
    Write-Host ("=" * 80) -ForegroundColor Cyan
    
    try {
        # Intentamos leer el contenido. Si es un binario, fallará o mostrará caracteres extraños, 
        # pero para archivos de texto/código funcionará perfecto.
        Get-Content -Path $file.FullName -Raw
    }
    catch {
        Write-Warning "No se pudo leer el archivo: $($file.Name)"
    }
    
    Write-Host "`n" + ("-" * 80) -ForegroundColor Gray
}