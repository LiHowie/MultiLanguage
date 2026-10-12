<# 魔多开发版安装：复制到独立产品目录，注册32/64位TSF；不设为默认输入法。 #>
[CmdletBinding()]
param([string]$InstallDir = "$env:ProgramFiles\Modo")
$ErrorActionPreference='Stop'
$modoIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
$modoPrincipal=[Security.Principal.WindowsPrincipal]::new($modoIdentity)
if(-not $modoPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){throw '请以管理员身份运行 Install.ps1，TSF注册需要写入Windows COM注册表'}
$modoSource=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot 'app')).Path
$modoTarget=[IO.Path]::GetFullPath($InstallDir)
if([IO.Path]::GetFileName($modoTarget.TrimEnd('\')) -ne 'Modo'){throw '安装目录最后一级必须是Modo'}
if(-not (Test-Path -LiteralPath (Join-Path $modoSource 'modo-server.exe'))){throw '安装包不完整：缺少modo-server.exe'}
New-Item -ItemType Directory -Path $modoTarget -Force | Out-Null
Get-Process -Name 'modo-server','modo-settings' -ErrorAction SilentlyContinue | Stop-Process
Get-ChildItem -LiteralPath $modoSource -Force | ForEach-Object {Copy-Item -LiteralPath $_.FullName -Destination $modoTarget -Recurse -Force}
& "$env:SystemRoot\System32\regsvr32.exe" /s (Join-Path $modoTarget 'modo_tsf.dll')
if($LASTEXITCODE -ne 0){throw '64位TSF注册失败'}
& "$env:SystemRoot\SysWOW64\regsvr32.exe" /s (Join-Path $modoTarget 'modo_tsf-x86.dll')
if($LASTEXITCODE -ne 0){throw '32位TSF注册失败'}
$modoRun='HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
New-Item -Path $modoRun -Force | Out-Null
New-ItemProperty -Path $modoRun -Name 'Modo' -Value ('"'+(Join-Path $modoTarget 'modo-server.exe')+'"') -PropertyType String -Force | Out-Null
Write-Output '魔多已安装。关闭管理员终端，双击安装目录的modo-server.exe启动；下次登录会自动启动。请在Windows语言与键盘设置添加魔多，再用Win+Space选择。设置程序为modo-settings.exe。'
