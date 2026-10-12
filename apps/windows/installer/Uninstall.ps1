<# 魔多开发版卸载：撤销TSF与本用户启动项，保留用户学习记录。 #>
[CmdletBinding()]
param([string]$InstallDir = "$env:ProgramFiles\Modo")
$ErrorActionPreference='Stop'
$modoTarget=[IO.Path]::GetFullPath($InstallDir)
if([IO.Path]::GetFileName($modoTarget.TrimEnd('\')) -ne 'Modo'){throw '卸载目录最后一级必须是Modo'}
Get-Process -Name 'modo-server','modo-settings' -ErrorAction SilentlyContinue | Stop-Process
if(Test-Path -LiteralPath (Join-Path $modoTarget 'modo_tsf.dll')){& "$env:SystemRoot\System32\regsvr32.exe" /s /u (Join-Path $modoTarget 'modo_tsf.dll')}
if(Test-Path -LiteralPath (Join-Path $modoTarget 'modo_tsf-x86.dll')){& "$env:SystemRoot\SysWOW64\regsvr32.exe" /s /u (Join-Path $modoTarget 'modo_tsf-x86.dll')}
Remove-ItemProperty -LiteralPath 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'Modo' -ErrorAction SilentlyContinue
Write-Output ('注册已撤销。重新登录后可删除安装目录：'+$modoTarget+'。学习数据仍保留在本用户Modo目录。')
