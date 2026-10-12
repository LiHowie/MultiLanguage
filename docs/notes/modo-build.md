# 魔多开发、构建与验收

本地目录为 `E:\project\MutliLanguage`（沿用已有目录拼写），GitHub 仓库是 `LiHowie/MultiLanguage`。origin 指向魔多仓库，upstream 指向青简，Git 历史保留。

Node 24 或更新版本：

```powershell
npm test
npm run check
Copy-Item .env.example .env
# 仅服务端填写模型服务配置
npm start
# 明确开启 .env 中 MODO_TEST_MODE=1 后，本机运营人员可开通测试会员
npm run member:grant -- learner@example.com 30
```

Windows：

```powershell
$env:QINGJIAN_UIACCESS='0'
cargo test --workspace --exclude qingjian-macos --exclude qingjian-linux-server
cargo clippy --workspace --exclude qingjian-macos --exclude qingjian-linux-server --all-targets -- -D warnings
cargo fmt --all --check
apps\windows\installer\build.ps1 -Zip
```

保留内部 Cargo package 名便于同步上游，Windows 产物是 `modo-server.exe`、`modo_tsf.dll`、`modo-settings.exe`。数据和日志使用独立的 Modo 目录，TSF GUID、命名管道、安装器 AppId 与青简不同。开发ZIP在 `dist/Modo-0.1.0-dev-Windows.zip`，含GPL和来源声明、自包含WinUI运行时及32/64位TSF。

解压ZIP后，管理员PowerShell运行 `powershell -ExecutionPolicy Bypass -File .\Install.ps1`（仅该进程，不修改系统执行策略）。关闭管理员终端，从安装目录双击 `modo-server.exe`；下次登录自动启动。在Windows键盘设置添加魔多，Win+Space切换，`modo-settings.exe`打开设置。卸载时管理员运行 `Uninstall.ps1`，撤销注册和启动项，保留学习数据，重新登录后可手动删除安装目录。ZIP初次安装使用固定DLL名，升级前请先关闭使用魔多的应用；正式版本化升级流程仍使用Inno。

已有Inno Setup时，不带 `-Zip` 可生成EXE。上游release工作流在魔多仓库不自动执行；CI保留共用crate和平台编译检查，并增加Node会员服务测试。验收范围见[验证记录](modo-verification.md)。

Android：

```powershell
apps\android\scripts\build.ps1
# 开发验收使用 fixed-output 服务，绝不调用收费模型
node scripts/smoke-server.mjs
adb reverse tcp:8789 tcp:8789
adb install -r apps\android\app\build\outputs\apk\debug\app-debug.apk
adb shell am instrument -w -e endpoint http://127.0.0.1:8789/v1 im.modolanguage.android/im.modolanguage.android.ModoSmokeInstrumentation
```

SDK/JDK/Gradle路径可通过构建脚本参数指定。Debug 包含真实编辑框 `TypingActivity` 和测试 instrumentation，Release 不包含。正式签名需改为发布密钥。首次使用九键建立索引，性能应在目标手机继续观察。

产品数据获取沿用 `tools/release/data-fetch.sh` 和 `data.lock` 指向的青简数据包，校验 SHA256；数据原许可保持。源码及必要数据的获取地址必须与发出的二进制版本对应，不能只提供上游原版源码。
