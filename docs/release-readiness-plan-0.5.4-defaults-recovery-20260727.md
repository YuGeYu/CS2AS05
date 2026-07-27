# 0.5.4 发布收尾、默认值调整与 Panel 恢复源码交接方案

## 1. 文档目的与角色边界

本文是交给下一位【实际执行 AI】的收尾方案。当前只做调查、契约冻结和实施编排，不在本轮修改业务代码、不修改真实 CS2 目录、不写入 D1/R2、不上传 GitHub、不构建新的安装包。

仓库：`E:\CS2AS05`

目标版本：`0.5.4`

上游：`ed0ard/CS2-Bot-Improver v1.4.3` / `d1d83982db88fbdb686b2bf13aa8c6f9d65a4604`

前一份迁移方案：[`docs/upstream-v1.4.3-release-plan-0.5.4.md`](./upstream-v1.4.3-release-plan-0.5.4.md)

本文件只补充以下收尾内容：

1. 冻结新的首次安装默认值。
2. 记录部分用户的探员模型和丢刀开关可能在游戏内无效，且本轮不修复。
3. 评估 `E:\破解\Panel-v1.4.3-recovered` 的可用证据和不可直接采用的部分。
4. 把签名、真实 CS2、提交、线上发布前的剩余工作排成可执行顺序。

不要 reset、checkout、restore、clean 或回退当前工作区。当前已有改动通常允许随本次版本发布，必须先审阅再增量修改。

## 2. 当前状态与发布判断

### 2.1 已确认的本地候选证据

当前候选安装器：

```text
workspace/runtime/cs2-bot-improver/release-evidence/0.5.4-candidate-20260726_205151/CS2人机增强助手_0.5.4_x64-setup.exe
size   69,389,566 bytes
sha256 C3D42B1B5501044ABE766C4F9E4FC2BBBC27F2F6B2C8488CC6162303A4C398C8
```

候选定制包：

```text
sha256 862021C84EECD32D2E332430E82CD921C56D93015A27DED4156A32FE373F2637
```

Panel：`5,844,480` bytes，SHA256 `3FD93DC7AF2702C50B9A7E4FCF1BB11387B107ABC863EE8A3067255022408CCD`。

NadeSystem DLL：`70,144` bytes，SHA256 `87E68BF9C0B4C46845F36A16981C7F4A0A5754ECDA9A1D5CCC016F73C1AF490A`。

已确认：前端 72 项测试、Rust 25 passed/1 ignored、Clippy、Nade 51 assertions、三视口 UI 和隔离安装均通过。Rust 被 ignored 的真实游戏目录往返测试不能由桌面安装验证替代。

### 2.2 当前仍不是正式发布版

以下事实必须继续写入报告，不得把候选版称为正式版：

- 当前 EXE `NotSigned`，没有 `0.5.4` updater `.sig`。
- `TAURI_SIGNING_PRIVATE_KEY` 和密码环境变量目前缺失。
- `dist-release/cs2-bot-improver/updater-prod.json` 仍是旧的 `0.5.3` manifest，不能发布。
- 当前 `main` 仍为 `ba51d0b`，0.5.4 改动尚未形成提交；工作区有大量已暂存、未暂存和未跟踪文件。
- 用户尚未完成真实 CS2 的 `less`、八项 Bot Items、`br_reroll` 和三个正常回合节奏验收。

因此当前准确结论是：**功能候选版已完成，本地未签名候选版已验证，正式发布条件尚未全部满足。**

## 3. 0.5.4 首次安装默认值契约

### 3.1 新安装默认值

在一套没有有效 `cs2as05-panel-state.json`、没有已管理用户选择的全新插件目录上，必须得到：

| 设置 | 默认值 | 写入/验证位置 |
| --- | --- | --- |
| 启动模式 | `bots` / BOT 模式 | `game/csgo/gameinfo.gi` 对应 `backup/WithBots/gameinfo.gi` |
| BOT 难度 | `Low` / 低 | `overrides/Low/botprofile.vpk` 复制到活动文件 |
| Aim | `mixed` / 混合 | 两个 `cfg/my_bot_*_config.cfg` 各一行 `bot_aim mixed` |
| Nades | `less` / 较少 | 两个 cfg 各一行 `bot_nades less` |
| BOT 物品 | 八项全部 `true` | `addons/counterstrikesharp/configs/core.json` |
| 丢刀按键 | 继续使用默认 `\` | 两个 cfg 的受管 bind |
| 刀具 | 仅 `507, 508, 515, 519, 525` | 两个 cfg 的受管 `subclass_create` bind |

八项顺序和 key 固定为：

```text
profiles, agents, music, weapons, knives, gloves, stickers, charms
```

“全开”必须落盘为八个官方键的 JSON 布尔值 `true`，而不是只在前端显示为开启。缺失官方键仍按上游行为视为 `true`，未知 `core.json` 字段继续保留。

### 3.2 新用户与旧用户必须分开处理

不要为了改变默认值而覆盖旧用户的合法选择：

- 全新安装或字段完全未初始化时，使用本节新默认值。
- `0.5.3 -> 0.5.4` 覆盖升级时，保留用户已经明确写入的 mode、difficulty、Aim、Nades、八项 Bot Items 和刀具选择。
- 旧用户没有刀具 bind 或没有对应偏好时，才使用本节的五把刀默认；不得把已有空选择强行变成五把。
- 旧用户已经明确选择 `normal` Nades 时，升级后仍保留 `normal`；只有缺失/未初始化字段才使用 `less`。
- 旧 `skins` 和 `bot_randomizer_options.json` 不迁移、不解析、不主动删除。

这一区分必须在测试中显式覆盖，不能用“全新默认测试通过”替代升级保留测试。

### 3.3 当前代码的确定性调整点

实际执行 AI 应先确认当前工作树版本，再修改：

- `src-tauri/src/services/panel.rs`
  - 初始化 Nades 的 fallback 从 `normal` 改为 `less`。
  - 新安装且没有 managed knife bind 时，fallback 从全部 `KNIVES` 改为 `[507, 508, 515, 519, 525]`。
  - 把五把刀提取为命名常量，避免测试和业务各写一组裸数字。
  - 不改变已有显式状态的捕获/恢复逻辑。
- `src-tauri/src/services/cs2.rs` 及安装初始化路径
  - 确认新安装恢复偏好时不会把五把刀默认误判为空旧偏好。
- `src/views/PresetsView.vue`、`src/views/OverviewView.vue`
  - UI 只回读后端 snapshot；不要用前端 fallback 把 `normal` 或全刀显示成新默认。
- `README.md`
  - 将“首次 Panel 默认值”从 `normal / 20 刀全选` 改为 `less / 507、508、515、519、525`。
  - 明确写“覆盖升级保留已明确选择”。
- `docs/panel-v1.4.3-contract.md`、`docs/CS2BotImprover-defaults-diff-0.5.4.json`
  - 更新默认值证据、输入摘要和最终产物摘要；历史 0.5.3 文档不全局替换。

## 4. 已知兼容性限制：探员模型和丢刀可能无效

### 4.1 结论

用户已在本项目和官方 `Panel v1.4.3.exe` 中观察到：

- 取消“探员模型”后，真实游戏内可能仍然使用探员模型。
- 取消刀具/丢刀后，真实游戏内可能仍然继续出现刀具。

这说明问题可能来自 CS2、CounterStrikeSharp、相关插件、游戏版本、服务器状态或本机环境，而不一定是助手 UI 或 Rust 写盘错误。因为官方 Panel 在同一环境也出现相同现象，本轮**不修复、不绕过、不宣称所有用户都能生效**。

### 4.2 文档和发布口径

在 0.5.4 发布说明、README 的兼容性说明或帮助文档中加入以下事实性文案，避免写成保证：

> 兼容性提示：部分用户的 CS2/插件环境可能无法在游戏内应用“探员模型”和“丢刀/刀具”开关。官方 Panel 在相同环境也可能出现相同限制。助手仍会按契约写入并回读配置；本提示不代表每台电脑都能得到对应的游戏内效果。

不要把失败显示为“保存失败”，也不要在没有游戏证据时显示“已生效”。现有 UI 的后端成功回读仍可保留；本限制属于游戏环境兼容性，而非本轮要新增的修复项。

### 4.3 验收边界

真实 CS2 验收仍检查：

1. 设置是否正确写入 `core.json`/cfg。
2. 应用重启后是否正确回读。
3. 在用户环境中记录探员模型和丢刀的实际结果。

若写盘和回读正确但游戏内无变化，报告为“配置链通过，游戏内效果受环境限制”，不要回滚本版本，也不要擅自增加新的插件、命令或强制清理。

## 5. Panel v1.4.3 恢复源码评估

### 5.1 已验证的输入

用户提供的恢复工程：`E:\破解\Panel-v1.4.3-recovered`。

关键输入：

- 目标 Panel：`5,844,480` bytes，SHA256 `3FD93DC7AF2702C50B9A7E4FCF1BB11387B107ABC863EE8A3067255022408CCD`。
- 恢复源码 ZIP SHA256：`BA5CA1388D3C58038EA765744869BCDED39D368C521C3F39007316F9076C52A9`。
- `verify-recovery.ps1` 已通过：`ipc=20 bot_items=8 nades=5`。
- 用户说明 `npm run build`、`cargo check` 已通过；恢复工程不是已上传的 GitHub 来源。

恢复工程的 `source/src-tauri/src/lib.rs` 还明确保留 20 个 IPC 名称、八项映射、`core.json` 布尔校验和 unknown fields 保留逻辑，可作为当前实现的交叉核对材料。

### 5.2 可直接用于本项目的证据

只把以下内容作为验证参考：

- 八项 key 顺序和官方 `core.json` key 映射。
- `profiles` 对应 `bot_hider github.com/XBribo all`。
- 其余七项对应 `bot_randomizer github.com/ed0ard <item>`。
- `core.json` 必须是 JSON object，受管字段必须是 bool，未知字段不得因切换被删除。
- IPC surface、错误边界、目录校验和“CS2 运行中变更需重启”的行为线索。
- 目标 Panel 的哈希和 `verify-recovery.ps1` 作为来源审计附件。

### 5.3 不得直接复制的内容

恢复工程明确是行为等价重建，不是逐字源码：编译器删除的局部变量名、注释、精确模块边界和部分实现无法证明与官方相同。因此：

- 不用恢复工程替换官方 `Panel v1.4.3.exe`。
- 不把恢复工程的整个 Rust backend 合并进 CS2AS05。
- 不用它覆盖现有事务写盘、插件门禁、安装回滚和 updater 逻辑。
- 恢复工程自己的 `AppConfig::default()` 仍是 `Medium / normal / 空刀具`，与本次用户要求不同；不能把它当作产品默认真值。
- 恢复工程公开 `commands.txt` 仍不是本项目最终含 `less` 和 `br_reroll` 的命令数据；命令以本项目固定文件和测试为准。
- 恢复工程的 `bot_items` fallback 使用 Rust `bool` 默认值，不能替代本项目“缺失官方 key 视为 true”的已确定契约。

### 5.4 建议保存的证据

实际执行 AI 可在本仓库 `workspace/runtime/panel-v1.4.3-recovered-evidence/` 保存：

- 恢复 ZIP 的 SHA256。
- `verify-recovery.ps1` 的完整输出。
- source/upstream/local-arena-reference 的来源说明。
- 与本项目 `core.json` snapshot 的字段级对照。

该目录只保存证据，不把恢复工程源码复制进产品资源，不把它当作上游官方源码发布。

## 6. 实施顺序

### 阶段 A：冻结与备份

1. `git status --short`、HEAD、当前版本字段和候选产物摘要全部留档。
2. 备份真实 `game/csgo` 前不得启动真实写盘验收；开发测试优先使用隔离夹具。
3. 对当前 `src-tauri/resources/CS2BotImprover.zip` 和候选安装器做只读摘要记录。
4. 运行恢复工程的 `verify-recovery.ps1`，把输出写入证据目录。

### 阶段 B：默认值实现

1. 修改 `panel.rs` 的新安装 fallback 和命名刀具常量。
2. 加入新安装 snapshot、cfg 两文件、core.json 和 bind 的断言。
3. 加入升级用例：显式 `normal`、空刀、Bot Items 全关均保留；缺失字段才使用新默认。
4. 更新 README、Panel v1.4.3 contract 和默认值差分报告。
5. 不改变探员模型/丢刀的运行时实现，只补兼容性文案和发布说明。

### 阶段 C：测试与候选重建

```powershell
npm run workspace:check
npm run typecheck
npm run lint
npm test
npm run build:web
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo test --manifest-path .\src-tauri\Cargo.toml
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings
dotnet run --project .\third_party\CS2-Bot-Improver-v1.4.3\nades-pacing\tests\NadePacingPolicy.Tests.csproj -c Release
git diff --check
```

UI 只维持现有紧凑桌面工具规范：`720x620`、`960x700`、`1280x800`，light/dark、reduced-motion、键盘 focus 和无横向溢出。不要采用 UI/UX 搜索工具返回的 3D/WebGL/紫色赛博风建议；现有产品应继续使用稳定、可扫描、无持续动画的设置界面。若加兼容性说明，使用现有 warning/说明样式，不新增装饰色、卡片嵌套或弹窗。

### 阶段 D：签名最终候选

1. 用户或发布环境提供与 `src-tauri/tauri.conf.json` pubkey 匹配的 Tauri 私钥和密码；执行 AI 不输出私钥内容。
2. 设置 `TAURI_SIGNING_PRIVATE_KEY`、`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`，确认 `Test-Path Env:` 均为 True。
3. 运行 `npm run verify` 后构建 `npm run bundle:desktop`。
4. 确认最终 EXE、`.sig`、资源 ZIP、Panel、NadeSystem 摘要。
5. 用 `RELEASE_CHANNEL=prod npm run release:manifest` 生成真正的 `0.5.4` manifest，禁止沿用现有 0.5.3 manifest。
6. 对签名候选重新做新装、30 秒启动、0.5.3 升级、进程关闭、sentinel 保留和卸载验证。

### 阶段 E：用户真实 CS2 验收

这是必须由用户完成的步骤，执行 AI 不得代替或推断：

1. 退出 Steam/CS2 相关运行状态，完整备份 `D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo`，实际路径以用户机器为准。
2. 安装最终签名 `0.5.4`，在助手中确认首次默认显示 BOT、低、混合、较少、八项全开和五把刀。
3. 安装插件包后执行 `bot_nades less`，再执行 `bot_nades`，确认回显 `less` 且无 usage error。
4. 连续完成至少三个正常回合，观察开局预算、同 Bot 5 秒间隔、紧急投掷和 less 总上限，保存 `game/csgo/console.log`。
5. 逐项切换八项 Bot Items，重启 CS2 后回读；记录探员模型取消和刀具取消在该机器上的实际结果，并按第 4 节口径报告可能失效。
6. 搜索并复制 `br_reroll`，执行后确认下一次安全出生是否重新抽取外观。
7. 退出 CS2，重新打开助手，确认 less、八项和刀具状态回读；不要在游戏运行时修改文件。

### 阶段 F：提交、线上发布与回退

真实验收通过且用户明确同意正式发布后才执行：

1. 审阅全部 dirty files，确认没有误把恢复工程、私钥、临时证据或用户数据加入提交。
2. 创建可追溯的 `0.5.4` 提交；普通 push 最多一次，不 force push。
3. 保存生产 D1 latest/settings、R2 对象清单和当前 feed 响应。
4. 保持 updater 总开关关闭，先上传 EXE/签名对象，写完整 artifact 元数据。
5. 验证 R2 HEAD、GET、下载哈希/大小、custom API、Tauri feed、CORS 和夸克回退。
6. 验证无误后再切换 D1 latest 和 updater 总开关；旧对象最后处理。
7. 用户确认后创建 GitHub Release，附 EXE、`.sig`、SHA256 和已知兼容性说明。群公告、内测邀请由用户自行发布。

回退顺序：先关闭 updater 开关，再恢复 D1 latest；保留旧 R2 对象和本地旧安装包；游戏目录只使用用户确认的完整备份恢复，不递归删除未知文件。

## 7. 最终报告必须包含

```text
结果：0.5.4 候选/正式版
默认值：bots / Low / mixed / less / 八项全开 / 507,508,515,519,525
默认值测试：新安装与升级保留分别通过/失败
Panel 恢复源码：目标哈希、恢复 ZIP 哈希、verify-recovery 输出路径
已知限制：探员模型和丢刀可能在部分用户游戏内无效，是否复现及官方 Panel 对照
候选 EXE：路径、size、SHA256、Authenticode
updater：.sig 路径、签名验证、manifest 版本/size/hash
自动化：npm/Rust/Clippy/dotnet/UI 结果
真实 CS2：备份路径、console.log 路径、less/节奏/八项/br_reroll/回读结论
Git：提交、分支、是否 push、是否 tag
线上：D1/R2/feed/API/GitHub 是否执行
回退：快照路径和仍未完成的用户步骤
```

在用户真实 CS2 验收和签名 manifest 均完成前，报告只能写“本地候选”，不得写“正式发布完成”。

