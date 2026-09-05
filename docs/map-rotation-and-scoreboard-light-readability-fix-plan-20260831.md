# MapRotation 默认状态与亮色本局战报可读性修复交接方案

日期：2026-08-31  
工作区：`E:\CS2AS05`  
交接对象：实际执行 AI  
状态：已调查，待执行；本文件仅制定方案，不代表业务代码、插件、安装器或实机已修复。

## 1. 用户问题与完成标准

### 1.1 MapRotation

用户在助手关闭自动换图后，游戏控制台仍返回：

```text
[Client] [MapRotation] enabled=1, current=de_vertigo, index=8, next=de_cache
```

用户随后退出 CS2、把外部配置设为 `enabled=false`、再次启动 BOT，仍然看到 `enabled=1`。执行 AI 必须证明：

- 助手写入的绝对文件与插件 `Load()` 读取的绝对文件 canonical 后相同；
- `enabled=false` 在 CS2 已退出时写入同一文件，下一次插件载入后首次 `lbtv_map_rotation` 返回 `enabled=0`；`true` 流程返回 `enabled=1`；
- 运行中外部改 JSON 不热覆盖插件内存 `_enabled`，界面明确文案为“下一次插件载入生效”；
- `lbtv_map_rotation 0|1` 仅改变运行时内存、不回写 JSON；
- BOT 启动/升级不会用 ZIP 内默认 JSON 覆盖用户的 false；
- 新安装器和插件资源来自当前工作树，不再出现历史 `FC8868...` / `76D9...` 摘要。

### 1.2 亮色战报

用户打开 `auto-20260826-1404-de_dust2-advent.dem` 的独立“本局战报”时，右下角 `LBRating 2.0 · lb-rating-2.0` 可见，但标题、地图、比分、玩家表格正文几乎与浅色背景同色，主要内容不可读。截图证据：

`C:\Users\GOPtZ\Pictures\Screenshots\屏幕截图 2026-08-29 104907.png`

完成标准：

- 亮色模式战报所有正文、标题、表头、表格单元格、队伍行、摘要字段均有稳定深色前景和 WCAG AA 级对比度（普通文字至少 4.5:1，大字至少 3:1）；
- 深色模式、调色板、圆角、密度仍跟随现有外观设置；
- 不通过隐藏内容、降低透明度或固定暗色背景掩盖问题；
- 1280x800、980x640 和移动宽度截图无裁切、重叠、横向不可滚动关键内容；页脚仍显示 `LBRating 2.0 · lb-rating-2.0`。

## 2. 已调查事实与最可能根因

### 2.1 MapRotation 运行链路

- 助手写入服务：`src-tauri/src/services/map_rotation.rs`，固定路径为
  `game/csgo/addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`。
- 插件读取：`third_party/CS2-Bot-Improver-map-rotation/addons/counterstrikesharp/plugins/MapRotation/MapRotation.cs` 的 `Load()` 使用 `Server.GameDirectory` 拼接同一相对路径，并把 JSON `enabled` 复制到进程 `_enabled`。
- `lbtv_map_rotation` 查询 `_enabled`，不会每次读取 JSON。因此写文件后当前已载入插件仍显示 1 属于时序语义；退出后重新载入仍显示 1 才是路径、旧 DLL/旧安装器、启动覆盖或根目录绑定缺陷。
- 当前 `MapRotationDefaultControl.vue` 已提示“只影响下一次 MapRotation 载入”，但返回 DTO 尚无可审计的 hash/read-back 字段，不能单凭 UI 状态证明插件读取了该文件。
- `scripts/generate-plugin-manifest.ps1` 已将 MapRotation JSON 归入 `mutableConfigEntries`；执行 AI 必须验证实际安装资源仍遵守该契约。

### 2.2 亮色战报现状

- 独立入口 `src/scoreboard.ts` 只加载 `src/styles/scoreboard.css`，随后调用 `initializeAppearancePreferences()`。
- `src/styles/scoreboard.css` 在 `:root` 定义 `--scoreboard-text:#18212b` 等浅色变量，`data-theme='dark'` 才切换深色变量；但 `.scoreboard-shell`、`body`、`table/th/td` 没有显式设置 `color:var(--scoreboard-text)`，大量元素依赖继承。
- 用户截图中 `.rating` 的颜色规则生效，普通正文却接近白色，说明存在继承/浏览器默认样式/运行时主题变量覆盖或透明度链路；执行时必须以实际窗口 DevTools/受控浏览器 `getComputedStyle()` 回读为准，不能只看源码猜测。
- 不能把 `LBRating` 页脚可见误判为整体主题正确；需要逐元素回读颜色、背景、opacity、visibility 和最终 stylesheet 来源。

## 3. 实际执行顺序

### P0-A：MapRotation 配置与运行时对账

1. 修改 `src-tauri/src/services/map_rotation.rs` 的 DTO，增加 `configSha256`、`readBackEnabled`、`observedAt`、`loadSemantics:"next-plugin-load"`；`set/reset` 写入后立即 read-back JSON、绝对路径和 SHA-256，并记录 `[MAP_ROTATION_DEFAULT_UPDATED]`。
2. 在助手启动 BOT 前和插件 `MapRotation.cs::Load()` 中记录 canonical 根目录、绝对配置路径、文件 SHA-256、解析后的 enabled 和 `Server.GameDirectory`。两边路径字符串必须能直接对账。
3. 审查 `install_game_files_transactionally()`、`launch_cs2_inner()` 及 marker 迁移：固定资源可更新，但不得无条件复制 ZIP 内 `MapRotation.json` 覆盖用户值；迁移/升级前后都保存并回读用户 false/true。
4. 保留运行命令语义：CS2 运行时禁写外部 JSON；命令 `0|1` 只改内存，退出后 JSON bytes/SHA 不变。
5. 重新生成并校验插件清单，确保 JSON 只在 `mutableConfigEntries`，固定 payload hash 与当前 ZIP/marker/payload digest 一致。

### P0-B：亮色战报可读性

1. 首先在实际独立战报窗口或受控页面回读以下节点的 computed style：`html`、`body`、`.scoreboard-shell`、`.scoreboard-titlebar strong/span`、`.match-band` 内 `span/strong`、`.honors`、`table`、`thead th`、`.team-row th`、`.player-row td`、`.detail-row`、`footer`。记录 `color/backgroundColor/opacity/visibility/filter` 与命中的 CSS 规则。
2. 在 `src/styles/scoreboard.css` 建立明确的作用域基线：`html, body, #scoreboard-app, .scoreboard-shell { color:var(--scoreboard-text); background:var(--scoreboard-bg); opacity:1; }`；`table, th, td, .match-band strong, .honors strong, .scoreboard-titlebar strong` 显式继承/使用 `var(--scoreboard-text)`；辅助文本统一 `var(--scoreboard-text-muted)`。不得依赖浏览器默认黑色或跨入口 `main.css`。
3. 检查 `initializeAppearancePreferences()` 与主题事件：战报入口必须先应用持久化 `data-theme/data-palette/data-radius/data-density` 再挂载；若浅色变量被主程序变量覆盖，改为 scoreboard 专属变量或提高选择器作用域，不能删除深色支持。
4. 检查是否有 `opacity`、disabled 样式、动画初始态或父级透明度导致截图中的正文“洗白”。加载完成后正文不得处于过渡初始态；为 `.scoreboard-shell`、`.post-scoreboard`、`.scoreboard-table-wrap` 增加稳定可见性断言。
5. 依据实际回读调整浅色前景：正文建议 `#18212b` 或更深，muted 不低于 `#52606d`；队伍/Rating 颜色在浅色背景上分别做对比度验证。不要只给 rating 加颜色而留下 K-D-A/ADR/HS% 不可读。
6. 保持表格容器可滚动和固定最小宽度；移动视口允许横向滚动表格，但摘要标题、页脚和关闭按钮必须可见，不能用 `overflow:hidden` 裁掉内容。

## 4. 自动化测试与证据

### 4.1 MapRotation

新增/更新：

- `tests/map-rotation-settings.spec.ts`：true/false read-back、路径/hash/loadSemantics 文案、CS2 运行时禁用、错误状态；
- Rust `map_rotation`：原子写入、canonical 路径、SHA 回读、缺失/损坏/fallback、CS2 running 拒绝；
- Rust `cs2/panel`：MapRotation 不在固定 payload、mutable entry 存在、升级前后用户 false 保持；
- 插件受控 fixture/C# 测试：Load 读取 true/false，命令只反映内存。

建议命令：

```powershell
Set-Location E:\CS2AS05
powershell -ExecutionPolicy Bypass -File .\scripts\generate-plugin-manifest.ps1
npm test -- --run tests/map-rotation-contract.spec.ts tests/map-rotation-settings.spec.ts tests/bot-plugin-gate.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml cs2 map_rotation panel
dotnet build .\third_party\CS2-Bot-Improver-map-rotation\addons\counterstrikesharp\plugins\MapRotation\MapRotation.csproj -c Release --nologo
```

### 4.2 亮色战报

新增 `tests/scoreboard-light-readability.spec.ts` 或扩展现有战报测试，至少覆盖：

- `data-theme=light` 下根节点和所有正文关键选择器的 computed color 不是接近背景的白色；
- `opacity=1`、`visibility=visible`、无未结束的入场透明动画；
- 亮色/深色均存在 `LBRating 2.0` 和 `lb-rating-2.0`；
- 旧 `simple-rating-v1/v2` 报告仍被门禁拒绝；
- 1280x800、980x640、375px 下无关键控件越界，表格按设计滚动；
- 使用真实浏览器/受控 Tauri 页面截图，而非仅字符串测试。可用 Playwright 或项目既有截图工具，但必须回读 `document.documentElement.dataset` 和关键节点 computed style。

建议补充命令：

```powershell
npm test -- --run tests/post-match-scoreboard.spec.ts tests/scoreboard-light-readability.spec.ts
npm run typecheck
npm run lint
npm run build:web
```

截图至少保存：

```text
E:\CS2AS05\artifacts\scoreboard-light-1280x800-20260831.png
E:\CS2AS05\artifacts\scoreboard-light-980x640-20260831.png
E:\CS2AS05\artifacts\scoreboard-light-375x800-20260831.png
E:\CS2AS05\artifacts\scoreboard-dark-1280x800-20260831.png
```

## 5. 真实验收流程

### 5.1 MapRotation

1. 确认 CS2 与 BOT/插件已退出，助手选择唯一正确的 CS2 根目录。
2. 设置 false，记录助手 DTO 的绝对路径、JSON bytes、`readBackEnabled=false`、SHA-256 和日志。
3. 从当前源码构建/启动 BOT；回读安装后 marker version、ZIP/marker/fixed payload 四段 hash。
4. 等插件 `Load()`，保存其 canonical 路径、SHA、enabled 日志；执行 `lbtv_map_rotation`，必须为 `enabled=0`。
5. 重复 true 流程，必须为 `enabled=1`。
6. 运行中执行 `lbtv_map_rotation 1`/`0`，确认只变内存；退出后 JSON hash/bytes 未被命令改写。

### 5.2 亮色战报

1. 使用用户指定 Demo 或同 schema/metrics 的受控报告打开独立窗口。
2. 回读 `data-theme=light`、关键节点 computed style、正文对比度和 `scoreboardReady=true`。
3. 确认地图、比分、报告回合、MVP、玩家 K-D-A/ADR/HS%、队伍行、按钮和页脚均清晰可读。
4. 切换深色再切回亮色，重新打开窗口，确认外观偏好一致且无依赖旧窗口缓存。
5. 保存四种截图和 JSON/文本回读；截图中右下角必须同时出现 `LBRating 2.0 · lb-rating-2.0`。

## 6. 停止条件与交付报告

任一条件成立即停止，不得宣称完成：

- CS2 重启后 false 仍返回 enabled=1，或助手/插件路径、SHA 不一致；
- 新安装资源仍出现历史 `FC8868...`/`76D9...`，或 MapRotation JSON 回到固定 payload；
- 启动/升级覆盖用户配置；
- 亮色正文任一关键区域 computed color 仍接近背景、对比度不达标、opacity 不为 1、文字被裁切/重叠；
- 只有源码/单元测试，没有真实战报截图或插件 Load 日志。

执行 AI 最终必须更新执行报告，写明：修改文件绝对路径、MapRotation JSON 内容/hash、助手与插件路径对账、true/false 两次命令回读、四段资源 hash、亮色关键节点 computed-style/对比度、四张截图路径、自动化结果、未签名安装器路径/SHA（若构建）及剩余用户实机验收项。证据未闭环时，状态只能写“第二阶段修复候选/待真实验收”。
