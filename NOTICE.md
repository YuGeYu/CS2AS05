# NOTICE

CS2 人机增强助手 `0.5.6` 同包并行包含 BotVision `0.2.2`（[XBribo/CS2-Bot-Vision](https://github.com/XBribo/CS2-Bot-Vision/releases/tag/v0.2.2)，固定提交 `17a2ca707ec1ffea8797e56d94915596ec5ef4c4`）的三个 Windows MetaMod 组件：`addons/BotVision/gamedata.json`、`addons/BotVision/bin/win64/BotVision.dll`、`addons/metamod/BotVision.vdf`。源 ZIP `E:\dow\BotVision-Windows-0.2.2.zip` SHA-256 为 `40B596D34BF336D9E59E663DAC2F94BD7C61D951C56E421EF66B5190B8787290`。上游 `v0.2.2` README/LICENSE 已核实为 AGPL-3.0；其闭源再分发、托管服务和不满足 copyleft 的专有集成需完整履行相应许可证或另行取得商业许可。该 DLL 当前核查结果为 Authenticode `NotSigned`；BotVision 是原生 MetaMod 并行组件，不替换 CounterStrikeSharp 或 NadeSystem。

CS2 人机增强助手 `0.5.6`（[YuGeYu/CS2AS05](https://github.com/YuGeYu/CS2AS05)）是独立下游项目，并非上游官方发行版。它内置并再分发基于 [ed0ard/CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver) 的最小定制资源包。

上游基线为 tag `v1.4.3`、提交 `d1d83982db88fbdb686b2bf13aa8c6f9d65a4604`。相对官方 ZIP 仅修改两个 BOT cfg、替换 `addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll`，并新增助手自用的 `CS2AS05.plugin.json` 完整性标记；对应修改源码、策略测试和基线说明位于 `third_party/CS2-Bot-Improver-v1.4.3/nades-pacing/`。该定制不生成 `[NadeAudit]` 或 `[NadeLimit]` 控制台行。

上游项目及本助手均遵循 GNU Affero General Public License v3.0 或更高版本。发布本助手时应保留内置资源包已有的 LICENSE、README、版权和归属信息，并同时提供与安装程序对应的源代码。

刀具页的 20 张 PNG 历史素材仍来自固定官方 `Panel v1.4.2.exe` 的 Tauri 嵌入资源；这与 `0.5.4` 运行时内置并校验的 `Panel v1.4.3.exe` 是两个独立来源记录。图片按运行资源路径中的 subclass 数字映射并通过 Brotli 无损解压，逐图路径、尺寸和 SHA256 位于 `src/assets/knives/manifest.json`，可复现提取工具为 `scripts/extract-panel-knives.mjs`。

0.5.4 原生融合的 `commands.txt`、队伍解析、Source 按键映射和交互语义以固定 `ed0ard/CS2-Bot-Improver v1.4.3` 运行资产为基线，并由本项目下游加入 `bot_nades less` 与 `br_reroll`。Rust 兼容实现按本项目目录模型、原子写入和白名单约束独立维护；它不代表上游官方 Panel 后端。

开屏鸣谢与原创彩蛋“青冥试剑”使用 [Three.js](https://github.com/mrdoob/three.js) `0.185.1`（MIT）进行 WebGL 渲染。场景、几何体和交互逻辑均在本项目中原创并程序化生成，没有复用第三方游戏源码或视觉资产。
## LaihoE/demoparser

The Demo review feature includes the Rust parser core from
https://github.com/LaihoE/demoparser at commit
`ba39cc44cd5abfd7f34df2b3c0a7dd3630048311`, distributed under the MIT License.
The preserved license and provenance are available in `third_party/demoparser/`.

## 简易 Rating

The final `simple-rating-v1` formula is an independent project-local KDA-dominant multi-factor simplification, not an OpenRating or HLTV rating. It weights `(K + A / 5) / max(D, 1)` at 80%, damage at 10%, the report-level survival proxy at 5%, and assists per round at 5%.

## akiver/cs-demo-manager map resources

The Demo workbench includes CS2 radar and thumbnail resources and mechanically
converted map coordinate metadata from
https://github.com/akiver/cs-demo-manager at commit
`8961f5072fe4d42803dde68e8e71b3c90b216504`, copyright (c) 2014-present AkiVer,
distributed under the MIT License. The preserved license, provenance, and
per-file asset manifest are available in `third_party/cs-demo-manager/`.

No Electron, PostgreSQL, analyzer sidecar, or account integration from the
upstream application is included.

## kaecho/CS2-Skin-Forge

The Skin Forge workbench includes mechanically converted catalog data and a
fixed PlayerSkinMod build from the public migration repository
https://github.com/kaecho/CS2-Skin-Forge at tag `v1.8.2`, commit
`75f52fbd5fd0616dbbdd09a65c3a1981593400d1`. The original `emptysuns` URL was
not anonymously accessible during verification. Upstream READMEs declare
GPL-3.0, but this commit contains no standalone LICENSE file; that condition is
preserved in `third_party/CS2-Skin-Forge/UPSTREAM.md`.

Upstream plugin metadata remains `1.8.1` despite the repository `v1.8.2` tag.
The bundled derivative applies the recorded two-line version patch and is
reported as a downstream `1.8.2` build. Source, patch, complete hashes, and
build evidence are retained under `third_party/CS2-Skin-Forge/`.

## CS2-insight-agent behavior reference

The Demo library's Play and FolderSearch interaction semantics, managed playback
lifecycle, and Windows Explorer argument-array behavior were studied from
[DrEAmSs59/CS2-insight-agent](https://github.com/DrEAmSs59/CS2-insight-agent) commit
`17d2a213ee8c32608feee3c63f0b6d05eef8f945`, licensed under PolyForm
Noncommercial 1.0.0. No source code, tests, styles, text, or assets from that
project are included; the implementation in this repository is independent
Rust, Tauri, and Vue code.
