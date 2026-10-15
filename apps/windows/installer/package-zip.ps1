<# 使用已构建产物生成魔多Windows开发版ZIP，不需要额外安装打包软件。 #>
[CmdletBinding()]
param()
$ErrorActionPreference='Stop'
$modoRepo=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..\..')).Path
$modoStage=Join-Path $modoRepo 'target\installer\Modo-0.1.0-dev-windows'
$modoApp=Join-Path $modoStage 'app'
New-Item -ItemType Directory -Path $modoApp -Force | Out-Null
foreach($modoFile in @('modo-server.exe','modo-settings.exe','modo_tsf.dll')){Copy-Item -LiteralPath (Join-Path $modoRepo "target\release\$modoFile") -Destination $modoApp -Force}
Get-ChildItem -LiteralPath (Join-Path $modoRepo 'target\installer\settings-runtime') -Force | ForEach-Object {Copy-Item -LiteralPath $_.FullName -Destination $modoApp -Recurse -Force}
Copy-Item -LiteralPath (Join-Path $modoRepo 'target\i686-pc-windows-msvc\release\modo_tsf.dll') -Destination (Join-Path $modoApp 'modo_tsf-x86.dll') -Force
New-Item -ItemType Directory -Path (Join-Path $modoApp 'data\generated') -Force | Out-Null
Get-ChildItem -LiteralPath (Join-Path $modoRepo 'data\generated') | ForEach-Object {Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $modoApp 'data\generated') -Recurse -Force}
foreach($modoAsset in @('emoji','levels','wubi','sample','stroke')){New-Item -ItemType Directory -Path (Join-Path $modoApp 'assets') -Force | Out-Null;Copy-Item -LiteralPath (Join-Path $modoRepo "assets\$modoAsset") -Destination (Join-Path $modoApp 'assets') -Recurse -Force}
Copy-Item -LiteralPath (Join-Path $modoRepo 'apps\windows\tsf\resources\modo.ico') -Destination $modoApp -Force
foreach($modoFile in @('LICENSE','THIRD_PARTY_NOTICES.md','README.md')){Copy-Item -LiteralPath (Join-Path $modoRepo $modoFile) -Destination $modoStage -Force;Copy-Item -LiteralPath (Join-Path $modoRepo $modoFile) -Destination $modoApp -Force}
foreach($modoFile in @('Install.ps1','Uninstall.ps1','Install.cmd','Uninstall.cmd','Run-Installer.ps1')){Copy-Item -LiteralPath (Join-Path $PSScriptRoot $modoFile) -Destination $modoStage -Force}
foreach($modoDest in @($modoStage,$modoApp)){Copy-Item -LiteralPath (Join-Path $modoRepo 'licenses') -Destination $modoDest -Recurse -Force}
$modoOut=Join-Path $modoRepo 'dist\Modo-0.1.0-dev-Windows.zip'
New-Item -ItemType Directory -Path ([IO.Path]::GetDirectoryName($modoOut)) -Force | Out-Null
Compress-Archive -LiteralPath $modoStage -DestinationPath $modoOut -Force
$modoHash=(Get-FileHash -LiteralPath $modoOut -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText(($modoOut+'.sha256'),($modoHash+'  '+[IO.Path]::GetFileName($modoOut)+[Environment]::NewLine),[Text.UTF8Encoding]::new($false))
Write-Output ('SHA256：'+$modoHash)
Write-Output $modoOut
