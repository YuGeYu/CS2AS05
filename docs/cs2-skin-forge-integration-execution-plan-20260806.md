# CS2-Skin-Forge 完整移植交接方案

状态：方案制定完成，交给下一个“实际执行 AI”实施。本文不是完成报告；在真实 CS2/CounterStrikeSharp 验收前不得宣称功能已移植或可用。

## 1. 目标与范围

在 `E:\CS2AS05` 的现有 Tauri 2 + Vue 3 + TypeScript + Pinia + Rust 桌面应用中增加一个主导航项“皮肤工坊”（建议 key：`skinForge`），完整移植上游 [emptysuns/CS2-Skin-Forge](https://github.com/emptysuns/CS2-Skin-Forge) 的面板和游戏插件能力：

- 武器涂装、磨损、种子；每个武器最多 5 个贴纸（位置/磨损/缩放/旋转）；挂件（位置/种子）；命名标签；StatTrak 与击杀数。
- CT/T 独立刀型、刀皮、磨损、种子；CT/T 独立手套类型/涂装/磨损/种子；CT/T 角色模型；音乐盒。
- 精确自定义与随机模式；保存、读取、旧 JSON 兼容迁移。
- 设置 CS2 根目录、检测 CounterStrikeSharp、部署/更新 PlayerSkinMod 插件资源、状态诊断。
- 游戏内命令 `skin_menu`、`skin_random`、`skin_reset` 和文件变更自动重载。

不在本次范围：在线库存/Steam 交易、VAC 绕过、受保护服务器支持、云同步、把上游 React/Tailwind 应用嵌套为第二个应用、无证据的“仅客户端/无风险”承诺。

## 2. 已核对的上游基线

- 仓库：`https://github.com/emptysuns/CS2-Skin-Forge`
- 固定 commit：`b2edea17db9128609dd41f726f179cd965206433`（`main`，2026-08-06 调查）。执行前如需升级，必须重新记录 commit、文件清单和数据差异。
- 上游许可证文件标为 GPL-3.0；仓库 README 同时含 AGPL/GPL 表述。执行前核对 `LICENSE`、第三方 CounterStrikeSharp 许可，并在本项目 `NOTICE.md` 和关于/来源页追加来源、commit、许可和风险文本；不要把上游 DLL 当作本项目原创。
- 上游是双端系统：`Panel/` React/Tauri 面板只负责编辑与写文件；`addons/counterstrikesharp/plugins/PlayerSkinMod/` C# 插件在 CS2 进程内读取 JSON、监听变化、钩取游戏事件和内存函数。

上游关键文件：

```text
Panel/src/App.tsx                         页面状态、旧配置迁移、保存/部署状态
Panel/src/lib/api.ts                      Tauri invoke 契约
Panel/src/utils/types.ts                  Loadout/贴纸/挂件/StatTrak 类型
Panel/src/data/{weapons,skins,knives,knifeSkins,stickers,keychains}.ts
Panel/src/data/{agents,music...}           角色/音乐/多语言静态数据（按实际文件名复制）
Panel/src/components/{Weapon,Knife,Glove,Agent,MusicKit}Panel.tsx
Panel/src/components/editors/*            武器/刀/手套编辑弹窗与磨损种子控件
Panel/src/i18n/*                           6 种语言字典
Panel/src-tauri/src/lib.rs                配置、JSON、插件检查、部署、更新命令
addons/.../PlayerSkinModPlugin.cs         事件注册、GiveNamedItem 钩子、命令、重载
addons/.../Services/WeaponService.cs      涂装/贴纸/挂件/刀/手套属性应用
addons/.../Services/LoadoutService.cs     容错 JSON 解析与旧字段镜像
addons/.../Models/PlayerLoadout.cs        插件端模型
addons/.../Data/StaticData.cs              defindex、模型、音乐/刀/手套静态表
addons/.../skins_en.json                   legacy paint 标记和皮肤表
```

## 3. 现有项目边界与集成原则

现有入口是 [`src/components/AppShell.vue`](../src/components/AppShell.vue)，已有 `overview/presets/items/knives/commands/demoReview/install`，现有 CS2 根目录选择与安装状态由 `src/stores/cs2.ts`、`src-tauri/src/services/cs2.rs` 管理，现有 Bot 物品/刀具页面不等于上游皮肤编辑器。

执行原则：

1. 复用现有 `cs2.selectedRoot`、`cs2.cs2Running`、Tauri dialog、toast、Lucide 图标、主题变量和 CSS 工具；不要新增第二套 CS2 路径配置。
2. 把上游插件源码/静态数据放在独立 vendor 边界（建议 `third_party/CS2-Skin-Forge/`），本项目适配层放在 `src-tauri/src/skin_forge/`，不直接改第三方源文件来塞业务逻辑。
3. 不嵌套 React；把上游功能按 Vue 组件重写/移植，数据常量可按许可逐文件复制并保留 `UPSTREAM.md`、commit、SHA-256。
4. 不让 Rust 直接执行任意路径拼接或覆盖文件：路径必须由已选 CS2 根目录解析、canonicalize/边界校验，并使用临时文件 + 原子 rename 写 `player_loadout.json`。
5. CS2 运行中禁止部署/覆盖插件 DLL 和 JSON；沿用项目已有进程状态 gate，失败时显示可操作原因。

## 4. 目标目录与文件变更

### 4.1 前端

```text
src/views/SkinForgeView.vue                 页面壳、tabs、加载/空/错误/未部署状态
src/components/skin-forge/
  SkinForgeHeader.vue                       应用装备、随机、重置、部署状态
  WeaponForgePanel.vue
  KnifeForgePanel.vue
  GloveForgePanel.vue
  AgentForgePanel.vue
  MusicKitForgePanel.vue
  WeaponEditorModal.vue / KnifeEditorModal.vue / GloveEditorModal.vue
  WearSeedControls.vue / TeamToggle.vue / PickerGrid.vue
src/data/skin-forge/                         上游静态表的 Vue/TS 版本
src/types/skin-forge.ts                      Loadout 与 API 类型
src/stores/skinForge.ts                      配置、loadout、dirty/busy、插件状态
src/services/tauri/skinForge.ts              invoke 薄封装，不在组件直接 invoke
src/features/skin-forge/loadout-migration.ts 旧 shared 字段 -> CT/T 字段迁移
tests/skin-forge-*.spec.ts                    契约、迁移、导航和组件行为测试
```

导航在 `AppShell.vue` 的 `ViewKey`、`nav`、动态 `view` map 中加入 `{ key: 'skinForge', label: '皮肤工坊', icon: Palette }`；位置建议在“刀具”之后、“命令”之前，避免改变现有 Demo/安装流程。按钮必须有 title/aria-label/aria-current，窄窗口不得溢出。

### 4.2 Rust/Tauri

```text
src-tauri/src/skin_forge/mod.rs
src-tauri/src/skin_forge/config.rs       应用配置与现有 selectedRoot 的适配
src-tauri/src/skin_forge/loadout.rs      schema 校验、原子写、slot=0 读写、迁移
src-tauri/src/skin_forge/deploy.rs       插件文件检查、资源部署、版本/hash
src-tauri/src/commands/skin_forge.rs     #[tauri::command] 公开命令
src-tauri/resources/skin-forge/           PlayerSkinMod.dll/json/skins_en.json（或打包 zip）
third_party/CS2-Skin-Forge/               上游源码快照、UPSTREAM.md、许可证
```

在 `src-tauri/src/commands/mod.rs`、`services/mod.rs`、`src-tauri/src/lib.rs::generate_handler![]` 注册命令；在 `tauri.conf*.json` 的 resources 中加入 `resources/skin-forge`。不要给前端开放任意 shell/HTTP/file capability。

推荐命令契约（camelCase 返回字段）：

```text
skin_forge_get_config() -> { language: string|null, cs2Path: string|null }
skin_forge_save_config({ cs2Path, language }) -> void
skin_forge_load_loadout({ slot: 0 }) -> Loadout|null
skin_forge_save_loadout({ slot: 0, loadout }) -> void
skin_forge_detect_path() -> string|null
skin_forge_check_plugin() -> PluginCheckResult
skin_forge_deploy_plugin() -> DeployResult
skin_forge_reset_loadout({ slot: 0 }) -> void
```

`Loadout` 可由 Rust 以 `serde_json::Value` 持久化以保持字段前向兼容，但边界必须检查对象、数值范围（wear 0..1、seed 非负、贴纸最多 5、路径无 `..` 越界），拒绝超大 JSON。`PluginCheckResult` 至少包含 `allPresent/missingFiles/versionMismatch/deployedVersion/panelVersion/counterstrikesharpInstalled`。

### 4.3 游戏插件与资源

把上游以下文件复制到独立资源/源码目录并按当前 CounterStrikeSharp API 编译：

```text
PlayerSkinModPlugin.cs
Services/WeaponService.cs
Services/LoadoutService.cs
Models/PlayerLoadout.cs
Data/StaticData.cs
PlayerSkinMod.csproj / PlayerSkinMod.json / skins_en.json
```

插件运行逻辑必须保留：`OnPlayerSpawn` 应用 CT/T agent、刀、手套、音乐盒；`GiveNamedItemFunc` 与 `OnEntitySpawned` 覆盖武器发放；动态属性写入、`SetStateChanged`、legacy bodygroup；`FileSystemWatcher` 200ms debounce + `Server.NextFrame` 重载；`skin_menu/skin_random/skin_reset` 命令。`PlayerLoadout` 的 legacy shared 字段和 CT/T 字段必须与前端 JSON 名称一致，旧文件逐 slot 容错解析。

插件 DLL 不能假定在构建机存在。执行 AI 需记录 CounterStrikeSharp SDK 版本、编译命令和 DLL SHA-256；构建失败时不得生成“占位 DLL”并声称可用。资源部署采用临时目录写入、hash 校验后替换，保留现有插件备份；CS2 运行中返回明确阻止。

## 5. 前端行为与 UI 约束

- `SkinForgeView` 首屏是紧凑工作台，不做营销 hero、不做卡片套卡片。页头显示当前 CS2 路径、插件状态、`应用装备` 主按钮和 `随机模式/自定义模式` segmented control。
- tabs：武器、刀具、手套、角色、音乐盒；重复条目使用稳定网格/表格，编辑使用 dialog。熟悉命令使用 Lucide 图标按钮并配 tooltip，文字按钮仅用于“应用/部署/重置”等明确动作。
- CT/T 使用 segmented control；武器编辑器提供涂装搜索/筛选、wear slider+number、seed stepper、贴纸 0..5、挂件、命名标签、StatTrak。手套涂装列表必须按所选 glove defindex 过滤，切换类型重置为合法默认值。
- 所有 invoke 都有 loading/disabled、错误 toast、空数据 `--`/unavailable、保存成功反馈；CS2 正在运行时部署按钮禁用。
- 复用 `src/styles` 现有 tokens，遵循 UI/UX Pro Max：可访问 focus、键盘 dialog、`prefers-reduced-motion`、移动/窄窗口不横向溢出；不使用浏览器原生文件夹 input，路径选择走现有 Tauri dialog。

## 6. 实施阶段（每阶段提交证据）

### A. 基线与 vendor

1. 记录上游 commit、`LICENSE`、插件/静态文件清单和每个资源 hash，写 `third_party/CS2-Skin-Forge/UPSTREAM.md`。
2. 核对当前项目 `src-tauri/src/services/cs2.rs` 的 CS2 根目录语义：`selectedRoot` 是 `.../game/csgo`，上游配置也应写该目录，不要多拼一层 `game/csgo`。
3. 确认 CounterStrikeSharp API/运行时版本与现有 `CS2BotImprover.zip` 是否共存；若插件 SDK 不兼容，停止并报告版本冲突。

### B. 数据契约与 Rust

1. 先写 `src/types/skin-forge.ts`、默认 Loadout、迁移函数和 Vitest；覆盖空/旧 shared/部分 CT-T/非法数值。
2. 实现 Rust config/loadout/deploy 模块与 command 契约；加入单元测试：路径边界、原子写、损坏 JSON、插件缺文件/hash/version、CS2 运行 gate。
3. 将资源加入 Tauri dev/build 两种布局测试，验证 `include_bytes!` 或 resource dir 在 debug 与 bundle 均可读取。

### C. Vue 工作台与导航

1. 先实现导航和 `SkinForgeView` 壳，再逐 tab 移植上游数据和编辑器；组件只通过 Pinia store 调服务。
2. 复用现有 `KnivesView` 的图像/卡片样式时，确认其数据字段与上游 defindex/paintkit 不同则建立显式映射，禁止静默按数组 index 套用。
3. 完成设置/部署状态、旧文件迁移、保存后 reload/check 状态闭环。

### D. 插件编译与部署

1. 在 Windows 真实安装的 CounterStrikeSharp SDK 下构建 DLL；运行 C# 编译/打包命令并保存日志、版本、hash。
2. `skin_forge_deploy_plugin` 先检测 CS2 未运行、目标目录在 selectedRoot 下，再备份并写入 `PlayerSkinMod.dll/json/skins_en.json`；不自动下载未固定版本的 CounterStrikeSharp。若确需自动安装，另列授权、固定 release、SHA-256、网络失败回滚任务。
3. 与现有 Bot Improver 共存安装测试：只新增/更新 `PlayerSkinMod`，不删除或覆盖 Bot 配置。

### E. 真实游戏验收

执行 AI 只能在用户允许的本机离线/本地服务器环境验收，并在结果中区分“自动化通过”和“真实游戏通过”：

1. 备份并确认 Steam 小号/`-insecure`；启动 CS2，确认进程状态 gate 正确阻止部署。
2. 部署插件，CounterStrikeSharp 日志显示 `PlayerSkinMod loaded successfully`，无加载异常。
3. 面板选择一把武器涂装 + wear/seed + 贴纸/挂件/命名/StatTrak，应用后 JSON 与插件日志字段一致；重生后肉眼确认第一/第三人称。
4. 分别验证 CT/T 刀、手套、角色、音乐盒；切换类型后涂装不发生 UV 错位；执行 `skin_random`、`skin_reset`、编辑器保存触发自动重载。
5. 记录 Demo/截图/日志路径、CS2 版本、插件 DLL hash；任一核心功能只在“真实可见 + 日志无错误 + JSON 可回读”三证据齐全后标记完成。

## 7. 自动化验收矩阵

```text
npm run typecheck
npm run lint
npm test -- --run
npm run build:web
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo test --manifest-path .\src-tauri\Cargo.toml --lib
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings
npm run workspace:check
```

新增测试至少包括：

- `tests/skin-forge-navigation.spec.ts`：导航项可见、点击只切换 `data-current-view=skinForge`、窄窗口无溢出。
- `tests/skin-forge-loadout.spec.ts`：默认值、旧 shared 字段迁移、CT/T 独立保存、贴纸上限和非法输入拒绝。
- `tests/skin-forge-ipc.spec.ts`：mockIPC 覆盖 config/load/save/check/deploy/reset、错误和 CS2-running 禁用。
- Rust：路径/JSON/部署 hash/version/原子写测试；不测试真实内存钩子替代游戏验收。

## 8. 完成定义、回滚与停止条件

完成定义：导航可达；五个 tab 的所有上游能力可编辑并写出契约 JSON；插件 DLL 由固定 SDK 编译且 hash 已记录；部署/检查/重载闭环可用；自动化命令全通过；至少一轮真实离线游戏完成武器、刀、手套、角色、音乐盒、随机/重置验证；许可证/`-insecure`/VAC 风险在 UI 与文档明确显示。

回滚：部署前备份 `PlayerSkinMod` 目录；新 DLL/JSON 校验失败或 CS2 加载异常时恢复备份，不碰 `addons/counterstrikesharp/configs` 和 Bot Improver 文件。前端若失败只移除新导航/模块文件，不回退用户已有改动。

必须停止并交回用户/项目负责人：CounterStrikeSharp API 与现有版本无法共存；无法获得可运行的固定版本 DLL；路径语义无法确认；真实游戏需要 VAC 保护服务器；或任何测试只能“看起来成功”但无法提供 JSON、日志、截图/游戏内结果证据。禁止以构建成功、占位 DLL、静态页面或未运行的插件替代完成声明。

## 9. 交接给执行 AI 的首条命令

```powershell
Set-Location E:\CS2AS05
git status --short
Get-Content .\docs\cs2-skin-forge-integration-execution-plan-20260806.md
git ls-remote https://github.com/emptysuns/CS2-Skin-Forge.git HEAD
```

执行 AI 应先回读本方案和当前工作树，再按 A->B->C->D->E 顺序实施；每阶段结束写对应 execution-report，保留命令输出和证据路径，不要把本方案改写成完成报告。
