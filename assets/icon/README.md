# 图标

- `modo.svg` / `modo.png`：魔多独立制作的紫色M标记，`node tools/modo-icon.mjs`生成。
- `qingjian-mark.svg` / `logo.png`（128×128）：兼容上游脚本路径，内容已替换为魔多标记。`apps/macos/scripts/bundle.sh` 打包时用 `sips` + `iconutil`
  生成 `Qingjian.icns`，生成物不进仓库。
- `menu.svg`：黑色M模板标记，系统只取alpha。`menu.pdf`是对应的
  22×16pt 矢量版，打包时拷成 `qingjian-menu.pdf`，Info.plist 的图标键都指向它。为什么是这个形式和尺寸见
  `docs/design/architecture.md`「Info.plist 约定」。改了 svg 重新导出：

  ```sh
  rsvg-convert -f pdf --page-width 22pt --page-height 16pt -w 22pt -h 16pt assets/icon/menu.svg -o assets/icon/menu.pdf
  ```
- `windows/mode-zh.svg` / `mode-en.svg` / `mode-caps.svg`：Windows 任务栏的中 / 英 / A 图标源文件（16×16 画布，单色）。
  `windows/render-mode-icons.sh` 用 rsvg-convert + magick 栅格化成 16 / 20 / 24 / 32 四档的 8 位 alpha 蒙版，
  写到 `apps/windows/tsf/resources/mode/`，DLL 用 `include_bytes!` 嵌入、运行时按任务栏深浅色填色（`com/mode/icon.rs`）。
  改了 svg 重跑脚本，生成物随仓库提交。
