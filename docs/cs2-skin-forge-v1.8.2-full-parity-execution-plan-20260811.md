# CS2-Skin-Forge v1.8.2 完整同等能力移植执行方案

日期：2026-08-11（Asia/Shanghai）
工作区：`E:\CS2AS05`
交接对象：下一位“实际执行 AI”（与本方案制定 AI 不共享上下文）
任务性质：在现有工作区继续实现，不新开分支，不回退任何既有改动；完成后必须交付新的 NSIS 安装程序，真实 CS2 游戏内验收由用户执行。

## 1. 任务结论和不可降级的完成定义

当前应用版本为 `0.5.7`。现有“皮肤工坊”已经具备导航、内部 Loadout、Tauri IPC、原子写入、插件固定资源部署、备份恢复和部分适配器测试，但产品层只是一组数字输入表单，不能视为 CS2-Skin-Forge 的完整移植，也不值得要求玩家开始游戏验收。

本轮目标不是继续修补数字表单，而是把上游面向玩家的完整选择体验、数据目录、编辑能力和 PlayerSkinMod 运行逻辑融入现有 Vue/Tauri 应用。完成后的普通玩家应当只通过名称、图片、分类、搜索、筛选和可视编辑完成配置，不需要知道 `defindex`、`paintKit`，也不需要手改 JSON。

下列条件必须同时成立，才能称为“可交给用户做真实游戏验收的安装候选”：

1. 上游 v1.8.2 的武器、皮肤、刀具、手套、角色、音乐盒、贴纸、挂件数据和中英文名称完整纳入固定快照；目录数量和 ID 集合经过自动对账。
2. 上游 Weapon/Knife/Glove/Agent/MusicKit 面板、预览、完整编辑弹窗、队伍切换、随机模式、教程、免责声明、关于和状态能力均有 Vue 3 等价实现。
3. 武器编辑器完整支持皮肤、wear、seed、命名标签、StatTrak、最多 5 张贴纸及每张贴纸参数、挂件及其参数；刀具和手套只显示合法组合。
4. CT/T 独立字段与上游插件真实 JSON 契约一致；不能仅在 UI 中看似独立，写入后却串队。上游契约中本来就是共享的字段必须明确标注，不能虚构插件不支持的独立语义。
5. PlayerSkinMod 源码、manifest、随机池和 `skins_en.json` 更新到 v1.8.2 固定来源并重新构建；不能继续把当前 v1.8.0 DLL 包装成最新版。
6. 现有部署、路径边界、CS2 运行 gate、备份/恢复、哈希回读、FileSystemWatcher 热重载和游戏命令逻辑继续有效。
7. 完整自动化、视觉和安装后冒烟通过，并交付一个全新的 `CS2人机增强助手_<新版本>_x64-setup.exe`。裸 `CS2BotImproverAssistant.exe` 不是玩家交付物。
8. 在以上条件通过之前，不要求用户启动游戏测试。用户只负责最后的离线 CS2 可见效果验收。

以下做法一律不得作为完成证据：只增加更多数字框、让用户查询 ID、让用户手改 `player_loadout.json`、只复制 C# DLL、不迁移目录和编辑器、只证明代码可编译、只交付裸 EXE、用 v1.8.0 数据冒充“最新”、在 UI 同等能力尚未完成时要求用户提前测试。

## 2. 已调查的当前项目状态

项目技术栈为 Vue 3 + TypeScript + Pinia + Tauri 2 + Rust，应用名“CS2人机增强助手”，当前版本 `0.5.7`，Tauri bundle 目标已经是 NSIS。

当前皮肤工坊实现位于：

| 层 | 当前文件 | 可保留内容 | 必须重做或扩展 |
| --- | --- | --- | --- |
| 页面 | `src/views/SkinForgeView.vue` | 导航入口、保存/部署意图、CS2 状态 | 现有单文件数字表单应拆成完整工作台和编辑器 |
| 类型 | `src/types/skin-forge.ts` | CT/T、基础 wear/seed/附件概念 | 默认只有 AK-47/M4A1-S；需覆盖完整目录和明确 schema 迁移 |
| 适配 | `src/features/skin-forge/player-skin-mod-adapter.ts` | slot `"0"`、JSON 双向转换基础 | 用最新版 golden fixture 对账；修正共享附件与 CT/T 语义 |
| 状态 | `src/stores/skinForge.ts` | load/save/deploy/busy/dirty 基础 | 加入目录、草稿编辑、选择状态、筛选、错误和缓存状态 |
| IPC | `src/services/tauri/skinForge.ts` | 命令封装 | 扩展图片缓存、资源/版本诊断时保持强类型 |
| Rust | `src-tauri/src/commands/skin_forge.rs` | 路径规范化、边界校验、1 MiB 限制、原子写、哈希、备份恢复、CS2 运行 gate | 更新资源清单、版本/哈希、契约校验和诊断信息 |
| 固定资源 | `src-tauri/resources/skin-forge/PlayerSkinMod` | Tauri bundle 资源路径 | 用 v1.8.2 重新构建产物替换 v1.8.0 固定产物 |
| 上游记录 | `third_party/CS2-Skin-Forge` | vendor 边界、BUILD/UPSTREAM/SHA256 形式 | 新建 v1.8.2 完整快照和重新生成全部记录 |

现有页面的具体差距：默认武器只有 AK-47 和 M4A1-S；武器、刀具、手套、角色、音乐盒主要依赖数字编号；没有完整图片目录、搜索、分类、图文选择器和实时预览；贴纸按钮只添加 ID 为 0 的空槽，不能选择或编辑 wear/scale/rotation/offset；没有挂件选择界面；没有上游完整编辑器、教程、免责声明、关于和更新信息体验。

现有安全部署闭环不是废代码。执行 AI 应在此基础上扩展，不要绕过或另写一套不受路径/哈希保护的写入路径。

## 3. 最新上游来源、固定版本和来源风险

用户提供的原地址为 `https://github.com/emptysuns/CS2-Skin-Forge`，但在本次调查时匿名 Git 和 GitHub 页面均返回 404/Repository not found。不能把本地旧 commit `b2edea17db9128609dd41f726f179cd965206433` 继续称作“最新版本”。

GitHub 公开搜索和 compare 历史指向迁移仓库：

- 候选公开最新地址：`https://github.com/kaecho/CS2-Skin-Forge`
- tag：`v1.8.2`
- 固定 commit：`75f52fbd5fd0616dbbdd09a65c3a1981593400d1`
- 旧基线：`b2edea17db9128609dd41f726f179cd965206433`
- compare：旧基线是新 commit 的祖先，相差 3 commits / 19 files / 1 contributor
- 关键提交：`e905d09`（v1.8.1 数据同步）、`1ffd0b8`（仓库迁移到 kaecho）、`75f52fb`（v1.8.2 release）

这说明 `kaecho` 是有提交链支持的公开迁移仓库，但原地址已不可公开访问，执行 AI 仍须在复制前重新验证来源链，并把结果写入 `UPSTREAM.md`。不得隐瞒迁移，也不得把“公开迁移仓库”擅自写成“原作者账号未变化”。如 GitHub 历史与上述事实不一致，停止替换固定资源，先在执行报告中说明差异。

### 3.1 获取顺序和网络回退

首选使用独立临时目录，不在项目 vendor 目录直接 clone：

```powershell
$skinForgeTemp = Join-Path $env:TEMP 'cs2-skin-forge-v1.8.2-75f52fb'
git clone --filter=blob:none https://github.com/kaecho/CS2-Skin-Forge.git $skinForgeTemp
git -C $skinForgeTemp checkout --detach 75f52fbd5fd0616dbbdd09a65c3a1981593400d1
git -C $skinForgeTemp rev-parse HEAD
git -C $skinForgeTemp merge-base --is-ancestor b2edea17db9128609dd41f726f179cd965206433 HEAD
git -C $skinForgeTemp status --short
```

网络重置时按以下次序回退，每次耐心重试，但不要无限循环：

1. `git clone` 最多 3 次，每次使用新的、已核实位于 `%TEMP%` 下的目录。
2. 下载固定 commit 的 codeload ZIP：`https://codeload.github.com/kaecho/CS2-Skin-Forge/zip/75f52fbd5fd0616dbbdd09a65c3a1981593400d1`。
3. 使用 GitHub tree API 获取 recursive tree，再逐个 raw 下载；必须对 tree 中每个目标路径建清单，不能只下载“看起来重要”的几个文件。
4. 如果三种方式都无法取得完整快照，停止源码/资源替换并报告阻塞；旧 v1.8.0 快照只能用于结构参考，不能作为最终交付。

获取后必须保存：最终 URL、tag、完整 commit、获取时间、`git show -s --format=fuller`、`git tag --points-at HEAD`、旧新 ancestry、完整文件 SHA-256、是否存在独立 LICENSE、README 中许可声明。vendor 文件原样保存，项目适配改动放在 vendor 外；若必须 patch 上游插件，则保留原件、patch 文件和修改后哈希。

### 3.2 v1.8.1 / v1.8.2 新增内容

相对当前固定的 v1.8.0，已调查到 v1.8.1 数据更新包含：

- 新增 51 款武器皮肤；新增 Arabesque Collection 和 Spy Tech Collection。
- paint kit 数据扩展至 1477。
- 新增 20 张 IEM Cologne 2026 冠军签名贴纸，涉及 NiKo、kyxsan、m0NESY、TeSeS、kyousuke，并覆盖普通、闪亮、全息、金色版本。
- 同步中英文名称、Steam CDN 图片链接、插件随机皮肤池和 `skins_en.json`。

v1.8.2 主要完成仓库迁移、链接和版本发布。最终仍以固定 commit 的实际 diff、数据计数和 ID 集合为准，不以本段文字代替自动对账。

## 4. 上游完整能力基线

本地已验证的 v1.8.0 快照只是结构基线，其中 Panel 已包含约 30+ 武器、约 540 KB 皮肤表、约 3.06 MB / 10,457 行贴纸表、约 180 KB 刀具皮肤表、约 25 KB 挂件表，以及中英文名称和 Steam CDN 图片引用。v1.8.2 必须在此基础上只增不漏。

执行 AI 必须制作一份“上游能力 -> 本项目入口 -> 自动化证据”对照表，至少逐项覆盖：

| 能力 | 玩家可见要求 | 数据/逻辑要求 |
| --- | --- | --- |
| 武器目录 | 按手枪、冲锋枪、步枪、狙击枪、霰弹枪、机枪等分类显示图片卡片；支持中文/英文搜索 | 完整 weapons 表、合法 defindex、队伍可用性 |
| 武器皮肤 | 点击武器进入图文皮肤选择器，显示已选状态、名称和 paint kit 仅作辅助信息 | 完整 skins/skinNames/localNames/nameMap，v1.8.2 ID 集合无遗漏 |
| 武器高级编辑 | wear、seed、命名标签、StatTrak、贴纸、挂件均在同一完整编辑器内可达 | 输入范围、清除/恢复默认、写入插件契约 |
| 贴纸 | 可搜索图片和名称；最多 5 槽；可替换、删除、排序/定位并编辑 wear、scale、rotation、offset | 完整 stickers 表，槽位和数值边界与插件一致 |
| 挂件 | 可搜索图片和名称；可设置/移除，编辑 seed 和上游支持的 offsets | 完整 keychains 表；不能只暴露 ID |
| 刀具 | 图文选择刀型，再选择该刀合法皮肤；实时展示当前选择 | knives + knifeSkins 合法关联；CT/T 按上游契约独立 |
| 手套 | 图文选择手套类型和合法涂装，支持 wear/seed | glove 类型/paint kit/defindex 映射必须与插件一致 |
| 角色 | CT/T 分别展示可用角色卡片和图片，不能用模型数字输入代替 | 模型 path 优先，index 仅兼容；按阵营过滤 |
| 音乐盒 | 展示名称和图片卡片，可选默认/清除 | 写入真实 MusicKitID，不误用数组下标 |
| 队伍 | 清楚可见的 CT/T 分段切换，切换不丢草稿、不串队 | 所有上游支持的 per-team 字段独立 round-trip |
| 随机模式 | 明确开启/关闭和状态解释，可回到自定义配置 | `useRandom`、随机池与插件 StaticData 同步 |
| 预览 | 工作台始终可辨认当前武器、皮肤、队伍和附件摘要 | 图片失败有稳定 fallback，不因图片加载改变布局 |
| 状态与部署 | 显示 CS2 路径、框架、插件版本、资源完整性、运行阻止原因 | 延续现有检查/部署/备份/恢复，不覆盖 CounterStrikeSharp |
| 辅助内容 | 教程、离线/`-insecure` 安全说明、免责声明、关于/来源/版本信息 | 文案中文友好，来源和 GPL 条件真实可查 |
| 游戏命令 | UI 中提供可复制的命令说明，但不依赖命令完成常规配置 | 保留 `skin_menu`、`skin_random`、`skin_reset` 和 watcher |

## 5. 源码、数据和资源迁移映射

不要把 React Panel 整体嵌进 WebView，也不要引入 React/Tailwind 双栈。读取 v1.8.2 的实现和行为，将其转换为 Vue 3 Composition API，并复用本项目的 Pinia、Lucide Vue、按钮、分段控制、模态框和样式 token。

建议新增目录（可依现有命名微调，但职责不可合并回一个巨型页面）：

```text
src/features/skin-forge/
  components/
    ForgeWorkbench.vue
    ForgeHeader.vue
    ForgeStatusBar.vue
    ForgeCategoryTabs.vue
    ForgeTeamToggle.vue
    ForgePreview.vue
    WeaponCatalog.vue
    KnifeCatalog.vue
    GloveCatalog.vue
    AgentCatalog.vue
    MusicKitCatalog.vue
    editors/
      WeaponEditorDialog.vue
      KnifeEditorDialog.vue
      GloveEditorDialog.vue
      StickerEditor.vue
      KeychainEditor.vue
      WearSeedControls.vue
    pickers/
      CatalogPicker.vue
      SkinPicker.vue
      StickerPicker.vue
      KeychainPicker.vue
    dialogs/
      ForgeTutorialDialog.vue
      ForgeDisclaimerDialog.vue
      ForgeAboutDialog.vue
      ForgeResetDialog.vue
  data/
    generated/                 # 从固定上游生成或机械转换；禁止手工删项
      weapons.ts
      skins.ts 或 skins.json
      stickers.ts 或 stickers.json
      knives.ts
      knifeSkins.ts
      keychains.ts
      weaponImages.ts
      skinNamesEn.ts
      localNames.ts
      nameMap.json
      catalog-manifest.json
    catalog.ts                 # 索引、过滤、名称和兼容性查询
  composables/
    useForgeCatalog.ts
    useCatalogSearch.ts
    useImageFallback.ts
    useDialogFocus.ts
  player-skin-mod-adapter.ts
  loadout-migration.ts
  validation.ts
```

上游文件到 Vue 的最低迁移映射：

| 上游 v1.8.2 Panel | 本项目目标 |
| --- | --- |
| `App.tsx` | `SkinForgeView.vue` + `ForgeWorkbench.vue` 的状态编排 |
| `WeaponPanel.tsx` | `WeaponCatalog.vue` |
| `KnifePanel.tsx` | `KnifeCatalog.vue` |
| `GlovePanel.tsx` | `GloveCatalog.vue` |
| `AgentPanel.tsx` | `AgentCatalog.vue` |
| `MusicKitPanel.tsx` | `MusicKitCatalog.vue` |
| `PreviewPanel.tsx` | `ForgePreview.vue` |
| `WeaponEditorModal.tsx` | `WeaponEditorDialog.vue`，所有字段完整迁移 |
| `KnifeEditorModal.tsx` | `KnifeEditorDialog.vue` |
| `GloveEditorModal.tsx` | `GloveEditorDialog.vue` |
| `PickerGrid.tsx` | 通用 `CatalogPicker.vue` + 具体 picker |
| `WearSeedControls.tsx` | `WearSeedControls.vue` |
| `TeamToggle.tsx` | `ForgeTeamToggle.vue` |
| `SettingsPanel.tsx` | 工作台设置/部署区，融入现有 CS2 根目录状态 |
| `StatusBar.tsx` | `ForgeStatusBar.vue` |
| `TabNavigation.tsx` | `ForgeCategoryTabs.vue` |
| `TutorialDialog.tsx` | `ForgeTutorialDialog.vue` |
| `DisclaimerDialog.tsx` | `ForgeDisclaimerDialog.vue` |
| `AboutDialog.tsx` | `ForgeAboutDialog.vue`，保留上游来源和许可 |
| `UpdateBanner.tsx` | 融入本项目已有 updater，不另造不工作的更新入口 |
| `data/*` | `data/generated/*` 原样/机械转换 + manifest + hash |
| `i18n/*` | 合并为现有中文优先文案；保留上游名称搜索别名，不必为工坊引入第二套全局 i18n 框架 |

插件部分完整迁移：

| 上游 PlayerSkinMod | 项目固定快照 / bundle |
| --- | --- |
| `PlayerSkinMod.csproj` | vendor 原样保留，用固定 SDK/package restore 构建 |
| `PlayerSkinModPlugin.cs` | watcher、spawn、GiveNamedItem、命令、随机和模型/音乐逻辑 |
| `Services/LoadoutService.cs` | JSON 读取、兼容和所有附件字段 |
| `Services/WeaponService.cs` | 武器/刀/手套/贴纸/挂件应用逻辑 |
| `Models/PlayerLoadout.cs` | JSON 契约权威来源 |
| `Data/StaticData.cs` | 随机池、刀/手套/角色/音乐等静态逻辑 |
| `PlayerSkinMod.json` | v1.8.2 manifest |
| `skins_en.json` | v1.8.2 完整皮肤表 |

## 6. 数据生成、完整性和图片策略

### 6.1 数据不可手工摘抄

优先原样复制可直接被 TypeScript 使用的数据；如最新版数据仍是 React TypeScript 常量，编写 `scripts/sync-skin-forge-upstream.mjs` 做机械转换和 manifest 生成。脚本输入必须是固定 commit 快照，输出稳定、可复现，不得依赖当前时间改变内容。

`catalog-manifest.json` 至少记录：上游 repo/commit/tag、生成脚本版本、每个源文件和生成文件 SHA-256、字节数、条目数、唯一 ID 数、重复 ID、缺图数、各类/阵营/武器计数。运行两次输出必须字节一致。

必须增加对账测试：

- 每个上游 weapon/skin/sticker/knife skin/keychain/agent/music kit 均能在应用目录中按稳定 ID 查到。
- v1.8.1 的 paint kit 1477 和 20 张 Cologne 2026 签名贴纸存在；最终数量以固定 commit 实际数据为准。
- 所有引用的 weapon/paint/sticker/keychain ID 合法且无意外重复。
- 刀具皮肤和手套涂装过滤不会展示不兼容组合。
- 中文名缺失时回退英文名，再回退稳定技术名；搜索同时命中中文、英文和 ID。
- `StaticData.cs` 随机池、Panel 目录和 `skins_en.json` 的关键 ID 集合有自动 diff；任何差异必须解释。

### 6.2 大目录性能

贴纸超过一万条，禁止首屏一次渲染全部 DOM。目录模块按标签页/弹窗动态 import；搜索建立规范化索引；贴纸和皮肤 picker 使用虚拟列表/虚拟网格（可选维护良好的 Vue 虚拟化库，新增依赖必须记录理由和许可），或者经基准证明的等价窗口化实现。

要求：打开工坊时不解析无关贴纸图片；输入搜索时做 100–180 ms debounce；卡片使用固定 `aspect-ratio` 和尺寸；图片 lazy load；切换筛选不改变工具栏位置；选中状态由稳定 ID 而不是数组下标驱动。以 release build 在 1440×900 下记录首屏、首次打开贴纸选择器、搜索 10k 条目和滚动性能，不能只凭主观“看起来不卡”。

### 6.3 图片和离线回退

先盘点 v1.8.2 的实际图片资源形态。若上游仅保存 Steam CDN/community URL，就完整保留 URL，不得声称安装器已经内置全部图片。采用三层策略：

1. 首选固定目录数据中的上游图片 URL。
2. 通过受控的 Tauri/Rust 图片缓存写入应用数据目录 `skin-forge/cache/images`，按 URL/内容哈希命名；设置连接/读取超时、并发上限、大小/MIME 校验和缓存上限。不要依赖浏览器内置下载行为。
3. 离线、超时、404 或解码失败时显示本地稳定占位图/武器图标和名称，保持完整选择能力与布局，不显示破图。

不得把数万张未知许可、未知体积图片盲目打进 NSIS。若要制作完整离线图片包，必须先给出来源许可、总大小、哈希和安装体积影响，并在执行报告中区分“目录完整”与“位图离线完整”。默认验收要求是完整目录 + 在线图片 + 已缓存图片离线可用 + 全部条目有本地 fallback。

## 7. Loadout 契约、迁移和状态管理

以 v1.8.2 的 `Models/PlayerLoadout.cs` 和 `LoadoutService.cs` 为唯一外部 JSON 权威，不根据当前 TypeScript 类型猜字段。先生成一份由最新版 Panel 保存的 golden `player_loadout.json`，至少包含 CT/T 不同武器/刀/手套/角色、音乐盒、5 张贴纸、挂件、命名、StatTrak、自定义和随机模式，再做双向 round-trip。

建议将内部 schema 从 2 升为 3，但外部文件仍严格使用上游 slot `"0"` 契约：

- 内部对象使用稳定目录 ID，另存真实 defindex/paint kit，不以显示名称作为主键。
- 分离 persisted loadout、当前编辑草稿和 UI selection；取消弹窗应丢弃局部草稿，确认后才写 store。
- 保存前统一 normalize/validate；wear、seed、scale、rotation、offset、StatTrak、名称长度、贴纸 5 槽均按插件边界处理。
- 对 upsteam 明确支持的 CT/T 字段分别保存和读取。现有适配器把附件详情从 `activeTeam` 生成共享字典，必须用 golden fixture 判断这是否是上游真实共享语义；如果共享，UI 标注“两个阵营共用”；如果最新版已有 per-team 附件字段，则完整迁移。不得静默覆盖另一队附件。
- 音乐盒保存真实 ID；角色同时保存 path 和兼容 index；刀/手套同时保持上游所需的数组 index、defindex 和 paint kit 关系。
- 导入现有 v1.8.0 `player_loadout.json`、当前 schema 2 草稿、legacy shared weapon/knife/glove/agent 字段，迁移后 CT/T 对象不得共享引用。
- 保存时继续写选定 CS2 根目录中的真实插件文件；应用数据目录只存草稿、缓存和备份。

迁移测试至少包含：空/损坏/未知字段、schema 1/2/3、legacy shared fields、只有 CT、只有 T、超界 wear/seed、超过 5 张贴纸、未知目录 ID、v1.8.0 golden、v1.8.2 golden、保存后重新读取深度相等。对无法识别但上游可能保留的字段，优先无损 round-trip，不能无提示丢弃玩家配置。

Pinia store 应提供明确状态：`initializing/catalogLoading/loadoutLoading/saving/deploying/dirty/error`，不得用一个 `busy` 覆盖所有原因；离开工坊或切换根目录时对未保存修改给应用内确认；保存成功后显示目标路径、写入时间和回读 hash，不用只有颜色的提示。

## 8. Rust/Tauri 和 PlayerSkinMod v1.8.2

### 8.1 保留现有安全边界

继续保留并补测：支持 CS2 游戏根目录或直接 `game/csgo` 输入；canonicalize；目标必须位于选定根目录边界内；JSON 最大 1 MiB（若 v1.8.2 合法文件可能超过，先用真实 fixture 测量再有证据地调整）；同目录临时写、flush、rename、回读 JSON/SHA-256；部署前完整备份；任一文件/哈希失败全部恢复；CS2 运行时返回结构化 `CS2_RUNNING`；不下载、不安装、不覆盖 CounterStrikeSharp 框架。

插件状态不能只有 `allPresent`，至少返回：选定根、解析后的 `game/csgo`、CounterStrikeSharp 是否存在及检测版本、PlayerSkinMod 是否存在及 manifest 版本、DLL/manifest/skins hash 是否匹配、loadout 是否可读、资源版本、是否可部署、阻止码和中文解释。

图片缓存如新增 Rust command，要限制 URL scheme/host（Steam/community 允许列表或经过验证的上游 host）、响应大小、MIME、文件名和缓存根边界，防止任意文件写入或 SSRF。前端不能传任意目标路径。

### 8.2 构建和替换固定插件

当前 bundle 是 PlayerSkinMod `1.8.0`：DLL 75,776 bytes，SHA-256 `47BF3733D3091D3EAB9E4B86052CBF77A002CFEBAACDEB8D8C43136FF2ADFDEA`，编译时 CounterStrikeSharp.API `1.0.313`。它只能作为回退证据，不能留在 v1.8.2 新安装器中。

执行步骤：

1. 从固定 `75f52...` vendor 源读取 `TargetFramework`、NuGet 版本和插件 manifest，不擅自升级 SDK/API。
2. 在 vendor 外的干净临时输出目录执行 `dotnet restore` 和 `dotnet build -c Release --no-restore`；记录 .NET SDK/runtime、NuGet lock/资产、warning/error。
3. 检查 DLL 元数据和 manifest 均为预期 v1.8.2；静态检查 watcher、200 ms debounce（若最新版数值变化以源码为准）、`Server.NextFrame`、spawn/GiveNamedItem、刀/手套刷新、贴纸/挂件、角色、音乐、随机和三个游戏命令仍存在。
4. 只复制运行必需的 DLL、manifest、`skins_en.json` 等到 `src-tauri/resources/skin-forge/PlayerSkinMod`，不带 `bin/obj`。
5. 更新 Rust 固定资源清单、版本和 SHA-256；重新生成 `third_party/CS2-Skin-Forge/SHA256SUMS.txt`、`BUILD.md`、`UPSTREAM.md`。
6. 用本机已检测到的 CounterStrikeSharp `1.0.371` 做兼容性诊断，但绝不覆盖框架；编译时 API 与运行时 API 的差异必须在报告中明确，真实兼容性最终由离线游戏日志确认。

部署升级必须保留已有 `player_loadout.json`。从 v1.8.0 更新到 v1.8.2 前备份整个 PlayerSkinMod 目录，替换代码/静态资源后重新校验 hash；任何一步失败恢复全部旧文件和 loadout，不能留下混合版本。

## 9. UI/UX Pro Max + Emil Design Engineering 执行要求

界面定位是高频桌面工具，不是营销落地页。保留应用现有整体风格，但让工坊在能力和信息层级上达到上游同等水平。

- 顶层只保留一个明确主操作“应用装备”；“部署/更新插件”是状态驱动的次操作；重置、教程、关于放入清晰但次要的入口。
- 典型布局：顶部标题/状态与主操作；紧凑分类 tabs；左/中为可搜索目录；右侧或稳定区域为当前装备预览/摘要。窄到 1100×700 时改为单列/抽屉式摘要，不能横向溢出。
- 分类、队伍和模式使用 tabs/segmented control；搜索框带 Search 图标和清除图标；图标按钮使用 `lucide-vue-next` 并有 tooltip/`aria-label`；不要用带文字的圆角胶囊替代熟悉图标。
- 卡片保持小于等于 8px 圆角并有固定宽高、图片比例和文本行数；不要卡片套卡片，不要让图片加载、hover 或标签改变网格尺寸。
- 名称是主信息，paint kit/defindex 只作为可选技术辅助信息，不把玩家界面重新变成数据库管理器。
- 颜色不能是一整页单调紫蓝；沿用应用中性深色基底，并用 CT/T、成功、警告、危险的语义色形成层次。状态不能只靠颜色，必须有图标和文字。
- 正文和紧凑面板使用克制字号；禁止按 viewport 宽度缩放字体；letter-spacing 为 0；中文长名称要换行/截断并可通过 tooltip 查看全名。
- 交互目标原则上至少 44×44 px；所有 picker、tabs、slider、dialog 和菜单支持键盘；focus ring 清晰；dialog 有 focus trap、Esc、恢复触发点焦点和正确 ARIA。
- 校验错误就地显示在对应控件旁；保存/部署错误提供结构化原因和可恢复下一步，不使用浏览器 `alert/confirm`。
- 微动效仅服务于理解：hover/选择/弹窗 150–250 ms，优先 transform/opacity；弹窗有触发源感知且可中断；busy 不引发布局移动；支持 `prefers-reduced-motion`。禁止装饰性大动画、渐变球、bokeh 和 hero。
- 编辑器要完整但不拥挤：基础皮肤选择为主区，wear/seed、贴纸、挂件、命名/StatTrak 分为清楚的无嵌套区段；桌面可双栏，窄窗口顺序堆叠。
- 文案用户向：用“选择皮肤”“磨损”“图案模板”“计数器”等可理解中文；技术编号隐藏在“详细信息”中。安全说明明确只用于 `-insecure` 离线/本地环境，不鼓励连接 VAC 保护服务器。

执行 AI 必须用 Playwright 对 release/dev 桌面 WebView 等价页面截图检查至少 `1440×900`、`1100×700` 和一个窄视口；覆盖空状态、完整目录、打开武器编辑器、贴纸搜索结果、图片失败、长中文名、busy/error、reduced-motion。截图中不得有文字溢出、控件遮挡、双滚动条、空白图片塌陷或弹窗超出视口。

## 10. 分阶段实施顺序

### 阶段 A：来源固定和差异审计

- 获取并验证 `kaecho/CS2-Skin-Forge@75f52...`，记录迁移事实、许可和 hashes。
- 输出 v1.8.0 -> v1.8.2 文件 diff、数据计数差异、Panel 能力清单、插件契约 diff。
- 检查工作区 `git status --short`，标记既有用户改动；不 reset/clean/checkout，不新开分支。
- Gate A：完整源码可验证、commit/tag/ancestry 清楚、许可证据已记录。失败则停止后续固定资源替换。

### 阶段 B：可复现数据目录

- 同步全部上游数据并生成 manifest、hash、索引和对账测试。
- 建立名称/搜索/合法组合查询和图片 fallback/cache。
- Gate B：上游/本地 ID 集合对账通过，v1.8.1 新数据存在，重复/缺图/不兼容项均有解释；10k 贴纸目录不一次渲染全部 DOM。

### 阶段 C：Loadout v3 与 golden contract

- 从最新版 C# 模型和 Panel 保存行为建立 golden JSON。
- 完成 schema 3、旧配置迁移、CT/T/shared 语义、附件和完整 round-trip。
- Gate C：v1.8.0、v1.8.2、schema 2 和 legacy fixtures 全部无意外数据丢失；外部 JSON 与插件逐字段一致。

### 阶段 D：Vue 完整工作台

- 按文件映射实现所有目录、预览、编辑器、picker、教程/免责声明/关于和状态。
- 保留现有主导航中“皮肤工坊”位置，不新增第二个重复导航项。
- Gate D：玩家不输入任何技术 ID 即可完成全部配置；上游能力对照表每项都有入口和组件测试；视觉/响应式/键盘验收通过。

### 阶段 E：PlayerSkinMod v1.8.2 和 Tauri 闭环

- 固定构建 v1.8.2，更新 bundle 资源、hash、版本诊断、部署升级和恢复测试。
- 延续安全路径、CS2 running gate、CounterStrikeSharp 不覆盖规则。
- Gate E：干净目标、已有 v1.8.0、缺文件、hash 损坏、写入失败恢复等测试通过；安装资源中不再含 v1.8.0 DLL。

### 阶段 F：自动化、安装器和玩家交接

- 升级应用 patch 版本：当前基线 `0.5.7`，默认目标 `0.5.8`；若执行时已有更高版本，使用下一个未占用 patch，并同步 `package.json`、lockfile、`Cargo.toml`、`Cargo.lock`、`tauri.conf.json` 和所有版本展示。
- 执行全部验证、构建 NSIS、隔离安装、从安装目录启动和卸载冒烟。
- Gate F：只在全部非游戏 gate 通过后交付新安装器和极简真实游戏步骤。不得让用户再从 `target/release` 找裸 EXE。

## 11. 自动化和验收矩阵

### 11.1 前端/数据单元测试

- catalog：数量、唯一 ID、分类、中文/英文搜索、阵营过滤、合法刀/手套组合、图片 fallback。
- adapter/migration：所有字段 golden round-trip、CT/T 不串队、shared 字段提示、旧 schema、未知字段、边界值、最多 5 张贴纸。
- store：初始化、dirty、取消编辑、确认编辑、保存失败保持草稿、部署状态、切根目录提示。
- component：全键盘选择武器/皮肤；添加、编辑、删除 5 张贴纸；第 6 张被明确阻止；挂件、StatTrak、命名；随机/自定义；CT/T 切换；清除和重置。

### 11.2 Rust/Tauri 测试

- 两种 CS2 根输入、canonicalize/边界逃逸、symlink/reparse 风险、文件上限、非法 JSON/slot/字段。
- 原子保存、回读 hash、临时文件清理、备份、部分失败恢复、已有 loadout 保留。
- 插件缺失/旧版/最新版/hash 错误/manifest 错误/skins 错误/CounterStrikeSharp 缺失和 CS2_RUNNING。
- 图片缓存 host/MIME/大小/路径/重复缓存/淘汰/离线错误（如实现缓存命令）。

### 11.3 必跑命令

```powershell
npm run workspace:check
npm run typecheck
npm run lint:oxlint
npm run lint:eslint
npm test -- --run
npm run build:web
cargo fmt --check --manifest-path .\src-tauri\Cargo.toml
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path .\src-tauri\Cargo.toml --lib
git diff --check
npm run bundle:desktop
```

若项目脚本语义发生变化，先读 `package.json` 后使用等价命令。vendor 原样数据可在 lint 中以窄路径排除，但生成器、适配器、组件和项目代码不能整体排除。

### 11.4 安装器硬闸门

构建成功后必须：

1. 找到 `src-tauri\target\release\bundle\nsis\CS2人机增强助手_<版本>_x64-setup.exe`，复制到清晰的交付目录，例如 `workspace\player-acceptance\skin-forge-<版本>-20260811\`。
2. 记录绝对路径、文件大小、SHA-256、ProductVersion/FileVersion、Authenticode、Tauri updater 签名状态。
3. 使用独立临时安装目录做静默或普通隔离安装；验证主 EXE、uninstaller、v1.8.2 DLL/manifest/skins 和必要目录数据均来自安装目录。
4. 从安装目录启动，确认窗口标题、版本、导航、完整皮肤目录、编辑弹窗和进程响应；正常关闭本轮启动的进程，再验证卸载。
5. updater 私钥缺失时可以生成本地测试 NSIS，但不得推送生产更新源、不得声称 updater artifact 已签名。

最终交付报告必须把安装器绝对路径放在开头，不能藏在长文末尾。

## 12. 真实游戏验收（仅在新安装候选完成后交给用户）

用户只执行真实游戏内步骤，文件移动、插件构建、部署准备、安装器构建和非游戏验证均由执行 AI 完成。验收仍分两场景，且都必须从“安装后的助手”操作，不能从源码目录或手工文件开始。

### 场景 A：未安装 PlayerSkinMod

执行 AI 在交接前只准备可恢复备份，不替用户启动游戏。用户安装并打开新助手，确认皮肤工坊能完整浏览和编辑；保持插件未部署状态进入 `-insecure` 离线/本地 CS2，验证助手原有 BOT/命令/启动等功能没有因工坊资源而回归。预期工坊明确显示“插件未部署”，不会写占位 DLL，不会影响其他 CounterStrikeSharp 插件。

### 场景 B：通过安装版助手安装 PlayerSkinMod

用户关闭 CS2，在新助手选择 CS2 根目录并点击“部署/更新插件”；助手完成检查、备份、v1.8.2 部署和 hash 回读。随后用户在 UI 中用图片和名称分别设置明显不同的 CT/T 武器、刀、手套、角色，并设置音乐盒、5 张贴纸参数、挂件、命名、StatTrak，点击“应用装备”。不得要求用户输入 ID 或改 JSON。

用户以 `-insecure` 进入离线/本地环境，收集：

- CounterStrikeSharp 日志显示 PlayerSkinMod v1.8.2 加载，无 API/signature 异常。
- 保存后 `player_loadout.json` 可回读，日志出现 watcher reload。
- 重生后 CT/T 武器、刀、手套、角色、音乐盒分别可见且不串队。
- 贴纸、挂件、命名标签和 StatTrak 可见；wear/seed 有预期差异。
- `skin_random` 开启随机，`skin_reset` 恢复默认，`skin_menu` 行为与上游一致。
- PlayerSkinMod 与现有 Bot Improver 同时运行，原助手功能无回归。

只允许离线/本地 `-insecure` 环境，不连接 VAC 保护服务器。最终“真实移植完成”必须同时拥有三类证据：JSON 回读、插件日志、游戏内可见截图/录像。缺任何一类都只能写“安装候选完成，游戏内待验收”。

为了极速验收，执行 AI 最终只给用户一个安装器路径、一份 6–10 步短清单和日志/截图保存位置；本长方案留给执行 AI，不把内部构建步骤甩给用户。

## 13. 执行边界、停止条件和报告格式

### 13.1 执行边界

- 不新开分支，不 reset/clean/checkout 现有工作，不回退用户和其他 AI 的改动。
- 允许与本任务重叠的既有改动随本次继续演进；修改前必须回读文件，避免覆盖刚落地的变化。
- 不 commit、不 push、不发布生产更新，除非用户另行明确授权。
- 不操作真实 CS2 进程做最终可见验收；需要游戏内确认时交给用户。
- 不覆盖 CounterStrikeSharp，不接管其他插件，不删除玩家原 loadout；所有部署变化可恢复。
- 保留上游来源、README、许可声明和修改记录。若最新 commit 仍无独立 LICENSE，必须如实保留这一风险，不能伪造许可证文件。

### 13.2 必须停止而不能伪装完成的情况

- 无法完整取得或验证 `75f52...`；tag/commit/迁移链不一致。
- 无法确认上游许可/再分发边界，或固定快照缺失关键文件。
- v1.8.2 C# 插件无法固定构建，或安装资源仍混入 v1.8.0。
- Panel 数据、StaticData、`skins_en.json` 的关键 ID 集合无法对账。
- 任何自动化失败、部署恢复失败、安装器内资源 hash 不匹配。
- UI 仍要求普通玩家输入 defindex/paint kit，或缺少任一完整编辑能力。
- 只能交付裸 EXE，或 NSIS 安装后无法从安装目录独立启动。

### 13.3 执行报告必须包含

新增 `docs/cs2-skin-forge-v1.8.2-full-parity-execution-report-<日期>.md`，开头直接写：新版本、安装器绝对路径、大小、SHA-256、签名状态、是否可进入用户游戏验收。

随后记录：

- 最终上游 URL/tag/commit/ancestry/许可和快照 hashes。
- v1.8.0 -> v1.8.2 文件、功能和数据计数 diff。
- 上游功能 parity 对照表，每项对应本项目文件和测试名。
- 新增/修改文件清单，schema/JSON 契约和迁移说明。
- v1.8.2 DLL/manifest/skins 的版本、大小、SHA-256、SDK/NuGet 构建证据。
- UI 截图绝对路径和各视口检查结果；10k 贴纸性能测量。
- 所有自动化命令的实际结果，不能只写“已测试”。
- NSIS 隔离安装、启动、资源回读和卸载证据。
- 未完成的真实游戏证据明确列为“待用户验收”，不得提前写 PASS。
- 用户最终只需执行的极简场景 A/B 清单，以及日志/截图保存位置。

## 14. 最终交接判定

下一位执行 AI 应先完成来源固定和 parity 表，再按 B -> C -> D -> E -> F 顺序持续实施，不能在“数字表单更丰富”处停下。只有完整目录和编辑体验、v1.8.2 插件、自动化、视觉、安装器、隔离安装都通过，才把新安装器交给用户做一次最终离线游戏验收。

本轮方案制定 AI 没有修改业务代码、没有重新部署插件、没有构建新安装器，也没有要求用户测试当前 `0.5.7`。当前 `0.5.7` 仅是前一阶段候选，不是本方案定义的完整移植版本。
