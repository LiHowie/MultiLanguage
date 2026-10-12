# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 魔多分支约定（优先于下方上游说明）

本仓库是魔多外语学习输入法，origin 为 LiHowie/MultiLanguage，客户端与本仓库程序按 GPL-3.0-or-later 发布。青简保留为 upstream，不直接向它推送魔多改动。独角是另一个独立闭源项目，不将青简或魔多 GPL 实现反向迁入独角。

当前优先 Windows 和 Android；安卓26/9键解码和排序在 Core 的 engine/keypad；会员网络接口在 qingjian-predict 的 membership；平台层仅做录音、权限、UI和事件桥接。基础输入和本地译词免费，整句翻译和语音老师会员20元/30天，服务端核验，客户端只保存用户会话令牌。没有真实付款，MODO_TEST_MODE=1 下可由本机管理员测试开通。

学习说明见 docs/design/modo.md 和 docs/user/learning/modo-membership.md；构建见 docs/notes/modo-build.md。Node服务测试用 npm test，语法检查 npm run check。Windows本机可以使用MSVC构建，设QINGJIAN_UIACCESS=0；测试和clippy排除qingjian-macos，Android用apps/android/scripts/build.ps1。内部crate名保留qingjian以便同步上游，产品名、包名、GUID、用户目录和图标独立。上游发版说明不代表魔多已通过真机验收或可收费发行。
## 仓库现状

跨平台输入法，Core 平台无关，各平台只做壳。已发版 0.1.3（macOS 与 Windows 共用版本号，2026-09-17）；Linux 第一阶段提供 Rust Server + Fcitx5 默认候选面板（源码安装，0.1.4 起另有 Ubuntu 26.04 预编译包）。
阶段与已完成项见 `docs/plan/roadmap.md`，待办见 `docs/plan/todo.md`。

## 目录地图

一行一个，只说它是什么、入口在哪；实现要点（数据文件、常数、生成命令）在 `docs/notes/crate-notes.md`，改了实现要同步那里。

- `crates/qingjian-core`：引擎。`Engine` 是对外唯一门面，`Translator` / `Learner` 等 trait 在 `engine` 模块；拼音解析、纠错、候选、排序、整句、双拼、注音、英文模式都在这里。
- `crates/qingjian-dictionary`：词库（TSV 或 `.qj` mmap），按音节位置二分查询。
- `crates/qingjian-translate`：释义表 `Glossary`、词汇等级表 `LevelTable`。
- `crates/qingjian-learning`：用户侧落盘：词频 / 用户词 / 个人 n-gram / 敲错表（`FrequencyLearner`）、输入日志（`InputLog`）、输入统计（`UsageStats`）、词汇记录（`VocabularyBook`）。
- `crates/qingjian-predict`：云联想 `CloudPredictor`（OpenAI 兼容接口）与释义兜底 `CloudGlossFiller`；`PredictConfig` 是 `[predict]` 分节。
- `crates/qingjian-lm`：整句转换的 bigram 语言模型 `BigramModel`。
- `crates/qingjian-neural`：字级 Transformer 本地推理 `CharScorer`（candle），给整句前几条路径重打分。
- `crates/qingjian-format`：`.qj` 数据容器（mmap 读、零拷贝视图、写入器、哈希索引）。
- `crates/qingjian-platform`：平台层共用：`Config`（TOML 配置）、`extra_dictionaries`、Windows Server ↔ DLL 的 `protocol` 类型。
- `crates/qingjian-render`：自绘渲染器（spike 中，分支 renderer-spike）：候选窗一帧 + 主题 → 位图，各平台只贴图。见 `docs/design/rendering.md`。
- `crates/qingjian-update`：检查更新：读官网的 `releases.json`、验 ed25519 签名、按平台与渠道挑新版本，只提示不安装；发版侧的签名工具是 `tools/release-sign`。见 `docs/design/update.md`。
- `apps/cli`：Core 的验证工具：查询、逐键计时、输入日志回放、整句评测、常数扫描。排序 / 整句 / 纠错的改动先跑它再合。
- `apps/macos`：IMK 壳，按 `app / host / imk / candidates / menubar / preferences` 分目录；`scripts/bundle.sh --install` 装到本机，`--pkg` 出分发包。
- `apps/windows`：`server`（Server 进程：Engine + IPC + 自绘候选窗与状态条）+ `tsf`（TSF DLL）+ `settings`（WinUI 3）+ `installer`（Inno）。DLL 不能带 Engine 的依赖树，所以是两个 package。
- `apps/linux`：`server`（Engine + Unix socket）+ `fcitx5`（框架事件、默认面板）+ `scripts`（用户级安装卸载、`package.sh` 打预编译包）；Server 手动启动，见 `docs/notes/linux-fcitx5.md`。
- `apps/android`：`native`（Rust JNI 门面，复用 Engine / 整句翻译）+ `app`（Android 输入法服务、自绘触摸键盘与设置）+ `scripts/build.ps1`（NDK / Gradle 打 APK）；见 `docs/notes/android.md`。
- `tools/dict-convert`、`tools/gloss-gen`、`tools/corpus`：产品数据生成（词库 / 语言模型 / 释义表 / emoji / 英文词表），输出到 `data/generated/`（gitignore）。
- `assets/`：随包数据源与样例，各目录有 README 写来源与许可。雾凇拼音（GPL）已彻底移除，不要再引入。

`docs/` 分四类（索引在 `docs/README.md`）：`design/` 设计与决定、`plan/` 路线与待办、`notes/` 工程记录（性能、复盘、踩坑、crate 实现要点）、
`user/` 用户文档（官网构建时拉取渲染，约定见 `docs/user/README.md`，措辞面向用户、不出现实现词）。

## 常用命令

```bash
cargo build                                   # 整个 workspace
cargo test                                    # 全部测试；-p <crate> 单个，加测试名过滤
cargo clippy --all-targets -- -D warnings
cargo fmt --all
cargo run -p qingjian-cli -- <拼音>...          # Core 的主要验证方式；--replay / --eval-text 见 crate-notes
apps/macos/scripts/bundle.sh --install        # mac 壳装到 ~/Library/Input Methods/（IMK 不能 cargo run 验证）
```

Windows 本机只 `cargo check --target x86_64-pc-windows-gnu`，真编译与真机测试在 Windows 机器上做（部署方式见 `apps/windows/README.md`）；
端到端验证 mac 可用 `osascript` 往 TextEdit 发按键再读回文本。

## 约定

架构约束、代码组织、版本号、提交信息、文档同步、提交前检查与发版都在 `docs/contributing.md`，随本文件一起载入：

@docs/contributing.md

交流用中文。
