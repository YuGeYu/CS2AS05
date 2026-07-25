# 0.5.3 Panel 融合实施方案

状态：待实际执行 AI 实施
方案日期：2026-07-25
目标版本：`0.5.3`
当前基线：`main` / `13b721839494fb1bdeef9bdd70e8b3d20a4b732d`（`0.5.2`）
前置调查：[`panel-integration-investigation.md`](./panel-integration-investigation.md)

## 1. 给实际执行 AI 的任务定义

在当前仓库直接开发并发布 `0.5.3`，把 `ed0ard/CS2-Bot-Improver v1.4.2` 的 Panel 功能原生融合进现有 Vue 3 + Pinia + Tauri 2 应用。除删除多语言选择外，保留原 Panel 的全部功能和用户可见状态语义；UI 改造成与本助手现有双主题、自定义标题栏和简体中文风格一致的紧凑桌面工具。

本方案不是要求继续输出方案。实际执行 AI 应完成代码、测试、构建、安装包、安装/升级/卸载验证，必要时直接提交并推送当前 `main`。通常不新开分支，不回退用户当前已有的 `README.md` 和 `docs/` 改动。

完成标准不是“页面已做出来”，而是从干净测试夹具和真实 CS2 目录备份各重放一次，证明融合版与原版 `Panel v1.4.2.exe` 的最终磁盘状态、启动模式和游戏内效果一致，并产出可安装的 `0.5.3` NSIS 安装程序。

## 2. 已冻结的来源与证据优先级

### 2.1 固定来源

| 来源 | 固定提交 | 只用于 |
| --- | --- | --- |
| 本仓库 | `13b721839494fb1bdeef9bdd70e8b3d20a4b732d` | 产品架构、安装安全、主题、发布流程 |
| `ed0ard/CS2-Bot-Improver` tag `v1.4.2` | `97fd57d2ee1e14e408ae3ca7b1b0cae596a792cc` | Panel 功能、DTO、命令数据、交互语义的权威源码基线 |
| 内置 `Panel v1.4.2.exe` | SHA256 `9C3AB83909E506C0D4BD4886C961DFC0E871DA71BB47E1D1BEA7EF2CCFE40AB2`，大小 `5,839,872` | 原版行为黑盒基线 |
| Local-Arena 初版兼容后端 | `5dc04e9c0def2a2d4bb8c35d441607a5a943a954` | 缺失 Rust 后端的公开参考，不作为官方行为证明 |
| Local-Arena 当前调查快照 | `568031eeefaf26f1e4f5fab83f9266a7ecd85e19` | 原子写、路径规范化、BotRandomizer 落盘等后续修复参考 |

开始实施时先执行：

```powershell
git status --short
git rev-parse HEAD
git ls-remote https://github.com/ed0ard/CS2-Bot-Improver.git refs/tags/v1.4.2 refs/heads/main
git ls-remote https://github.com/numakkiyu/Local-Arena.git refs/heads/main
Get-FileHash src-tauri/resources/CS2BotImprover.zip -Algorithm SHA256
```

远端 `main` 可以变化，但本次兼容目标仍固定为上游 `v1.4.2`。若内置 ZIP 摘要不是 README 当前记录的 `D6BCFF73BBACFEBC0BB3ED91A440A584EFE2C812962655F392F319463DA8D9E0`，立即停止发布并先解释资源变化。

### 2.2 证据冲突规则

按以下顺序裁决，不可反过来：

1. 原版 `Panel v1.4.2.exe` 在干净目录上的实际前后差分。
2. 本仓库内置 ZIP 和真实 CS2 安装后的文件内容。
3. 上游 `v1.4.2` Panel 前端调用契约。
4. Local-Arena 公开兼容实现。
5. 注释、README 和推测。

已发现 Local-Arena 初版 `set_bot_item` 只改自己的配置，不能直接照搬；当前实现才会写 `addons/counterstrikesharp/plugins/BotRandomizer/bot_randomizer_options.json`。上游 tag 又没有公开 Rust 后端，且内置 ZIP 默认不存在该 JSON。因此每个写操作必须先做原版 EXE 差分，不能以参考源码替代验证。

## 3. 范围边界

### 3.1 必须交付

- 复用本助手目录扫描、手动选目录、安装、覆盖更新、卸载、诊断和软件更新。
- 在线模式/BOT 模式、`-insecure` 协调和启动 CS2。
- Low/Medium/High 难度读取与切换。
- Aim：`head`、`mixed`、`body`。
- Nades：`max`、`more`、`normal`、`off`。
- Bot skins、profiles、agents、music 四个独立开关。
- 队伍预设：40 支队伍、CT/T 选择、复制完整命令。
- 丢刀按键捕获和 20 个 subclass 多选：`500, 503, 505, 506, 507, 508, 509, 512, 514, 515, 516, 517, 518, 519, 520, 521, 522, 523, 525, 526`。
- 完整 176 行 `commands.txt` 内容、分类标题、搜索、上下一个匹配、点击复制、尾随空格规则和参数提示。
- 文件缺失、目录无效、CS2 运行、待重启、写入失败和复制结果反馈。
- 原版 Panel 兼容入口暂时保留在“安装与诊断”的高级区域，功能验收全部通过后才允许删除。

### 3.2 明确不做

- 不保留语言选择、语言设置和首次语言向导，只提供简体中文 UI。
- 不引入 Local-Arena 的 Preview、安装器、比赛、Demo、真人饰品、枪械/贴纸预设、独立更新、主题队伍皮肤或新插件。
- 不引入 React，不在 WebView 内嵌原 Panel 页面，不启动浏览器实现本地功能。
- 不改变插件 DLL 的功能语义，不替换当前定制 `NadeSystem.dll`。
- 不在用户可见界面加入 Local-Arena 品牌；其代码来源只写入源码注释和 `NOTICE.md`。
- 不通过公网 URL 在运行时加载刀具图。资源必须随应用本地打包；上游 tag 未提交其 `src/assets/icons/*.png`，因此不要引用不存在的目录。

## 4. 先做行为基线，不先画 UI

### 4.1 建立测试夹具

新增：

```text
tests/fixtures/panel-v1.4.2/
  pristine/                 # 从内置 ZIP 提取的最小 game/csgo，可用脚本生成而非提交大文件
  expected/                 # 小型文本/JSON 期望结果
  commands.txt              # 原样复制上游 v1.4.2 的 176 行内容
  manifest.json             # 相对路径、大小、SHA256、来源提交
scripts/panel-diff.ps1      # 快照、执行提示、差分、恢复
docs/panel-v1.4.2-contract.md
```

不要把真实 Steam 路径、用户名或完整大文件提交进 Git。VPK/DLL 夹具从现有受摘要保护的 ZIP 临时提取，测试结束删除。

### 4.2 对原版 EXE 逐操作取证

每次只改变一个变量，流程固定：

1. 从相同 pristine 快照恢复一个隔离的 `game/csgo`。
2. 记录全树相对路径、大小、SHA256，并单独保存文本文件内容和换行风格。
3. 用原版 Panel 选择该目录，只执行一个动作。
4. 再次记录快照，输出新增/删除/改写文件和精确内容差异。
5. 恢复快照，在 CS2 运行和未运行两种状态分别验证允许/拒绝/待重启行为。
6. 记录 Panel 自身配置文件实际位置与内容，禁止假设 Tauri app id。

至少覆盖：首次启动、每个难度、两种模式、启动 CS2、3 个 Aim、4 个 Nades、4 个 Bot item 的开/关、空/单个/全部刀具、按键映射、目录切换、文件缺失、无权限、CS2 运行中修改。把结论写入 `docs/panel-v1.4.2-contract.md` 后才进入后端实现。

## 5. 后端目标架构

### 5.1 文件布局

保持现有安装逻辑，不把 Panel 代码继续堆进 `src-tauri/src/services/cs2.rs`：

```text
src-tauri/src/
  commands/
    mod.rs
    cs2.rs                    # 保留现有安装管理
    panel.rs                  # 新增薄 command 层
  models/
    mod.rs
    panel.rs                  # DTO、受限枚举、结构化错误上下文
  services/
    mod.rs
    panel/
      mod.rs                  # 聚合快照和公共入口
      paths.rs                # 只允许已验证 root 下的 game/csgo
      atomic_fs.rs            # 临时文件 + flush/sync + replace + 写后校验
      config.rs               # 助手自己的 panel-state-v1.json 与迁移
      cfg.rs                  # Valve cfg 行解析/保留换行/定点替换
      mode.rs                 # gameinfo.gi 与 Steam 启动
      difficulty.rs           # botprofile.vpk
      presets.rs              # Aim/Nades
      bot_items.rs            # BotRandomizer JSON
      knives.rs               # bind 与 subclass 白名单
```

`commands/panel.rs` 只做参数反序列化、调用 service 和 `AppError::into_string`；路径校验、备份、写入与状态检测必须在 service 内集中完成。

### 5.2 前后端契约

建议使用带 `panel_` 前缀的命令，避免与当前和未来 command 冲突：

```text
get_panel_snapshot(rootPath) -> PanelSnapshot
set_panel_mode(rootPath, mode) -> PanelSnapshot
set_panel_difficulty(rootPath, level) -> PanelSnapshot
set_panel_aim(rootPath, value) -> PanelSnapshot
set_panel_nades(rootPath, value) -> PanelSnapshot
set_panel_bot_item(rootPath, item, enabled) -> PanelSnapshot
set_panel_drop_knives(rootPath, bindKey, selected) -> PanelSnapshot
launch_panel_cs2(rootPath, mode) -> LaunchResult
```

写命令返回聚合快照而不是 `void`，前端用写后真实状态收敛。DTO 全部 `#[serde(rename_all = "camelCase")]`，TS 端一一镜像。建议字段：

```ts
type PanelSnapshot = {
  rootPath: string
  ready: boolean
  missingFiles: string[]
  cs2Running: boolean
  mode: { current: 'online' | 'bots' | null; insecure: boolean; writable: boolean }
  difficulty: { current: 'Low' | 'Medium' | 'High' | null; available: string[] }
  presets: { aim: 'head' | 'mixed' | 'body' | null; nades: 'max' | 'more' | 'normal' | 'off' | null; writable: boolean }
  botItems: { skins: boolean; profiles: boolean; agents: boolean; music: boolean; writable: boolean }
  dropKnives: { bindKey: string; selected: number[]; writable: boolean }
}
```

不要把原 Panel 的 `language`、`first_run_done` 或目录状态复制到新 DTO；目录权威仍是现有 `useCs2Store.selectedRoot`。

### 5.3 路径和写入安全

- 每次 command 都调用现有 `normalize_root` 的共享版本，接受根目录、`game` 或 `game/csgo`，最终 canonicalize 到根目录。
- 所有目标必须由规范化根目录拼接产生；前端不得传相对目标文件。
- 目录至少验证 `game/csgo/gameinfo.gi`、`cfg` 和本项目安装所需文件。
- 枚举参数在 Rust 端使用 enum 或严格白名单，不接受自由字符串。
- `selected` 仅接受上述 20 个 subclass，去重并按固定顺序排序；`bindKey` 只接受 key-capture 白名单，拒绝引号、分号、换行和控制字符，防止 cfg 注入。
- 不使用 `starts_with("bind ")` 删除任意用户 bind。必须只管理当前已记录的丢刀按键行，或匹配值中完全由允许的 `subclass_create <id>` 构成的行。
- 所有 JSON 用 `serde_json` 读写并保留未知字段；所有 cfg 定点修改，保留未管理行和原换行风格。
- 写入使用同目录临时文件、flush/sync、原子替换、重新读取验证；失败时保留原文件并返回稳定错误码。
- 修改真实目录前创建带时间戳的单文件备份或事务日志；成功后只清理本次临时文件，不删除未知用户备份。

### 5.4 各功能落盘规则

最终规则以第 4 节原版差分为准，以下是实现起点：

| 功能 | 当前已知目标 | 必须验证 |
| --- | --- | --- |
| 难度 | `overrides/<Level>/botprofile.vpk` -> `overrides/botprofile.vpk` | 字节完全一致、缺源文件报错、写后 SHA256 |
| 模式 | `gameinfo.gi` 与 `backup/Online|WithBots/gameinfo.gi` | 原版是复制备份还是重写 SearchPaths；在线模式必须不加载 Metamod |
| 启动 | Steam `-applaunch 730`；BOT 模式含 `-insecure`，原版还显示 `-console/-condebug` | 精确参数、Steam 查找顺序、CS2 已运行行为 |
| Aim | 两个 `cfg/my_bot_*_config.cfg` 的 `bot_aim` | 默认 ZIP 主要存在 alias，真实写入位置与读取回退 |
| Nades | 两个 `cfg/my_bot_*_config.cfg` 的 `bot_nades` | 默认 ZIP 未见普通配置行，真实新增规则 |
| Bot items | `BotRandomizer/bot_randomizer_options.json`，四个 bool | 文件默认不存在时的创建 schema；skins 是否还协调 `core.json` |
| 丢刀 | 两个 cfg 中 `bind <key> "subclass_create ..."` | 旧按键行的精确移除、空选择行为、换行与引号 |

`core.json` 当前内置值 `FollowCS2ServerGuidelines: false`。只有原版差分证明 skins 会改变它时才实施协调；不要因为上游前端调用名 `reconcile_core_json` 就猜测语义。

### 5.5 进程与轮询

- 复用现有 Windows Toolhelp 进程检查，不新增 shell/WMI 轮询。
- 模式切换和启动 CS2 必须在 CS2 运行时拒绝，除非原版差分明确允许。
- Aim/Nades/Bot item/刀具/难度若原版允许运行中写入，则写入后标记该功能 `pendingRestart`；该标记属于前端会话状态，不冒充磁盘状态。
- 当前 10 秒进程轮询保留；Panel 页面可在可见时以 2 秒聚合 `get_panel_snapshot` 刷新，不允许原版 500ms x 6 command 的调用风暴。
- 聚合刷新必须 non-overlapping；隐藏/最小化暂停；静默失败保留最后一次有效快照，不清空 UI、不重复 Toast。

## 6. 前端目标架构

### 6.1 文件布局

不必引入 `vue-router`。该应用只有一个桌面窗口，可用 Pinia/ref 管理当前视图：

```text
src/
  App.vue
  components/
    AppShell.vue
    AppSidebar.vue
    StatusStrip.vue
    ui/
      SegmentedControl.vue
      ToggleSwitch.vue
      SectionStatus.vue
      ConfirmDialog.vue
  data/panel/
    commands.txt
    commands.ts
    knives.ts
  features/panel/
    api.ts
    types.ts
    key-capture.ts
  stores/
    cs2.ts                    # 保留安装和目录权威
    panel.ts                  # 新增功能状态与 pending flags
    navigation.ts             # 可选；也可留在 AppShell
  views/
    OverviewView.vue
    PresetsView.vue
    BotItemsView.vue
    KnivesView.vue
    CommandsView.vue
    InstallView.vue           # 改为“安装与诊断”，保留原功能
```

`src/services/tauri/cs2.ts` 保留；新增 `src/services/tauri/panel.ts`。不要把 Panel invoke 混入软件更新 service。

### 6.2 信息架构

- 左侧固定导航：`概览`、`人机预设`、`Bot 物品`、`刀具`、`命令`、`安装与诊断`。
- 窄窗口改为顶部横向可滚动 tab，不使用浏览器后退/历史 API。
- 全局顶端状态条显示目录、CS2 进程、插件完整性和待重启状态；目录选择放在概览和安装页均可到达，但共用同一 store。
- `概览` 放模式、难度和“启动 CS2”主操作，不做营销 hero。
- `人机预设` 用 segmented controls 表示 Aim/Nades；队伍用原生样式的下拉菜单 + CT/T segmented + 复制图标按钮。
- `Bot 物品` 用四个 toggle，不用文字按钮模拟开关。
- `刀具` 顶部是按键捕获按钮，下方固定 4/5 列选择网格；每项至少显示本地刀具图或 subclass 编号与中文名称，选中态不能仅靠颜色。
- `命令` 保留完整列表、分类标题、搜索高亮、`当前/总数`、上/下一个按钮和点击复制；只滚动命令列表本身，不能 `scrollIntoView` 带动祖先页面。
- `安装与诊断` 保留现有安装、覆盖更新、卸载、软件更新、来源、诊断和原版 Panel 兼容入口。

### 6.3 UI 风格和交互约束

已运行 UI/UX Pro Max 的桌面游戏配置工具检索。采用其中“高信息密度、清晰焦点、Lucide、150-300ms 反馈、reduced-motion”规则，但不采用其 OLED-only 和 Bento 营销布局，因为本项目已有成熟双主题和工具型信息架构。

- 继续使用 `src/styles/main.css` 现有 light/dark semantic tokens，补充 component tokens，不另造一套绿色主题。
- 保留 `AppTitlebar.vue`、44px 标题栏、主题切换、自定义窗口控制。
- 卡片半径不超过 8px；页面 section 不套 card，只有独立控制组/弹窗可用 card。
- 所有命令按钮使用 `lucide-vue-next` 图标；未知图标提供 `title`/tooltip 和 `aria-label`。
- 常用目标至少 44px；正文不小于 13px；焦点可见；状态同时用图标/文字和颜色。
- 页面字体大小不随 viewport 缩放，`letter-spacing: 0`。
- 处理 720x620 最小窗口、960x700 默认窗口、1280x800；必要时把最小宽度降到 680 之前先截图验证文字不溢出。
- 不使用 `alert/prompt/confirm`；沿用应用内 modal/toast。
- 不使用 `navigator.clipboard`。安装并使用 `@tauri-apps/plugin-clipboard-manager`，Rust 注册插件，`capabilities/default.json` 加最小写剪贴板权限。
- 不从网络加载字体或刀具图；沿用系统字体栈。本地刀具资源需记录来源、许可证和 SHA256。

### 6.4 状态模型

`usePanelStore` 至少包含：`snapshot`、`loading`、`mutationKey`、`lastError`、每功能 `pendingRestart`、`refreshInFlight`。行为规则：

1. `selectedRoot` 改变时清空旧根目录的 pending 标记并立即刷新聚合快照。
2. 用户写操作先保存旧快照，可乐观显示选择；后端失败恢复旧快照并 Toast。
3. 后端成功后完全采用返回快照，不用乐观值覆盖磁盘事实。
4. 只有操作返回 `cs2Running=true` 且原版语义要求重启时设置对应 pending。
5. 进程从 running 变 stopped 时刷新并清除已生效的 pending。
6. 背景刷新错误只记日志；用户主动刷新/写入错误才展示 Toast/对话框。
7. `GlobalToast.vue` 不应继续只靠中文正则猜状态；把 store message 升级为 `{ tone, title, message }`，同时兼容软件更新现有调用。

## 7. 数据与依赖迁移

### 7.1 上游静态数据

- 原样复制上游 `97fd57d2` 的 `Panel/src/data/commands.txt` 和解析逻辑，保留命令英文内容，只翻译 7 个分类标题和参数提示。
- 为 `commands.txt` 增加 SHA256/行数测试，要求 176 行和 40 支队伍解析成功。
- key capture 迁移上游 `Panel/src/lib/keycapture.ts` 的 Source key 映射，并在 Vue 中使用 window capture listener；结束、取消、卸载组件时必须清理监听器。
- 刀具 ID 使用固定白名单。上游 tag 引用了但未提交 `src/assets/icons/*.png`，不要生成失效 import。优先从受许可的稳定来源下载 20 张基础刀图并本地化；若无法明确许可，使用本项目自制的统一刀形占位图 + 中文刀名 + subclass ID，功能不依赖远程图片。

### 7.2 新依赖

预计只新增：

```text
npm: @tauri-apps/plugin-clipboard-manager
cargo: tauri-plugin-clipboard-manager
dev only（如确有需要）: tempfile
```

不要引入 React、路由库、组件大套件或状态库。安装依赖后检查 `package-lock.json` 和 `Cargo.lock`，确保只由包管理器更新；禁止全局替换 `0.5.2`，以免误改第三方依赖版本。

### 7.3 本助手配置

建议在 `app_local_data_dir` 写 `panel-state-v1.json`，只保存无法可靠从磁盘恢复的信息：最后模式、Bot item 开关、丢刀按键/选择。目录仍由现有 localStorage key `cs2-bot-improver.selected-root.v1` 管理。写配置必须原子化；损坏时保留 `.corrupt-<timestamp>` 并回退默认值，不覆盖游戏配置。

## 8. 实施顺序与每阶段闸门

### 阶段 A：契约和夹具

- 完成第 4 节黑盒差分及 `panel-v1.4.2-contract.md`。
- 固定静态数据摘要、错误场景和写前/写后样本。
- 闸门：每个目标功能都有“输入 -> 改动文件 -> 返回状态 -> 重启要求”的证据行。

### 阶段 B：只读后端

- 抽取共享路径规范化和目标校验。
- 实现 `PanelSnapshot`，只读真实文件和助手配置。
- 注册 command，写 Rust 临时目录测试。
- 闸门：在 pristine、已修改、缺文件、无权限夹具上读取正确且零写入。

### 阶段 C：写后端

- 按 Difficulty -> Aim/Nades -> Bot items -> Knives -> Mode/Launch 顺序实现。
- 每项先单元测试，再对原版快照差分；一次只推进一个功能。
- 闸门：成功、无权限、缺源文件、中断恢复、CS2 运行五类测试通过。

### 阶段 D：前端壳和页面

- 先做 AppShell/导航/全局状态，再逐页接真实 command。
- 保留 InstallView 行为，不在迁移布局时改安装逻辑。
- 接入可靠剪贴板、键盘捕获和结构化 Toast。
- 闸门：Vitest 覆盖每个控件的 invoke 参数、失败回滚、pending、目录切换和复制。

### 阶段 E：兼容验收

- 对所有写操作跑原版/融合版双基线差分。
- 用真实 CS2 备份验证 online <-> bots 往返和游戏内最终状态。
- 验证原版 Panel 兼容入口仍可启动，不与融合版同时写同一目录。
- 闸门：第 10 节矩阵全通过，否则不删除兼容入口。

### 阶段 F：0.5.3 发布

- 精确修改版本、文档、NOTICE、更新文案和测试契约。
- 完整 verify、Rust tests、bundle、安装/升级/卸载/持续启动。
- 生成 SHA256、安装包大小、时间戳和 Git commit；按用户习惯可直接推送当前 `main`。

## 9. 文件级修改清单

### 必改

- `src/App.vue`：替换单页直挂为 AppShell，保留 titlebar/toast。
- `src/views/InstallView.vue`：迁移为安装与诊断页，版本文案改 0.5.3，保留原版 Panel 高级入口。
- `src/stores/cs2.ts`：仅做结构化消息和与 panel store 的目录联动，不混入 Panel 细节。
- `src/styles/main.css`：拆分/补充壳层、导航和 Panel controls 样式；不得破坏现有 modal/update/about。
- `src-tauri/src/lib.rs`：注册 clipboard 插件和所有 panel commands。
- `src-tauri/src/commands/mod.rs`、`models/mod.rs`、`services/mod.rs`：注册新模块。
- `src-tauri/capabilities/default.json`：最小 clipboard 写权限。
- `package.json`、`package-lock.json`、`src-tauri/Cargo.toml`、`Cargo.lock`：依赖及精确版本字段。
- `src-tauri/tauri.conf.json`、`NOTICE.md`、`README.md`：0.5.3、来源和用户说明。
- `config/workspace/projects.json`：描述从“Panel 启动入口”改为“集成 Panel 功能”。

### 新增

- 第 5、6 节列出的 Panel Rust/Vue 模块。
- `tests/panel-*.spec.ts`：导航、store、命令、队伍、按键、剪贴板和可访问性。
- Rust 模块内单元测试或 `src-tauri/tests/panel_integration.rs`。
- `docs/panel-v1.4.2-contract.md` 和差分脚本。

### 不应改

- `src-tauri/resources/CS2BotImprover.zip` 及摘要常量，除非用户另行要求更新资源。
- `third_party/.../nades-per-bot-round-limit/` 定制源码和 DLL。
- 软件更新服务器/线上 manifest，除非本仓库已有明确发布凭据和流程。

## 10. 验收矩阵

| 场景 | 必须结果 |
| --- | --- |
| 无目录 | 所有写控件禁用，可扫描/选择，页面无异常 |
| 目录无效 | 后端拒绝，错误指出缺失项，不产生文件 |
| 插件未装 | 概览提示安装，Panel 控件不可写，安装入口可用 |
| 完整安装 | 快照与磁盘一致，刷新不闪空 |
| Low/Medium/High | active VPK 与对应源 SHA256 相同 |
| Aim 3 值 | 两个目标 cfg 的管理行一致，其他行字节语义不变 |
| Nades 4 值 | 同上 |
| 四个 Bot item | 独立开关、重启提示、JSON schema 与原版一致 |
| 刀具空/单/全选 | 丢刀 bind 正确、无重复、旧受管 bind 被精确更新 |
| 40 支队伍 | CT/T 命令逐字复制，缺一侧时明确禁用/提示 |
| 命令搜索 | 大小写不敏感、上下循环、尾随空格保留、只滚列表 |
| Online | `gameinfo.gi` 不加载 Metamod，启动不含 `-insecure` |
| Bots | `gameinfo.gi` 正确加载插件，启动含 `-insecure` |
| CS2 运行 | 模式/安装/卸载等危险写被阻止；允许项显示待重启 |
| 写入失败 | 原文件保留，临时文件清理，UI 回滚并给稳定错误 |
| 目录切换 | 旧状态/pending 不串到新目录 |
| 隐藏窗口 | 停止高频 Panel 刷新，恢复可见后立即刷新一次 |
| 720x620 | 导航、长路径、命令、弹窗无重叠或裁切 |
| light/dark | 文本、focus、disabled、success/warning/danger 均清晰 |
| 升级 0.5.2 -> 0.5.3 | 已选目录和用户游戏配置保留，融合页可直接读取 |
| 卸载应用 | 不删除用户 CS2 配置；插件卸载仍遵守现有范围 |

## 11. 自动化验证命令

每个阶段至少运行对应子集，发布前按顺序全跑：

```powershell
npm install
npm run workspace:check
npm run typecheck
npm run lint
npm run test
npm run build:web
npm run verify
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run bundle:desktop
```

UI 完成后启动 `npm run dev:desktop`，用真实桌面窗口截图验证 960x700、720x620、1280x800 的 light/dark 状态。不要只凭截图宣称功能完成；截图之外必须附 invoke/文件差分/Rust test 证据。

NSIS 验证：

1. 记录安装包路径、大小、SHA256、生成时间。
2. 干净安装后持续运行至少 30 秒并打开每个导航页。
3. 从已安装 0.5.2 覆盖升级，验证目录选择和配置保留。
4. 安装目录含空格、非 ASCII 用户目录下验证一次。
5. 卸载后确认快捷方式/程序目录按预期删除，CS2 目录和助手用户配置保留策略符合文案。

## 12. 版本与发布注意事项

只精确修改以下项目自身版本字段：

- `package.json` 顶层 `version`，随后用 npm 更新 lock 顶层项目版本。
- `src-tauri/Cargo.toml` 当前 package `version`，随后由 Cargo 更新 `Cargo.lock` 中 `ai_pc_fac`，不可全局替换。
- `src-tauri/tauri.conf.json` `version`。
- README、NOTICE、InstallView、测试中的产品版本文案。

`Cargo.lock` 中存在其他依赖的 `0.5.2`/`0.5.20`，绝不能批量替换。发布说明应明确：Panel 已原生融合、保留原版兼容入口、仅简体中文、插件资源仍基于上游 v1.4.2 且定制 ZIP 未变化。

## 13. 来源与许可证

- 本项目继续 `AGPL-3.0-or-later`。
- 复制上游 commands/key mapping/交互逻辑时，在 `NOTICE.md` 记录 `ed0ard/CS2-Bot-Improver`、tag、提交和用途。
- 复制或实质改写 Local-Arena 的 Rust 实现时，在对应 Rust 文件头用简短注释记录仓库、提交和修改范围，并在 `NOTICE.md` 源码归属区记录；不要称其为“官方 Panel 后端”。
- 用户界面“关于与来源”继续突出本项目和 `ed0ard/CS2-Bot-Improver`，不展示 Local-Arena。
- 刀具图若来自第三仓库，必须先确认许可证允许再分发，写入来源、具体提交和资源摘要；否则使用自制本地图形。
- 发布安装包时同步提供对应完整源代码。

## 14. 失败回退

- 后端某一功能未通过差分：隐藏该融合控件并保留原版 Panel 入口，不用猜测实现凑齐 UI。
- UI 壳回归安装管理：先恢复旧 `InstallView` 作为“安装与诊断”页，不回退已验证的独立 Panel 后端。
- 模式往返失败：立即恢复测试前 `gameinfo.gi` 和 Steam 启动状态，停止真实 CS2 测试。
- 发布安装包冒烟失败：不推送 release，不覆盖最后一个已验证安装包；保留构建日志和 SHA256。
- 任何回退只处理本次新建/修改的文件，不删除用户原有 CS2 文件、未知备份或当前工作区已有改动。

## 15. 最终交付报告模板

实际执行 AI 完成后必须报告：

```text
结果：0.5.3 Panel 原生融合是否全部完成
功能：模式/难度/Aim/Nades/Bot items/队伍/刀具/命令逐项状态
差分证据：原版与融合版快照/报告路径
测试：npm verify、cargo fmt/clippy/test 的结果
真实 CS2：online <-> bots 往返和游戏内确认结果
UI：各尺寸 light/dark 截图路径
安装包：绝对路径、字节数、SHA256、时间戳
安装验证：干净安装、0.5.2 升级、卸载、持续启动
Git：commit、push/release URL（如已执行）
限制：仍保留原版 Panel 入口的原因或其他未完成项
```

只有上述证据完整，且第 10 节没有未解释失败，才可把任务标记为完成。
