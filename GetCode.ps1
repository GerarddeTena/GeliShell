# How to use : .\GetCode.ps1 -SourcePath "C:\TuProyecto\App"
param (
    [Parameter(Mandatory=$true)]
    [string]$SourcePath
)

# Ruta donde guardaremos el archivo de salida
$outputFile = "code.txt"

# Limpiar el archivo previo para no duplicar contenido
if (Test-Path $outputFile) {
    Clear-Content $outputFile
}

# Verificar si la ruta existe
if (-not (Test-Path $SourcePath)) {
    Write-Error "La ruta especificada no existe: $SourcePath"
    exit
}

# Directorios a excluir
$excludeDirs = @("*node_modules*", "*.git*", "*bin*", "*obj*", "*.next*", "*dist*", "*target*")

# Obtener archivos filtrando directorios excluidos
$files = Get-ChildItem -Path $SourcePath -Recurse -File | Where-Object { 
    $fullName = $_.FullName
    $shouldExclude = $false
    foreach ($dir in $excludeDirs) {
        if ($fullName -like $dir) { $shouldExclude = $true; break }
    }
    -not $shouldExclude
}

foreach ($file in $files) {

    $header = "`n" + ("=" * 80) + "`nARCHIVO: $($file.FullName)`n" + ("=" * 80) + "`n"

    # Mostrar en pantalla
    Write-Host $header -ForegroundColor Cyan

    # Añadir al archivo
    Add-Content -Path $outputFile -Value $header

    try {
        $content = Get-Content -Path $file.FullName -Raw

        # Mostrar
        Write-Host $content

        # Guardar
        Add-Content -Path $outputFile -Value $content
    }
    catch {
        $warn = "No se pudo leer el archivo: $($file.Name)"
        Write-Warning $warn
        Add-Content -Path $outputFile -Value $warn
    }

    $separator = "`n" + ("-" * 80) + "`n"
    Write-Host $separator
    Add-Content -Path $outputFile -Value $separator
}

Write-Host "`nProceso completado. Código extraído en: $outputFile" -ForegroundColor Green