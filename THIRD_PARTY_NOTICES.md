# 魔多外语学习输入法：来源与许可

魔多基于 [青简 Qingjian](https://github.com/qingjian-team/qingjian) 的 `c08ae57` 提交及本地已开发的整句翻译、Android 适配代码修改。青简原作者和贡献者保有其代码版权；本仓库保留上游 Git 历史，客户端及本仓库新增程序按 **GPL-3.0-or-later** 发布。完整文本见 [LICENSE](LICENSE)。魔多不是青简官方发行版。

魔多改动包括独立品牌、应用标识与 Windows TSF GUID、安卓 26/9 键、账号同步、会员整句翻译及语音老师。魔多图标由本项目独立制作，不使用青简图形商标。

账号、语音纠错和同步服务的部分实现，经项目所有者授权，迁移自其独角 `unicornLanguage` 项目。此仓库内的副本按 GPL 发布；独角原项目继续独立开发，不导入魔多或青简 GPL 代码。

词库、释义、词汇等级、语言模型语料、emoji 与码表分别保留原许可，不能统一改成自有数据。详细来源与声明见 [数据清单](docs/design/landscape.md)、[词库](assets/lexicon/README.md)、[释义表](assets/glossary/README.md)、[等级表](assets/levels/README.md)。CC BY-SA 数据的加工版继续遵守相同方式共享要求，MIT、Apache、Unicode 等声明随数据保留。

Windows设置程序附带Microsoft.WindowsAppSDK.Runtime 2.4.0；这些未修改的微软运行时文件按其独立许可提供，不改标为魔多GPL源码。许可原文见安装包的 `licenses/Microsoft-WindowsAppSDK-Runtime-2.4.0.txt` 和[NuGet官方许可](https://www.nuget.org/packages/Microsoft.WindowsAppSDK.Runtime/2.4.0/License)。本项目保持微软与第三方声明，分发者需遵守该运行时的分发条款。

青简的数据清单标注部分规范字表、常用词表转录仓库未附许可证。开发版保留来源记录；公开商业发行前须完成这些数据的授权核验或替换，不能把仓库根 GPL 视为所有数据的商用保证。

发布安装包时，提供该版本完整对应源码、构建与安装脚本，并在下载处提供不额外收费的源码获取方式。不得通过魔多使用协议限制 GPL 授予的修改或再分发权利。
