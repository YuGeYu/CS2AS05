# Panel 融合调查记录与实施思路

状态：调查完成，尚未开始融合实现
调查日期：2026-07-25
当前项目基线：`YuGeYu/CS2AS05` `13b721839494fb1bdeef9bdd70e8b3d20a4b732d`（0.5.2）

## 1. 调查目标

确认能否把 `ed0ard/CS2-Bot-Improver` 的 Panel 融入 CS2 人机增强助手，在保留本助手安装、更新、卸载和诊断能力的同时，保留 Panel 除多语言切换之外的全部既有功能。这里的“只改 UI”定义为：

- 不改变 CS2 插件的功能语义、配置格式和运行结果。
- 不把 Local-Arena 的比赛、Demo、玩家饰品、独立更新系统等扩展功能带入本项目。
- 允许为融合界面增加必要的 Tauri command 和 Rust 文件操作适配层；否则 Vue 页面无法执行原 Panel 的本地操作。
- 最终只保留一套中文界面，不保留原 Panel 的语言选择页。

## 2. 调查对象与固定基线

| 对象 | 本次检查的提交 | 用途 |
| --- | --- | --- |
| [ed0ard/CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver) `v1.4.2` | [`97fd57d2`](https://github.com/ed0ard/CS2-Bot-Improver/tree/97fd57d2ee1e14e408ae3ca7b1b0cae596a792cc) | 本助手当前资源和原版 Panel 的功能基线 |
| `ed0ard/CS2-Bot-Improver` `main` | [`4b2ae161`](https://github.com/ed0ard/CS2-Bot-Improver/tree/4b2ae161b7236be5b32b08ff1e10a4a286513b63) | 检查上游当前公开源码结构 |
| [numakkiyu/Local-Arena](https://github.com/numakkiyu/Local-Arena) `main` | [`568031ee`](https://github.com/numakkiyu/Local-Arena/tree/568031eeefaf26f1e4f5fab83f9266a7ecd85e19) | 检查公开的兼容后端及其后续扩展 |
| `Local-Arena` 首次加入 Panel 后端的提交 | [`5dc04e9c`](https://github.com/numakkiyu/Local-Arena/commit/5dc04e9c0def2a2d4bb8c35d441607a5a943a954) | 确定后端代码来源和最小参考点 |

上述提交是调查快照，不表示将来实现时自动跟随各仓库 `main`。开始实现前应再次获取远端状态，但功能兼容仍以本项目内置的上游 `v1.4.2` 为准。

## 3. 已确认的事实

### 3.1 上游公开了 Panel 前端和调用契约

上游 `v1.4.2` 的 `Panel/` 包含 React/TypeScript 页面、状态管理、翻译、样式和 `Panel/src/lib/api.ts`。公开接口覆盖：

- 配置读取与保存。
- CS2 目录检测、选择和文件校验。
- 在线模式与 BOT 模式切换、Steam 启动参数协调和启动 CS2。
- Low、Medium、High 难度读取与切换。
- Aim 和 Nades 预设读取与写入。
- Bot skins、profiles、agents、music 开关。
- 队伍配置。
- 丢刀按键和刀具子类选择。
- 命令列表展示、搜索和复制。
- CS2 运行中修改后的“重启后生效”状态提示。

### 3.2 上游仓库没有提交原版 Panel 的 Rust 后端

本次分别检查了上游 `v1.4.2` 和 2026-07-25 的 `main` 树。两者都有 `Panel/src/lib/api.ts`，但没有 `Panel/src-tauri/`。因此不能从上游仓库直接取得原版 Panel EXE 对应的 Rust 实现。

这只说明原上游仓库没有公开该目录，不代表没有公开可参考的兼容实现。

### 3.3 Local-Arena 公开了独立实现的完整兼容后端

GitHub 元数据表明 Local-Arena 是 `ed0ard/CS2-Bot-Improver` 的公开 fork。其提交历史显示：

- `Panel/src-tauri/src/lib.rs` 在提交 `5dc04e9c` 中首次出现。
- 该文件在此提交的父版本中不存在。
- 提交作者为 Local-Arena 维护者，提交说明为 `feat: release CS2BotImproverPlus 1.4.2`。
- 该提交同时新增 `Cargo.toml`、Tauri 配置、`mode_files.rs` 和约 740 行初版 Rust 后端。
- Local-Arena 当前版本继续公开完整 Rust 后端，并已扩展到安装、诊断、比赛、Demo、饰品和在线更新等更大范围。

因此，准确结论是：原上游没有提交 Panel Rust 后端；Local-Arena 后来独立实现并以 AGPL-3.0 公开了一套兼容后端。我们可以在保留来源和许可证声明的前提下参考或移植其中与原 Panel 等价的部分。

### 3.4 本助手当前只有安装器级接口

当前项目的 `src-tauri/src/commands/cs2.rs` 只暴露目录发现、目录检查、进程检查、安装、打开原版 Panel、卸载和诊断接口。具体 Panel 功能仍通过 `open_upstream_panel` 启动独立的 `Panel v1.4.2.exe` 完成。

所以融合不是简单复制 CSS 或 React 页面。需要把 Panel 的功能契约接入本助手现有 Vue 3、Pinia、Tauri 2 架构，但不需要修改插件 DLL 的功能逻辑。

## 4. 调查结论

融合可行，而且不需要逆向 `Panel v1.4.2.exe`。

推荐以三类来源分别承担不同职责：

1. 上游 `v1.4.2` 的 Panel 前端和 README：定义必须保留的原版功能、选项和用户可见行为。
2. Local-Arena 的早期兼容后端及当前修复：作为文件操作、Steam 启动、模式切换和状态检测的公开实现参考。
3. 本项目现有 Vue/Tauri 代码：作为最终产品架构、安装安全检查、中文界面、更新渠道和发布流程。

不能直接把 Local-Arena 当前 `main` 整体合并进来。它已经成为独立产品，包含大量超出本次目标的功能和插件改动，整体合并会扩大行为面并破坏“只融合 Panel”的边界。

## 5. 目标功能清单

融合完成前，至少应逐项验证以下功能：

| 功能域 | 必须保留 | 目标行为 |
| --- | --- | --- |
| 本助手安装管理 | 是 | 扫描目录、安装/覆盖更新、卸载、诊断、软件更新保持可用 |
| 模式 | 是 | 在线模式、BOT 模式、`-insecure` 状态和启动 CS2 |
| 难度 | 是 | Low、Medium、High，识别磁盘真实状态 |
| Aim | 是 | `head`、`mixed`、`body` |
| Nades | 是 | `max`、`more`、`normal`、`off` |
| 队伍 | 是 | 保持上游 Panel 的双方人数/队伍配置能力 |
| Bot 物品 | 是 | skins、profiles、agents、music 独立开关 |
| 刀具 | 是 | 丢刀按键捕获、刀具子类多选及配置持久化 |
| 命令 | 是 | 完整列表、分类/折叠、搜索、复制及参数提示 |
| 状态反馈 | 是 | 目录、文件、CS2 进程、修改待重启、错误码和 Toast |
| 多语言选择 | 否 | 统一为简体中文，不提供语言设置页 |
| Local-Arena 扩展 | 否 | 不引入比赛、Demo、真人玩家饰品、贴纸或独立更新系统 |

## 6. 推荐实施步骤

### 阶段一：冻结基线和功能契约

1. 固定上游 `v1.4.2`、Local-Arena 参考提交和本项目提交号。
2. 将上游 `Panel/src/lib/api.ts` 的 DTO 和 command 整理成功能契约表。
3. 对原版 Panel 逐项记录输入、修改前文件、修改后文件、返回状态和 CS2 运行中行为。
4. 为真实 CS2 测试目录建立时间戳备份；普通开发优先使用模拟目录夹具。
5. 明确不可改变的资源摘要、NadeSystem 定制 DLL 和安装清理范围。

阶段产物应是测试和契约文档，不先做大规模 UI。

### 阶段二：建立独立的 Panel 后端模块

不要继续把所有逻辑堆入现有 `services/cs2.rs`。建议新增：

```text
src-tauri/src/
  commands/panel.rs
  models/panel.rs
  services/panel/
    mod.rs
    config.rs
    files.rs
    mode.rs
    presets.rs
    bot_items.rs
    knives.rs
    steam.rs
```

实现顺序：

1. 只读接口：目录、文件、模式、难度、预设、Bot 物品和刀具状态。
2. 可逆写入：Aim、Nades、Bot 物品、队伍和刀具配置。
3. 文件替换：难度和在线/BOT 模式切换，写入前备份并使用原子替换。
4. Steam 启动参数与启动 CS2。
5. 配置迁移、错误码、日志和诊断输出。

写接口必须限制在用户已选择并验证过的 `game/csgo` 中。所有路径使用结构化路径 API，配置文件尽量使用对应解析器或明确的行级语法，不使用无边界字符串替换。

### 阶段三：在 Vue 中迁移 Panel 状态模型

新增独立 Pinia store 和 Tauri service，例如：

```text
src/
  features/panel/
    types.ts
    state.ts
    api.ts
  stores/panel.ts
```

需要保留原 Panel 的关键状态语义：

- 磁盘真实状态优先于上次保存的配置。
- 操作时可以乐观更新，但后端失败必须回滚。
- CS2 运行中修改时显示“重启后生效”，并在进程退出后重新读取磁盘状态。
- 后台刷新失败不能清空最后一次有效状态，也不能重复弹窗。
- 目录变化后一次性刷新所有相关功能状态。

### 阶段四：融合 UI

建议将当前单页安装器改成紧凑的桌面工具结构：

- `概览`：CS2、插件、目录、模式、难度和主要启动操作。
- `人机预设`：Aim、Nades、队伍。
- `Bot 物品`：skins、profiles、agents、music。
- `刀具`：按键与刀具选择。
- `命令`：搜索、分类和复制。
- `安装与诊断`：保留当前安装、更新、卸载和诊断能力。
- `关于`：本项目、上游与 Local-Arena 参考来源和许可证。

界面继续使用 Vue 3、Pinia 和 `lucide-vue-next`，不在同一 WebView 内混装 React。统一简体中文，保留现有主题和自定义标题栏，并保证键盘焦点、禁用态、错误态和 44px 操作目标。

### 阶段五：差分验证

每个写操作都应执行同一套对照：

1. 从相同的干净目录快照开始。
2. 用原版 `Panel v1.4.2.exe` 执行一次操作并保存文件清单和 SHA256。
3. 恢复快照，用融合版执行相同操作。
4. 比较新增、删除、内容、Steam 启动参数和返回状态。
5. 分别在 CS2 未运行与运行中验证。
6. 启动真实 CS2，确认最终游戏状态，而不只检查中间文件。

差异必须能解释。不能仅因为界面显示相同就判定兼容。

### 阶段六：替换和回退

首个融合测试版本保留“打开原版 Panel”兼容入口，但放入诊断/高级区域，不作为主流程。满足以下条件后再删除：

- 功能清单全部通过自动测试和人工差分测试。
- 从干净安装、已有 0.5.2 安装和异常中断三种基线均能恢复。
- 实际 CS2 的在线模式与 BOT 模式往返测试通过。
- 发布安装包完成安装、升级、卸载和启动冒烟测试。

如果融合功能失败，回退应只切回原版 Panel 入口，不回退本助手现有安装资源或用户配置。

## 7. 测试与发布要求

最低验证集合：

```powershell
npm run verify
cargo test --manifest-path src-tauri/Cargo.toml
npm run bundle:desktop
```

另外应新增：

- 每个 Panel command 的 Rust 单元测试和临时目录集成测试。
- Vue store 的成功、失败、乐观更新回滚和待重启状态测试。
- 上游 Panel 功能清单契约测试，防止后续误删入口。
- 安装资源 ZIP 与定制 `NadeSystem.dll` 摘要测试。
- NSIS 安装、覆盖升级、卸载和持续启动测试。
- 真实 CS2 冒烟测试前的自动备份以及测试后的文件和启动参数恢复。

## 8. 来源与许可证处理

- 本项目继续使用 `AGPL-3.0-or-later`。
- 可以复制或改写 Local-Arena 后端代码，应在 `NOTICE.md` 和对应源码文件中明确记录仓库、提交号和用途。
- 不把 Local-Arena 描述为上游官方 Panel 后端；它是独立维护者公开实现的兼容后端。
- 保留 `ed0ard/CS2-Bot-Improver` 的原始来源、作者和许可证信息。程序里不提示Local-Arena，因为在他的项目里，明明引用了`ed0ard/CS2-Bot-Improver`，却删除了几个主要作者，令人痛心与唏嘘。我们只在我们的源码里提及Local-Arena即可，不在向用户的这一边提及。我们这样可以既提到Local-Arena又不提到Local-Arena。
- 发布安装程序时同步提供与该安装程序一致的完整源代码。

## 9. 调查复现命令

以下只读命令可用于重新确认远端提交：

```powershell
git ls-remote https://github.com/ed0ard/CS2-Bot-Improver.git HEAD refs/heads/main refs/tags/v1.4.2
git ls-remote https://github.com/numakkiyu/Local-Arena.git HEAD refs/heads/main
```

确认 Local-Arena 首次加入后端的方式：

```powershell
git clone --filter=blob:none https://github.com/numakkiyu/Local-Arena.git
Set-Location Local-Arena
git log --diff-filter=A --format="%H`t%ad`t%an`t%s" --date=iso -- Panel/src-tauri
git show --stat --summary 5dc04e9c0def2a2d4bb8c35d441607a5a943a954
git show 5dc04e9c0def2a2d4bb8c35d441607a5a943a954^:Panel/src-tauri/src/lib.rs
```

最后一条命令应报告该路径在父提交中不存在，这证明该后端是在 Local-Arena 分叉后新增的。
