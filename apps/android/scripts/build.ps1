<# Android APK：交叉编译 Rust 引擎，随后由 Gradle 打包。工具路径可显式传入。 #>
[CmdletBinding()]
param(
    [string]$Sdk = $env:ANDROID_HOME,
    [string]$Java = $env:JAVA_HOME,
    [string]$Gradle = $env:QINGJIAN_GRADLE,
    [string[]]$Abis = @('arm64-v8a', 'armeabi-v7a', 'x86_64'),
    [switch]$SkipNative
)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$android = Join-Path $repo 'apps\android'
if (-not $Sdk -and (Test-Path 'E:\Geek\AndroidSDK')) { $Sdk = 'E:\Geek\AndroidSDK' }
if (-not $Java -and (Test-Path 'E:\Geek\AndroidStudio\jbr')) { $Java = 'E:\Geek\AndroidStudio\jbr' }
if (-not $Sdk -or -not $Java) { throw '请用 -Sdk / -Java 指定 Android SDK 与 JDK 17 或更新版本' }
$Sdk = (Resolve-Path -LiteralPath $Sdk).Path
$Java = (Resolve-Path -LiteralPath $Java).Path
$env:ANDROID_HOME = $Sdk
$env:ANDROID_SDK_ROOT = $Sdk
$env:JAVA_HOME = $Java
$env:PATH = "$Java\bin;$env:PATH"
if (-not $Gradle) {
    $command = Get-Command gradle.bat -ErrorAction SilentlyContinue
    if ($command) { $Gradle = $command.Source }
}
if (-not $Gradle -and (Test-Path 'E:\Geek\AndroidStudio\CDriveData\gradle\wrapper\dists\gradle-8.14.3-all')) {
    $Gradle = Get-ChildItem 'E:\Geek\AndroidStudio\CDriveData\gradle\wrapper\dists\gradle-8.14.3-all' -Filter gradle.bat -Recurse |
        Select-Object -First 1 -ExpandProperty FullName
}
if (-not $Gradle) { throw '请用 -Gradle 指定 Gradle 8.13 或更新的 8.x 版本' }
$ndk = Join-Path $Sdk 'ndk\28.2.13676358'
if (-not (Test-Path -LiteralPath $ndk)) { throw '需要 Android NDK 28.2.13676358，可在 SDK Manager 安装' }
$bin = Join-Path $ndk 'toolchains\llvm\prebuilt\windows-x86_64\bin'
$targets = @{
    'arm64-v8a' = @('aarch64-linux-android', 'aarch64-linux-android26')
    'armeabi-v7a' = @('armv7-linux-androideabi', 'armv7a-linux-androideabi26')
    'x86_64' = @('x86_64-linux-android', 'x86_64-linux-android26')
}
if (-not $SkipNative) {
    foreach ($abi in $Abis) {
        if (-not $targets.ContainsKey($abi)) { throw "不支持的 ABI：$abi" }
        $target, $clangTarget = $targets[$abi]
        rustup target add $target
        if ($LASTEXITCODE -ne 0) { throw 'Rust Android 目标安装失败' }
        $suffix = $target.Replace('-', '_')
        [Environment]::SetEnvironmentVariable("CC_$suffix", (Join-Path $bin "$clangTarget-clang.cmd"), 'Process')
        [Environment]::SetEnvironmentVariable("AR_$suffix", (Join-Path $bin 'llvm-ar.exe'), 'Process')
        $env:RUSTFLAGS = "-C linker=$bin\clang.exe -C link-arg=--target=$clangTarget -C link-arg=-Wl,-z,max-page-size=16384"
        Push-Location $repo
        try {
            cargo build --release --locked -p qingjian-android-native --target $target
            if ($LASTEXITCODE -ne 0) { throw "Rust $abi 构建失败" }
        } finally { Pop-Location }
        $dest = Join-Path $android "app\src\main\jniLibs\$abi"
        New-Item -ItemType Directory -Path $dest -Force | Out-Null
        Copy-Item -LiteralPath (Join-Path $repo "target\$target\release\libqingjian_android.so") -Destination $dest -Force
    }
}
Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue
[IO.File]::WriteAllText((Join-Path $android 'local.properties'), ('sdk.dir=' + $Sdk.Replace('\', '/') + [Environment]::NewLine), [Text.UTF8Encoding]::new($false))
Push-Location $android
try {
    & $Gradle --no-daemon --console=plain assembleRelease assembleDebug
    if ($LASTEXITCODE -ne 0) { throw 'Android APK 打包失败' }
} finally { Pop-Location }
$out = Join-Path $repo 'dist\Modo-0.1.0-dev-SentenceAI-Android.apk'
New-Item -ItemType Directory -Path (Join-Path $repo 'dist') -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $android 'app\build\outputs\apk\release\app-release.apk') -Destination $out -Force
$hash = (Get-FileHash -LiteralPath $out -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText(($out + '.sha256'), ($hash + '  ' + [IO.Path]::GetFileName($out) + [Environment]::NewLine), [Text.UTF8Encoding]::new($false))
Write-Output "APK：$out"
