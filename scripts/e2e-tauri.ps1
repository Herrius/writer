# E2E de escritorio Windows: tauri-driver + msedgedriver.
# Ejecuta los dos escenarios reales (Ctrl+S y cierre de ventana) sobre WebView2.
$ErrorActionPreference = 'Stop'

$raiz = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $raiz

function Ejecutar {
    param([string]$Comando, [string[]]$Argumentos)
    & $Comando @Argumentos
    if ($LASTEXITCODE -ne 0) {
        throw "falló: $Comando $($Argumentos -join ' ')"
    }
}

if (-not (Get-Command tauri-driver -ErrorAction SilentlyContinue)) {
    throw 'falta tauri-driver (cargo install tauri-driver --version 2.0.6 --locked)'
}
if (-not $env:MSEDGEDRIVER -or -not (Test-Path $env:MSEDGEDRIVER)) {
    throw 'falta MSEDGEDRIVER con la ruta a msedgedriver.exe'
}

$baseTemporal = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [System.IO.Path]::GetTempPath() }
$temporal = Join-Path $baseTemporal ("writer-e2e-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $temporal -Force | Out-Null
$proyecto = Join-Path $temporal 'novela'
$app = Join-Path $raiz 'apps/desktop/src-tauri/target/release/writer-desktop.exe'
$cli = Join-Path $raiz 'target/debug/writer.exe'
$logDriver = Join-Path $temporal 'tauri-driver.log'
$driver = $null

function Puerto-Abierto {
    param([int]$Puerto)
    $cliente = New-Object System.Net.Sockets.TcpClient
    try {
        $cliente.Connect('127.0.0.1', $Puerto)
        return $true
    } catch {
        return $false
    } finally {
        $cliente.Dispose()
    }
}

function Detener-App {
    Get-Process -ErrorAction SilentlyContinue |
        Where-Object { $_.MainWindowTitle -eq 'writer' } |
        Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 750
}

try {
    Ejecutar 'cargo' @('build', '--quiet', '--locked', '--bin', 'writer')
    Ejecutar $cli @('init', $proyecto, '--titulo', 'E2E Tauri')

    # La feature `e2e` autoabre el proyecto; la app de producción no la lleva.
    Ejecutar 'pnpm' @('--dir', 'apps/desktop', 'exec', 'tauri', 'build', '--no-bundle', '--features', 'e2e')
    if (-not (Test-Path $app)) {
        $alternativa = Join-Path $raiz 'apps/desktop/src-tauri/target/release/writer.exe'
        if (Test-Path $alternativa) {
            $app = $alternativa
        } else {
            throw "no se generó la app E2E: $app"
        }
    }

    $env:WRITER_E2E_PROJECT = $proyecto
    $env:WRITER_DESKTOP_BIN = $app
    $env:WRITER_E2E_PORT = '4444'

    $driver = Start-Process -FilePath 'tauri-driver' -ArgumentList @(
        '--port', '4444',
        '--native-port', '4445',
        '--native-host', '127.0.0.1',
        '--native-driver', $env:MSEDGEDRIVER
    ) -PassThru -RedirectStandardOutput $logDriver -RedirectStandardError "$logDriver.err"

    $listo = $false
    for ($intento = 0; $intento -lt 80; $intento += 1) {
        if ($driver.HasExited) {
            throw "tauri-driver terminó antes de escuchar:`n$(Get-Content $logDriver -Raw -ErrorAction SilentlyContinue)"
        }
        if (Puerto-Abierto 4444) {
            $listo = $true
            break
        }
        Start-Sleep -Milliseconds 250
    }
    if (-not $listo) {
        throw "tauri-driver no escuchó en 4444:`n$(Get-Content $logDriver -Raw -ErrorAction SilentlyContinue)"
    }

    $env:WRITER_E2E_MODE = 'guardar'
    Ejecutar 'node' @('e2e-tauri/tauri.test.mjs')

    Detener-App
    $env:WRITER_E2E_MODE = 'cerrar'
    Ejecutar 'node' @('e2e-tauri/tauri.test.mjs')
} catch {
    Write-Host '--- tauri-driver stdout ---'
    Get-Content $logDriver -Raw -ErrorAction SilentlyContinue
    Write-Host '--- tauri-driver stderr ---'
    Get-Content "$logDriver.err" -Raw -ErrorAction SilentlyContinue
    throw
} finally {
    if ($driver -and -not $driver.HasExited) {
        Stop-Process -Id $driver.Id -Force -ErrorAction SilentlyContinue
    }
    Detener-App
    Remove-Item -Recurse -Force $temporal -ErrorAction SilentlyContinue
}
