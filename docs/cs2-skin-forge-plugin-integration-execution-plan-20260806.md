# CS2-Skin-Forge 第二轮插件闭环执行方案

状态：交给下一位实际执行 AI。第一轮前端/安全 IPC 已落地，但本方案完成前不得声称“皮肤工坊可在 CS2 游戏内使用”。

## 1. 当前基线与本轮目标

工作区：`E:\CS2AS05`。

第一轮已确认通过：导航、`SkinForgeView.vue`、Pinia store、简化 Loadout schema=2、迁移、Rust 安全 IPC，以及 `npm run typecheck`、Oxlint、ESLint、127 项 Vitest、`npm run build:web`、`cargo check`、`git diff --check`。

第一轮明确未完成：

- `third_party/CS2-Skin-Forge` 只有 `UPSTREAM.md`，没有上游源码、静态表或 DLL。
- `skin_forge.rs` 当前把配置保存到 Tauri app-local-data，`skin_forge_check_plugin` 返回固定缺失结果，部署命令永远拒绝占位 DLL。
- 当前 schema=2 的 `weapons/ct/t` 结构不是上游 C# `PlayerLoadout` 的 JSON 契约，不能直接交给插件。
- 没有固定 CounterStrikeSharp SDK 编译出的 `PlayerSkinMod.dll`，没有部署、FileSystemWatcher 重载或真实 CS2 验收。

本轮只补齐闭环：固定上游快照 -> 固定 SDK 编译真实插件 -> 明确 schema 适配 -> 写入选定 CS2 根目录的插件文件 -> 检查/备份/回滚 -> 离线 `-insecure` 游戏验收。已完成的导航和页面只做必要契约调整，不重做 UI。

## 2. 上游证据与获取顺序

上游基线必须保持：

- 仓库：`https://github.com/emptysuns/CS2-Skin-Forge`
- commit：`b2edea17db9128609dd41f726f179cd965206433`
- 已存在的临时快照：`C:\Users\GOPtZ\AppData\Local\Temp\cs2-skin-forge-plan-20260806`，当前可读，`git rev-parse HEAD` 应输出上述 commit。

执行 AI 先运行：

```powershell
Set-Location E:\CS2AS05
git status --short
$up='C:\Users\GOPtZ\AppData\Local\Temp\cs2-skin-forge-plan-20260806'
git -C $up rev-parse HEAD
rg --files $up\addons\counterstrikesharp\plugins\PlayerSkinMod $up\Panel\src\data $up\Panel\src\utils
```

若临时目录缺失或 commit 不一致，按顺序尝试：`git fetch --depth 1`；GitHub commit zip URL `https://github.com/emptysuns/CS2-Skin-Forge/archive/b2edea17db9128609dd41f726f179cd965206433.zip`；GitHub tree API `https://api.github.com/repos/emptysuns/CS2-Skin-Forge/git/trees/b2edea17db9128609dd41f726f179cd965206433?recursive=1` 加 raw 文件下载。每次失败保存 HTTP 状态/错误，不要改 commit。

复制上游源码、静态表、`LICENSE` 到 `third_party/CS2-Skin-Forge/upstream/`，保留 C# 与原始数据边界，不复制 `node_modules/bin/obj`。在 `UPSTREAM.md` 追加 commit、下载来源、文件清单、复制时间、资源 SHA-256、GPL/第三方许可和未修改声明；临时目录不可作为最终资源路径。

## 3. P0：统一 JSON 契约，解决当前 schema 不兼容

上游 `LoadoutService.ParseLoadout` 期待以 slot 为 key 的对象，而当前前端 `schema=2` 是 `weapons/ct/t` 编辑模型。新增纯函数 `toPlayerSkinModFile(loadout): Record<string, UpstreamPlayerLoadout>` 和 `fromPlayerSkinModFile(value): Loadout`，不能在 Vue 组件中手写转换。

目标文件示例：

```json
{
  "0": {
    "weaponPaintsCt": { "7": 600 },
    "weaponWearsCt": { "7": 0.12 },
    "weaponSeedsCt": { "7": 3 },
    "weaponStickers": { "7": [{ "id": 123, "offsetX": 0, "offsetY": 0, "wear": 0.1, "scale": 1, "rotation": 0 }] },
    "weaponKeychains": { "7": { "id": 1, "seed": 0 } },
    "weaponNametags": { "7": "Local" },
    "weaponStatTrak": { "7": { "enabled": true, "count": 12 } },
    "knifeIndexCt": 507, "knifePaintCt": 44, "knifeWearCt": 0.12, "knifeSeedCt": 1,
    "knifeIndexT": 507, "knifePaintT": 44, "knifeWearT": 0.12, "knifeSeedT": 1,
    "gloveDefIndexCt": 5027, "gloveIndexCt": 0, "glovePaintCt": 10006, "gloveWearCt": 0.12, "gloveSeedCt": 1,
    "gloveDefIndexT": 5027, "gloveIndexT": 0, "glovePaintT": 10006, "gloveWearT": 0.12, "gloveSeedT": 1,
    "agentModelCt": -1, "agentModelT": -1, "agentModelPathCt": "", "agentModelPathT": "",
    "musicKit": -1, "useRandom": false
  }
}
```

具体要求：

1. CT/T 武器 maps、knife/glove/agent/music、贴纸和挂件逐字段映射；`weapon.defindex` 是 JSON object key，不能使用数组 index。
2. legacy shared 字段按上游规则镜像到 CT/T；旧文件逐 slot 容错，单个坏 slot 不影响其他 slot。
3. 保存路径改为选定 CS2 根目录下 `addons\counterstrikesharp\plugins\PlayerSkinMod\player_loadout.json`。应用本地目录只保留草稿/备份，不可作为插件唯一输入。
4. Rust command 从 selectedRoot 解析目标，canonicalize/边界校验，拒绝任意绝对路径；写入采用同目录临时文件、flush/close、rename，并回读返回 `loadoutPath`、`sha256`、slot 数。
5. wear 限制 `0..1`，seed 非负，贴纸最多 5 个；拒绝非 object、超大 JSON、路径越界和非法 defindex。

新增测试必须证明保存后目标文件根对象存在 key `"0"`，字段可被 C# `PlayerLoadout` 读取，旧 schema 仍可读。

## 4. P1：固定 CounterStrikeSharp SDK 并编译 DLL

从上游 `PlayerSkinMod.csproj` 和当前 CounterStrikeSharp API 读取 TargetFramework、包版本和引用；记录 `dotnet --info`、SDK/API 版本、TargetFramework、编译命令、完整日志和 DLL SHA-256。先确认现有 `CS2BotImprover.zip` 的 API 版本，不得用最新版本猜测或覆盖框架。

在 `third_party/CS2-Skin-Forge/upstream/.../PlayerSkinMod/` 执行 `dotnet restore`、`dotnet build -c Release`，把真实 DLL、manifest、`skins_en.json` 复制到 `src-tauri/resources/skin-forge/PlayerSkinMod/`；不提交 `bin/obj`。禁止占位 DLL。

必须保留并静态核对：`OnPlayerSpawn`、GiveNamedItem/OnEntitySpawned、动态属性和 `SetStateChanged`、legacy bodygroup、200ms FileSystemWatcher + `Server.NextFrame`、`skin_menu/skin_random/skin_reset`。插件端 `LoadoutService` 必须保留安全 JSON accessor、旧字段兼容和逐 slot 隔离。

## 5. P2：Rust 检查、部署、备份和回滚

重构 `src-tauri/src/commands/skin_forge.rs`，可拆到 `src-tauri/src/skin_forge/{config,loadout,deploy}.rs`：

1. 复用 `src/stores/cs2.ts` 的 selectedRoot 语义（根目录是 `...\game\csgo`，不要再拼一层）。
2. 目标固定为 `addons\counterstrikesharp\plugins\PlayerSkinMod`。
3. 检查 `PlayerSkinMod.dll/json`、`skins_en.json`、`player_loadout.json`、manifest/version/hash 和 `CounterStrikeSharp.API.dll`。
4. CS2 运行时返回结构化 `CS2_RUNNING`；部署前备份目录，逐文件临时写入并 rename，失败恢复备份。
5. 不自动下载未锁定版本的 CounterStrikeSharp。若另做下载，必须固定 release URL、SHA-256、网络失败回滚。
6. 部署成功返回 `targetDir/files/pluginVersion/hashes/loadoutPath/counterstrikesharpInstalled`，前端显示实际结果。

Tauri resources 必须在 debug、`tauri build --no-bundle` 和最终 bundle 三种布局测试，禁止依赖当前工作目录。

## 6. P3：前端只做契约适配与状态补齐

保持现有五个 tab、CT/T 控件和窄窗口布局。只调整 store/service：保存前调用适配器；无 selectedRoot 时走现有 Tauri dialog；保存成功显示 `player_loadout.json` 路径/hash；插件缺失、版本不匹配、CS2 运行、CounterStrikeSharp 缺失分别显示原因与恢复动作；部署/保存期间禁用按钮并提供 aria-live 反馈；重置需要确认。

遵循已使用的 UI/UX Pro Max 约束：可见 focus、键盘 dialog、至少 44px 交互目标、无横向溢出、错误不只依赖颜色、`prefers-reduced-motion` 下不依赖动画。不引入 React/Tailwind 或第二套主题。

## 7. 自动化验收

新增或更新 `tests/skin-forge-loadout-adapter.spec.ts`、`tests/skin-forge-path-binding.spec.ts`、`tests/skin-forge-deploy-state.spec.ts` 以及 Rust 单元测试，覆盖：schema 每字段映射、CT/T 不串队、空/旧/坏 JSON、selectedRoot 变化、路径越界、CS2 running 拒绝、原子写失败回滚、缺文件/hash/API 缺失、mockIPC 返回路径/hash。

执行：

```powershell
npm run typecheck
npm run lint:oxlint
npm run lint:eslint
npm test -- --run
npm run build:web
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo test --manifest-path .\src-tauri\Cargo.toml --lib
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings
npm run workspace:check
```

## 8. 真实 CS2 验收闸门

用户执行真实游戏操作，执行 AI 准备和记录证据。环境必须离线/本地、优先 Steam 小号、启动选项含 `-insecure`，不得连接 VAC 保护服务器。

1. 退出 CS2，选择正确 `...\game\csgo`，部署插件并记录目录清单、DLL hash。
2. 启动 CS2 + CounterStrikeSharp，保存插件加载日志，确认无 API/加载错误。
3. 选择武器 paint/wear/seed/贴纸/挂件/命名/StatTrak，保存并回读 JSON，重生后截图第一/第三人称。
4. 分别验证 CT/T 刀、手套、角色、音乐盒；切换手套类型后确认涂装合法且无 UV 错位。
5. 修改并再次保存，执行 `skin_menu` 或重生确认 watcher 重载；执行 `skin_random`、`skin_reset` 并记录日志/画面。
6. 记录 CS2、CounterStrikeSharp、插件版本，DLL/JSON hash，日志、截图/Demo 路径。每项核心能力必须有“JSON 回读 + 插件日志 + 游戏内可见”三证据。

## 9. 完成定义与停止条件

只有固定 commit 源码/静态表入 vendor、固定 SDK 编译真实 DLL、schema 适配写入上游 slot JSON、部署绑定 selectedRoot 且可备份回滚、CounterStrikeSharp 无错误、所有核心功能有真实游戏证据、自动化全通过、许可与 `-insecure`/VAC 风险明确，才可写“真实插件移植完成”。

无法获得固定 commit、SDK/API 冲突、DLL 非真实产物、目标路径不确定、CS2 运行中、插件日志报错，或只能提供静态 build/模拟 IPC 而无游戏结果时，立即停止并写报告，不得用前端完成替代插件完成。

## 10. 执行 AI 首步

```powershell
Set-Location E:\CS2AS05
Get-Content .\docs\cs2-skin-forge-integration-execution-report-20260806.md
$up='C:\Users\GOPtZ\AppData\Local\Temp\cs2-skin-forge-plan-20260806'
git -C $up rev-parse HEAD
rg --files $up\addons\counterstrikesharp\plugins\PlayerSkinMod $up\Panel\src\data
```

先完成 P0 JSON 适配测试，再复制并记录 vendor 快照，之后才进入 DLL 编译和部署；不要先改 UI 或生成占位资源。
