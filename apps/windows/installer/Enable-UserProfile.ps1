<# 在发起安装的普通用户会话启用魔多，保留现有键盘和默认输入法。 #>
[CmdletBinding(SupportsShouldProcess=$true)]
param()
$ErrorActionPreference='Stop'
$modoProfile='0x0804:{CB973A65-1455-43A9-B85F-386AA11627A1}{D46E5490-4F39-4BDB-AA9A-A1AE496122DC}'
if(-not $PSCmdlet.ShouldProcess($modoProfile,'将魔多加入本用户的输入法列表')){return}
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class ModoInputProfile {
    [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
    [DllImport("input.dll", CharSet = CharSet.Unicode, ExactSpelling = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool InstallLayoutOrTip(string profiles, uint flags);
}
'@
if(-not [ModoInputProfile]::InstallLayoutOrTip($modoProfile,0)){throw '魔多已安装，但加入当前用户输入法列表失败，请在Windows键盘设置中添加魔多'}
# 新安装器改由启动文件夹管理自启，清除开发ZIP留下的同名本用户启动项。
Remove-ItemProperty -LiteralPath 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'Modo' -ErrorAction SilentlyContinue
