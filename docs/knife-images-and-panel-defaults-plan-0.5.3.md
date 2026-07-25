# 0.5.3 刀具图片与首次默认配置实施方案

本文件是给实际执行 AI 的完整交接依据。执行 AI 不需要本轮聊天、用户截图或其他 AI 的上下文；以本文件列出的当前提交、文件路径、运行契约和验收条件为准。

状态：待实际执行 AI 实施
方案日期：2026-07-25
工作区：`E:\CS2AS05`
当前基线：`main` / `de1da20ca6fb85af780c6ddbccf45215a4a7c939`（0.5.3，尚未推送）
远端基线：`origin/main` / `13b721839494fb1bdeef9bdd70e8b3d20a4b732d`（0.5.2）
目标发布：继续完善并打包 `0.5.3`，不升版，不新开分支，不回退当前已有改动。

## 1. 交付目标

1. 为 `src/views/KnivesView.vue` 中固定的 20 个刀具 subclass 增加与上游 Panel 对应的本地刀具图片，保留中文名称、subclass ID、多选、全选、清空和按键绑定功能。
2. 仅在没有用户选择的首次状态使用以下默认值：

   | 功能 | 默认值 | 磁盘权威状态 |
   | --- | --- | --- |
   | 启动模式 | BOT 模式 | `game/csgo/gameinfo.gi` 与 `backup/WithBots/gameinfo.gi` 一致 |
   | BOT 难度 | 低 | `overrides/botprofile.vpk` 与 `overrides/Low/botprofile.vpk` 一致 |
   | Aim | 混合 / `mixed` | 两个 `cfg/my_bot_*_config.cfg` 中唯一 `bot_aim mixed` |
   | Nades | 正常 / `normal` | 两个 cfg 中唯一 `bot_nades normal` |
   | BOT 物品 | skins/profiles/agents/music 全开 | `BotRandomizer/bot_randomizer_options.json` 四项均为 `true` |
   | 刀具 | 20 项全选 | 两个 cfg 的默认 `\` bind 包含全部允许的 `subclass_create` |

3. 任何可识别的用户已有选择优先于默认值。程序刷新、切换页面、覆盖安装、BOT 启动自动更新和同版本资源更新均不得把用户选择重置为默认值。
4. 默认值不是前端展示占位值。UI 必须显示后端从磁盘解析出的实际状态；首次默认必须真实、原子地写入游戏配置。

## 2. 已调查的当前实现

### 2.1 刀具页现状

- `src/data/panel/knives.ts` 保存 20 个 `[subclassId, 中文名]` 元组：`500, 503, 505, 506, 507, 508, 509, 512, 514, 515, 516, 517, 518, 519, 520, 521, 522, 523, 525, 526`。
- `src/views/KnivesView.vue` 的 `.knife-grid` 当前用 `<span class="knife-silhouette">{{ knife[0] }}</span>` 显示编号占位，没有图片。
- `src/styles/main.css` 中 `.knife-silhouette` 是 44x40 的倾斜文本框；刀卡为 4/3/2 列响应式网格，选中态使用 `aria-pressed`、边框和 `Check` 图标。
- `src-tauri/src/services/panel.rs::set_drop_knives` 已对 ID 白名单、去重、固定顺序、bind key 和 cfg 注入做约束，不需要因图片功能修改后端刀具命令。

### 2.2 上游刀图证据

固定上游为 `ed0ard/CS2-Bot-Improver` tag `v1.4.2` / commit `97fd57d2ee1e14e408ae3ca7b1b0cae596a792cc`，仓库许可证为 AGPL-3.0。

已实时读取固定提交中的以下文件：

- `Panel/src/data/knifeIcons.ts` 使用 `import.meta.glob("../assets/icons/*.png", { eager: true, query: "?url", import: "default" })`，从文件名中的数字生成 subclass ID。
- `Panel/src/panels/DropKnivesSection.tsx` 依据 `KNIFE_ICONS` 渲染 `<img>`，按钮 `title` 为 `subclass_create <id>`。
- `Panel/src/panels/DropKnivesSection.css` 使用 5 列正方形图片按钮、`object-fit: contain` 和选中态。

但固定 tag 的 Git tree、上游当前主分支和 Git 历史均没有提交 `Panel/src/assets/icons/<id>.png`。源码中的 glob 不是可直接复制的图片来源。官方 `v1.4.2` 发布资产 `CS2BotImprover.zip` 内的 `Panel v1.4.2.exe` 为 5,839,872 字节、SHA256 `9C3AB83909E506C0D4BD4886C961DFC0E871DA71BB47E1D1BEA7EF2CCFE40AB2`；用户确认运行中的原版 Panel 能看到刀图。因此实际运行 Panel 的资源响应优先于不完整源码。

### 2.3 当前默认状态

当前内置 `src-tauri/resources/CS2BotImprover.zip` 的实际内容为：

- `gameinfo.gi` 与 `backup/WithBots/gameinfo.gi` SHA256 相同，首次模式已是 BOT。
- `overrides/botprofile.vpk` 与 `overrides/Medium/botprofile.vpk` SHA256 相同，当前首次难度是中，不符合新要求。
- 两个 cfg 只有 `bot_aim_*` alias，没有普通 `bot_aim <value>` 管理行；没有普通 `bot_nades <value>` 管理行。
- `BotRandomizer/bot_randomizer_options.json` 不存在，`read_bot_items` 因此回退为四项 `false`。
- 两个 cfg 的 `\` bind 已包含全部 20 个 subclass，刀具首次状态已是全选。

`src-tauri/src/services/panel.rs::snapshot_at` 当前直接从上述文件解析状态。`read_bot_items` 将“文件不存在/JSON 损坏”和“用户明确全关”折叠成同一个全 false 结果；`parse_drop_knives` 将“从未初始化”和“用户明确清空”折叠成 `("\\", [])`。不能用现有 DTO 的 false/空数组直接判断是否需要默认值。

### 2.4 当前安装与升级路径

- 手动安装：`src-tauri/src/services/cs2.rs::install_bot_package` 先 `remove_upstream_package`，再解压内置 ZIP，会覆盖 cfg、gameinfo、active botprofile 和插件配置。
- BOT 启动自动更新：`ensure_bot_plugin_current` 使用 plugin marker 和事务安装；当前 marker payload 只列出 51 个 NadeSystem 文件，不覆盖 Panel 用户偏好。
- 当前 HEAD 已包含 BOT 插件版本门禁、事务安装、正式标题栏图标和侧栏版本文字。实施本方案时必须与这些现有逻辑协作，不能按旧文档重复实现或删除。
- 当前没有已实现的 `panel-state-v1.json`/偏好初始化层。旧文档中的相关文字只是建议，不是运行事实。

## 3. 刀具图片资源实施

### 3.1 获取顺序

执行 AI 必须按以下顺序获得 20 张真实对应图片：

1. 从当前内置 ZIP 精确提取官方 `Panel v1.4.2.exe`，先验证上述大小和 SHA256。
2. 在隔离临时目录启动该 Panel，使用独立 WebView2 user-data 目录。通过 WebView2/CDP、运行时资源响应、DOM `<button title="subclass_create <id>"> <img ...>` 或可复现的程序资源提取方式，捕获每个 ID 的实际图片字节。
3. 图片 URL 若为 `data:`，解码原始字节；若为 `blob:`，在运行页面上下文读取 blob；若为本地/自定义协议 URL，捕获对应响应。不要截整张卡片后手工裁剪，除非能证明原资源无法读取，并在证据中说明。
4. 每张图片以按钮 `title=subclass_create <id>` 与相邻 `<img>` 的实际映射为准，不能按肉眼猜刀名。
5. 如果运行资源仍无法提取，再使用有明确再分发许可、能稳定映射同一 20 个 CS2 subclass 的来源。必须记录 URL、许可证、获取日期和 SHA256；不得从随机皮肤交易站、搜索缩略图或不明 CDN 抓图。
6. 少于 20 张、ID 重复、空白/损坏、映射不明确时不算完成。不要用生成图、emoji、统一占位图冒充“对应图片”。

### 3.2 本地资产结构

新增：

```text
src/assets/knives/
  500.png
  503.png
  ...
  526.png
  manifest.json
```

`manifest.json` 至少保存：

```json
{
  "upstream": "ed0ard/CS2-Bot-Improver",
  "tag": "v1.4.2",
  "commit": "97fd57d2ee1e14e408ae3ca7b1b0cae596a792cc",
  "panelSha256": "9C3AB83909E506C0D4BD4886C961DFC0E871DA71BB47E1D1BEA7EF2CCFE40AB2",
  "items": [{ "id": 500, "file": "500.png", "sha256": "...", "source": "..." }]
}
```

保持原始 PNG 或做可复现的无损优化；不要依赖远程 URL。单图使用适合 80-120px 卡片的合理尺寸，透明背景保留，禁止把 4K 图片直接塞入安装包。更新 `NOTICE.md`/第三方来源说明，明确代码与图片的上游来源及许可证。

### 3.3 前端数据与布局

将 `src/data/panel/knives.ts` 从元组改为稳定对象：

```ts
type Knife = { id: number; name: string; image: string }
```

显式导入 20 张本地图片，或使用构建期 eager glob 并在模块加载时验证 20 个允许 ID 一一对应。推荐显式映射，避免缺图被静默过滤。

`KnivesView.vue` 每个按钮应包含：

- `<img class="knife-image" :src="knife.image" :alt="knife.name" draggable="false">`。
- 中文刀名和 `subclass <id>`，不要像上游一样只显示图片；这能保持可识别性和无障碍名称。
- 现有 `aria-pressed`、`Check` 图标、点击切换、全选、清空和绑定逻辑。

样式要求：

- 卡片使用稳定 `aspect-ratio`/最小高度，图片容器尺寸固定，`object-fit: contain`，加载状态不能改变网格尺寸。
- 选中状态至少同时具有边框/背景、Check 图标和 `aria-pressed=true`，不能只靠颜色。
- 延续现有 light/dark tokens，不照搬上游白色半透明卡片，不引入外部字体、3D、持续动画或远程图片。
- 960x700 为 4-5 列，720x620 至少 2-3 列；长中文名和 subclass 文本不溢出。
- `prefers-reduced-motion` 下不做位移动画。图片加载失败显示中文名和 ID，但发布测试必须保证正常包中不会触发 fallback。

## 4. 默认值与用户选择优先模型

### 4.1 原则

默认值只解决“没有选择”的情况，不能在 Vue 中写成 `snapshotValue ?? default` 后再自动提交，也不能在每次 `snapshot/refresh` 时无条件写盘。

判定优先级：

1. 有效的本项目偏好状态标记。
2. 当前游戏磁盘上可解析的有效配置。
3. 旧配置备份中能证明的用户操作（主要用于识别刀具清空）。
4. 仅当以上都不存在时使用默认值。

任何合法值都属于用户选择，包括在线模式、中/高难度、Aim 头部/身体、Nades 关闭、BOT 物品任意 false、刀具空数组和自定义合法 bind key。

### 4.2 偏好状态文件

新增应用自有文件，建议：

```text
game/csgo/cfg/cs2as05-panel-state.json
```

该文件不加入内置 ZIP、不由 CS2 执行、不写入 `cfg/plugins`，覆盖安装和插件自动更新必须保留。建议 schema：

```json
{
  "schema": 1,
  "initializedBy": "0.5.3",
  "mode": { "initialized": true, "value": "bots" },
  "difficulty": { "initialized": true, "value": "Low" },
  "aim": { "initialized": true, "value": "mixed" },
  "nades": { "initialized": true, "value": "normal" },
  "botItems": {
    "initialized": true,
    "skins": true,
    "profiles": true,
    "agents": true,
    "music": true
  },
  "dropKnives": {
    "initialized": true,
    "bindKey": "\\",
    "selected": [500, 503, 505, 506, 507, 508, 509, 512, 514, 515, 516, 517, 518, 519, 520, 521, 522, 523, 525, 526]
  }
}
```

文件写入使用现有 `atomic_write`、同目录临时文件、备份和回读校验。损坏时改名为 `.corrupt-<timestamp>`，然后从磁盘配置重新迁移；不能因状态文件损坏而直接恢复所有默认值。

所有 `set_mode`、`set_difficulty`、`set_preset`、`set_bot_item`、`set_drop_knives` 在游戏配置写入成功后同步更新相应状态。若状态写入失败，应恢复本次游戏配置修改或返回明确的部分失败并保留恢复证据，不能 UI 显示失败但磁盘已悄悄改变。

### 4.3 首次安装模板

修改内置 ZIP 模板：

- `gameinfo.gi` 保持 BOT。
- 用 `overrides/Low/botprofile.vpk` 精确替换 `overrides/botprofile.vpk`。
- 在两个 cfg 中各写唯一 `bot_aim mixed` 和 `bot_nades normal` 管理行；不能改变 alias 和其他 cfg 内容。
- 新增 `addons/counterstrikesharp/plugins/BotRandomizer/bot_randomizer_options.json`，四个 bool 为 true，字段名必须与当前 `set_bot_item/read_bot_items` 契约一致。
- 两个 cfg 保持 `\` bind 全部 20 个 subclass。

使用结构化 ZIP 替换脚本，在临时副本中更新后再原子替换资源。输出 before/after entry `{name,size,sha256}` 报告，更新 `CUSTOM_ZIP_SHA256`、fixture、README/NOTICE 和插件 marker 所需内容。官方 `Panel v1.4.2.exe` 必须保持原大小和 SHA256。

### 4.4 现有安装的一次性迁移

新增后端服务 `initialize_panel_defaults(root)`，由专用 Tauri command 在选择根目录后、第一次 `panel.refresh` 前调用，并在 BOT 启动门禁前再次保证完成。它必须幂等，状态文件完整后不得重复写配置。

迁移规则逐字段执行，不能用一个全局 `firstRun` 覆盖所有字段：

- 模式：当前 `gameinfo.gi` 可解析为 online/bots 时保存现值；只有无法识别且安装环境完整时才写 BOT。
- 难度：active botprofile 与 Low/Medium/High 任一文件相同时保存现值；只有无匹配时写 Low。
- Aim/Nades：已有合法管理行时保存现值；缺失时分别写 mixed/normal。
- BOT 物品：JSON 文件存在、schema 有效时保留四个实际 bool（包括全 false）；文件缺失时创建全 true。JSON 存在但损坏时不得静默覆盖，保留 `.corrupt-*` 并报告。
- 刀具：存在合法 managed bind 时保存实际 bind key 和选择；现有安装环境中没有 bind 时优先视为用户明确清空并保存空数组，避免覆盖旧用户。只有能证明是新安装模板时使用 `\` + 全 20 项。
- CS2 正在运行时不做初始化写入，返回 `deferred`；游戏退出后的下次根目录初始化/BOT 启动再执行。

“能证明是新安装”必须来自安装事务上下文，而不是仅凭某个文件缺失。这样才能同时满足新用户全选默认和旧用户清空优先。

### 4.5 覆盖安装与自动更新

在手动 `install_bot_package` 和 BOT 自动安装开始前调用 `capture_panel_preferences`：

- 优先读取有效状态文件；缺失时按 4.4 的磁盘迁移规则捕获每个可识别字段。
- 捕获必须发生在 `remove_upstream_package`/事务复制之前。
- 安装新模板后调用 `restore_panel_preferences`，仅恢复捕获到的字段；没有捕获值的字段保留新默认。
- 恢复和状态文件写入属于安装事务，任一步失败都回滚，不能留下“插件新、用户配置半旧”的状态。
- 用户安装的更高测试版插件被版本门禁保留时，也不得触碰 Panel 偏好。

手动安装按钮的用户交互仍是“明确覆盖安装资源包”，但覆盖的是程序文件，不代表清空用户 Panel 选择。不要新增未经请求的“恢复默认”按钮。

## 5. 文件级修改清单

### 必改

- `src/assets/knives/`：20 张图片和来源 manifest。
- `src/data/panel/knives.ts`、`src/views/KnivesView.vue`、`src/styles/main.css`。
- `src-tauri/src/services/panel.rs`：逐字段初始化、状态迁移、setter 状态同步、偏好捕获/恢复。
- `src-tauri/src/models/panel.rs`：初始化结果/内部偏好 DTO（若对前端公开则保持 camelCase）。
- `src-tauri/src/commands/panel.rs`、`src-tauri/src/lib.rs`、`src/services/tauri/panel.ts`、`src/stores/panel.ts`：初始化 IPC 与根目录首次调用。
- `src-tauri/src/services/cs2.rs`：手动/自动安装事务接入偏好捕获和恢复；资源 ZIP 摘要更新。
- `src-tauri/resources/CS2BotImprover.zip`、`tests/fixtures/panel-v1.4.2/manifest.json`、资源差异报告。
- `README.md`、`NOTICE.md`、`docs/panel-v1.4.2-contract.md`。

### 不改

- 20 个 subclass 白名单、中文刀名、bind key 安全规则和 `subclass_create` 排序。
- BOT 插件版本“当前或更高”的门禁规则、启动特效三个退出条件、在线模式启动参数。
- NadeSystem 投掷策略、原版 Panel EXE 和“猜你想选”目录扫描。
- 版本号仍为 0.5.3。

## 6. 自动化测试

### 6.1 图片契约

- `KNIVES` 恰好 20 项，ID 与 Rust `KNIVES` 白名单完全一致，无重复/缺失。
- 每个图片文件存在、可解码、宽高大于 0、至少包含一定比例非透明像素，不是 1x1 或全透明占位图。
- manifest 的 SHA256 与实际文件一致；所有图片均能追溯到固定上游运行资源或明确许可来源。
- Vue 测试断言每个刀卡有正确 `img src/alt`、中文名、subclass、`aria-pressed`；图片加载不改变卡片尺寸。

### 6.2 默认值矩阵

- 全新安装：bots/Low/mixed/normal/四项 true/20 刀全选。
- 已有 online + High + head + off + 四项 false + 空刀具：初始化、刷新、覆盖安装、自动更新后全部保持。
- 每个字段单独已设置、其他字段缺失：只补缺失字段，不改变已设置字段。
- BotRandomizer JSON 缺失时创建全 true；存在全 false 时保留；损坏时不静默覆盖。
- 刀具无 bind 的既有环境按空选择保存；新安装无历史时全选。
- 状态文件损坏：归档 corrupt 文件，从磁盘恢复，不全量重置。
- CS2 运行中初始化：不写文件，返回 deferred；退出后完成一次。
- 初始化连续调用两次：第二次零文件写入、零新备份。

### 6.3 安装事务

- 手动覆盖和 BOT 自动更新都先捕获偏好，模板安装后逐字段恢复。
- 在模板解压、偏好恢复、状态写入、回读各阶段注入失败，原插件和偏好均恢复。
- ZIP 差异只包含预期 cfg、active Low profile、新 BotRandomizer JSON、plugin marker/摘要变化；Panel EXE 不变。
- 真实路径测试使用备份目录并在 `finally` 恢复，不得直接破坏用户 CS2。

## 7. UI/桌面验收

UI/UX Pro Max 的可用建议只采用图片优化、稳定缩放、清晰选中态、alt/ARIA 和响应式检查；不采用其营销页、3D、外部字体或紫色主题建议。

实际执行 AI 启动 `npm run dev:desktop`，验证：

- 960x700、720x620、1280x800，light/dark。
- 20 张刀图均可见、方向/ID 映射正确、无拉伸和重叠。
- 全选、清空、单项切换、按键捕获后图片和选中状态不跳动。
- 页面切换和应用重启后用户选择保持。
- 图片断网仍正常，证明无远程依赖。

真实刀图是否与上游 Panel 视觉对应，应将同一 subclass 的原版 Panel 与本程序并排展示，由用户最终确认；执行 AI 不得仅凭“图片能加载”宣称对应正确。

## 8. 构建与发布

依次运行：

```powershell
dotnet run --project .\third_party\CS2-Bot-Improver-v1.4.2\nades-per-bot-round-limit\tests\NadePacingPolicy.Tests.csproj -c Release
npm run verify
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run bundle:desktop
```

交付报告必须包含：

- 20 张图片来源 manifest、总大小和 SHA256 清单。
- 新旧 ZIP entry 差异、ZIP SHA256、Panel EXE 不变证明。
- 默认值/用户选择优先测试矩阵结果。
- 3 个窗口尺寸、light/dark 刀具页截图路径。
- 0.5.3 NSIS 安装包绝对路径、字节数、SHA256、生成时间。
- 干净安装、从 0.5.2 升级、现有 0.5.3 同版本覆盖结果。
- 用户仍需执行的真实 CS2/原版 Panel 对照步骤。

版本文件 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 保持 0.5.3。当前 `main` 比 `origin/main` 领先一个未推送的 0.5.3 提交；完成后将本次改动 amend 进该提交，推送前显示 `git status -sb` 和 `git log origin/main..HEAD --oneline`，只普通推送一次，禁止 force push。

## 9. 回退

- 刀图映射不完整：不发布图片功能，保留现有中文名/subclass UI，不提交部分错误映射。
- 默认初始化异常：停用初始化入口，保留磁盘原状态和状态文件证据，不用前端默认值掩盖。
- 覆盖安装恢复失败：从事务备份恢复插件和偏好，不启动 CS2。
- ZIP 非预期 entry 改变：恢复原 ZIP，从干净副本重新做结构化替换。
- 回退只处理本次新增资产、状态文件和相关代码，不回退已有 0.5.3 Panel、插件门禁、启动特效或用户配置。

## 10. 最终交付模板

```text
结果：0.5.3 刀图和首次默认值是否完成
刀图：20/20 映射、来源、哈希、上游对照结果
默认值：bots/Low/mixed/normal/全开/全选
用户优先：已有值、全关、空刀具、升级覆盖测试结果
状态迁移：首次、幂等、损坏、CS2 运行中 deferred
安装事务：手动/自动更新捕获恢复与失败回滚
测试：dotnet、npm verify、fmt、clippy、cargo test
资源：ZIP/Panel/图片 manifest 摘要和差异报告
UI：尺寸、主题、离线图片和交互截图
安装包：路径、大小、SHA256、时间
Git：0.5.3 amend 与单次 push 结果
限制：用户真实 CS2 与原版 Panel 刀图对照待验收项
```
