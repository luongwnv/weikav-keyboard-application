param([switch]$SkipBuild, [switch]$Installer)
$ErrorActionPreference = 'Stop'
Push-Location (Join-Path $PSScriptRoot '..')
try {
    $version = (Select-String -Path Cargo.toml -Pattern '^version = "([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value
    if (-not $SkipBuild) {
        cargo build --release --locked
        if ($LASTEXITCODE -ne 0) { throw 'Rust build failed' }
    }
    $package = Join-Path (Get-Location) 'dist/NUT-Studio-Windows'
    New-Item -ItemType Directory -Force -Path "$package/assets", 'dist/release' | Out-Null
    Copy-Item 'target/release/nut-studio.exe' "$package/NUT Studio.exe"
    Copy-Item 'README.md', 'LICENSE' -Destination $package
    Copy-Item 'assets/SOURCES.md', 'assets/Inter-LICENSE.txt', 'assets/fonts/NotoSansCJK-LICENSE.txt' -Destination "$package/assets"
    Compress-Archive -Path "$package/*" -DestinationPath "dist/release/NUT-Studio-$version-Windows-x64.zip" -Force
    if ($Installer) {
        $compiler = Get-Command ISCC.exe -ErrorAction SilentlyContinue
        $iscc = if ($compiler) { $compiler.Source } else { "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe" }
        if (-not (Test-Path $iscc)) { throw 'Install Inno Setup 6 to build the Windows installer.' }
        $outputDir = Join-Path (Get-Location) 'dist/release'
        & $iscc "/DAppVersion=$version" "/DSourceDir=$package" "/DOutputDir=$outputDir" 'scripts/windows-installer.iss'
        if ($LASTEXITCODE -ne 0) { throw 'Windows installer build failed' }
    }
    Write-Output 'Built release packages in dist/release'
} finally { Pop-Location }
