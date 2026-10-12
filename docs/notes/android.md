# 魔多 Android 工程记录

`apps/android/native` 是 `qingjian-android-native` / `libqingjian_android.so`，只装配共用 Engine、Keypad 与 MembershipClient，经JNI传递输入、候选和账号请求。Java包名为 `im.modolanguage.android`，版本独立为 `0.1.0-dev`。源文件路径保留上游目录名以便对照。

`ModoImeService` 管理 InputConnection，`KeyboardView` 自绘候选和26/9键，`ModoSettingsActivity` 为原生账号和学习设置。JNI工作在单线程执行器，输入不等待网络。九键索引、解码、排序、组句与候选消耗都在 Core `engine/keypad/`；英文仍用26键。视图创建与重新显示都重新应用布局，避免 Android 先调用 `onStartInput` 后创建视图时漏掉九键设置。

焦点切换递增代次，重置Engine，旧结果不会上屏到新输入框。密码和 `IME_FLAG_NO_PERSONALIZED_LEARNING` 将Engine设为private，关闭云功能；密码直接输入字符。

## 数据、账号和语音

Gradle `stageProductData` 从锁定产品数据包复制 `dict.qj`、`lm.qj`、学习语言释义表与英文词表，首次运行原子复制到应用私有目录，由Rust mmap加载。不带本地神经重排权重；组词复用bigram，用户学习数据在 `files/user`，没有输入日志。

整句翻译复用CloudSentenceTranslator的防抖、缓存和过期隔离，默认800ms防抖，会员模式经MembershipClient请求魔多服务。HTTPS使用Rustls/ring和Mozilla根证书，保留证书链与主机名验证；仅本机/模拟器允许HTTP。客户端不保存供应商密钥，用户会话令牌由Android Keystore AES-GCM加密。

VoiceRecorder采集16kHz单声道16位PCM，内存生成WAV，最长30秒；VoiceTeacher在后台提交，取消和失焦作废结果。设置页先申请麦克风；输入法长按空格录音。跟读保留纠正文本并再次录音，没有发音评分。

## 构建和验证

需要JDK17或更新版本、Android SDK platform36、NDK28.2.13676358、Gradle8.13或更新的8.x（已使用8.14.3）、AGP8.13.0和Rust1.96。

```powershell
apps/android/scripts/build.ps1 -Sdk E:\Geek\AndroidSDK -Java E:\Geek\AndroidStudio\jbr -Gradle <gradle.bat>
node scripts/smoke-server.mjs
adb reverse tcp:8789 tcp:8789
adb install -r apps/android/app/build/outputs/apk/debug/app-debug.apk
adb shell am instrument -w -e endpoint http://127.0.0.1:8789/v1 im.modolanguage.android/im.modolanguage.android.ModoSmokeInstrumentation
adb shell am start -n im.modolanguage.android/.TypingActivity --es layout t9
adb shell ime enable im.modolanguage.android/.ModoImeService
adb shell ime set im.modolanguage.android/.ModoImeService
```

默认构建arm64-v8a、armeabi-v7a、x86_64，原生与APK均16KB页对齐。APK与SHA-256在 `dist/Modo-0.1.0-dev-SentenceAI-Android.apk`。Release用开发签名供侧载，发行需要独立发布证书。`-SkipNative`仅用于Rust没有变化时重打Java/资源。

`ModoSmokeInstrumentation`与真实编辑框TypingActivity只进Debug包。固定输出服务明确显示demo状态，用于账号、翻译和同步集成；真实供应商测试使用固定测试文本和合成语音，不采集用户输入。具体通过项和未验范围见[魔多验收记录](modo-verification.md)。APK重新安装或 `am start -S` 强制停止整个包后，系统可能切换到Gboard，需重新选择魔多再验证。
