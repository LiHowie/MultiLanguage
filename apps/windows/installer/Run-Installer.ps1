<# ZIP安装包入口：请求UAC授权，显示安装结果；-WhatIf仅检查入口，不执行安装或提权。 #>
[CmdletBinding(SupportsShouldProcess=$true)]
param([ValidateSet('Install','Uninstall')][string]$Action='Install')
$ErrorActionPreference='Stop'
$modoScript=Join-Path $PSScriptRoot ($Action+'.ps1')
try {
    if(-not (Test-Path -LiteralPath $modoScript -PathType Leaf)){throw ('安装包不完整，缺少：'+$modoScript)}
    if(-not $PSCmdlet.ShouldProcess($modoScript,'使用管理员权限运行魔多安装或卸载')){return}
    $modoIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
    $modoPrincipal=[Security.Principal.WindowsPrincipal]::new($modoIdentity)
    if(-not $modoPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){
        $modoPowerShell=Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
        $modoArgs=@('-NoLogo','-NoProfile','-ExecutionPolicy','Bypass','-File',('"'+$PSCommandPath+'"'),'-Action',$Action)
        $modoProcess=Start-Process -FilePath $modoPowerShell -ArgumentList $modoArgs -Verb RunAs -Wait -PassThru
        exit $modoProcess.ExitCode
    }
    & $modoScript
    if(-not $?){throw '操作未完成，请查看上方错误'}
    Write-Host ''
    Read-Host '操作完成，按回车关闭窗口' | Out-Null
    exit 0
} catch {
    if($_.Exception.NativeErrorCode -eq 1223){
        Write-Host '已取消管理员授权，安装或卸载未执行。' -ForegroundColor Yellow
    } else {
        Write-Host ('操作失败：'+$_.Exception.Message) -ForegroundColor Red
    }
    Read-Host '按回车关闭窗口' | Out-Null
    exit 1
}
