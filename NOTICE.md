# NOTICE

CS2 人机增强助手 `0.5.3`（[YuGeYu/CS2AS05](https://github.com/YuGeYu/CS2AS05)）是独立下游项目，并非上游官方发行版。它内置并再分发基于 [ed0ard/CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver) 的最小定制资源包。

上游基线为 tag `v1.4.2`、提交 `97fd57d2ee1e14e408ae3ca7b1b0cae596a792cc`。相对官方 ZIP 替换 `addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll`，并新增助手自用的 `CS2AS05.plugin.json` 完整性标记；对应修改源码、策略测试、补丁和基线说明位于 `third_party/CS2-Bot-Improver-v1.4.2/nades-per-bot-round-limit/`。该定制不生成 `[NadeAudit]` 控制台行。

上游项目及本助手均遵循 GNU Affero General Public License v3.0 或更高版本。发布本助手时应保留内置资源包已有的 LICENSE、README、版权和归属信息，并同时提供与安装程序对应的源代码。

刀具页的 20 张 PNG 来自固定官方 `Panel v1.4.2.exe` 的 Tauri 嵌入资源。图片按运行资源路径中的 subclass 数字映射并通过 Brotli 无损解压，未使用搜索缩略图或生成图；逐图路径、尺寸和 SHA256 位于 `src/assets/knives/manifest.json`，可复现提取工具为 `scripts/extract-panel-knives.mjs`。图片与上游 Panel 一并按 AGPL-3.0-or-later 再分发。

0.5.3 原生融合的 `commands.txt`、队伍解析、Source 按键映射和交互语义取自上述固定 tag。Rust 兼容实现参考了 [numakkiyu/Local-Arena](https://github.com/numakkiyu/Local-Arena) 提交 `568031eeefaf26f1e4f5fab83f9266a7ecd85e19` 的公开 Panel 后端，并针对本项目目录模型、原子写入和白名单约束重新实现；它不代表上游官方 Panel 后端。
