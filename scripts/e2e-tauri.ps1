# E2E de escritorio Windows: WebView2 + CDP + Playwright.
# No usa tauri-driver/msedgedriver: en CI fallan con DevToolsActivePort.
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

$baseTemporal = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [System.IO.Path]::GetTempPath() }
$temporal = Join-Path $baseTemporal ("writer-e2e-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $temporal -Force | Out-Null
$proyecto = Join-Path $temporal 'novela'
$app = Join-Path $raiz 'apps/desktop/src-tauri/target/debug/writer-desktop.exe'
$cli = Join-Path $raiz 'target/debug/writer.exe'
$configE2e = Join-Path $raiz 'apps/desktop/src-tauri/tauri.e2e.conf.json'
$proceso = $null

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
    if ($proceso -and -not $proceso.HasExited) {
        Stop-Process -Id $proceso.Id -Force -ErrorAction SilentlyContinue
    }
    Start-Sleep -Milliseconds 750
}

function Ejecutar-Escenario {
    param([string]$Modo)
    $env:WRITER_E2E_PROJECT = $proyecto
    $env:WRITER_E2E_MODE = $Modo
    $env:WRITER_CDP_PORT = '9222'
    $env:WEBVIEW2_USER_DATA_FOLDER = Join-Path $temporal "webview-$Modo"

    $stdout = Join-Path $temporal "app-$Modo.out"
    $stderr = Join-Path $temporal "app-$Modo.err"
    $script:proceso = Start-Process -FilePath $app -PassThru `
        -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    $env:WRITER_DESKTOP_PID = [string]$proceso.Id

    $listo = $false
    for ($intento = 0; $intento -lt 120; $intento += 1) {
        $proceso.Refresh()
        if ($proceso.HasExited) {
            Write-Host "--- app stdout ($Modo) ---"
            Get-Content $stdout -Raw -ErrorAction SilentlyContinue
            Write-Host "--- app stderr ($Modo) ---"
            Get-Content $stderr -Raw -ErrorAction SilentlyContinue
            throw "la app terminó antes de abrir CDP (código $($proceso.ExitCode))"
        }
        if (Puerto-Abierto 9222) {
            $listo = $true
            break
        }
        Start-Sleep -Milliseconds 250
    }
    if (-not $listo) {
        throw "WebView2 no abrió el puerto CDP 9222"
    }

    Ejecutar 'node' @('e2e-tauri/tauri.windows-cdp.mjs')

    if ($Modo -eq 'guardar') {
        Detener-App
    } else {
        # El propio test espera a que WM_CLOSE guarde y termine el proceso.
        for ($intento = 0; $intento -lt 80; $intento += 1) {
            $proceso.Refresh()
            if ($proceso.HasExited) {
                break
            }
            Start-Sleep -Milliseconds 250
        }
        if (-not $proceso.HasExited) {
            throw 'el proceso de writer sigue vivo después del cierre'
        }
    }
}

try {
    Ejecutar 'cargo' @('build', '--quiet', '--locked', '--bin', 'writer')
    Ejecutar $cli @('init', $proyecto, '--titulo', 'E2E Tauri')

    Ejecutar 'pnpm' @(
        '--dir', 'apps/desktop', 'exec', 'tauri', 'build',
        '--debug', '--no-bundle', '--features', 'e2e', '--config', $configE2e
    )
    if (-not (Test-Path $app)) {
        $alternativa = Join-Path $raiz 'apps/desktop/src-tauri/target/debug/writer.exe'
        if (Test-Path $alternativa) {
            $app = $alternativa
        } else {
            throw "no se generó la app E2E: $app"
        }
    }

    Ejecutar-Escenario 'guardar'
    Ejecutar-Escenario 'cerrar'
} catch {
    Write-Host "E2E Windows: $($_.Exception.Message)"
    throw
} finally {
    Detener-App
    Remove-Item -Recurse -Force $temporal -ErrorAction SilentlyContinue
}
