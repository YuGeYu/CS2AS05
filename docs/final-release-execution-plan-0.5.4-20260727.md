# 0.5.4 最终发布执行交接方案

## 1. 交接边界

本文交给下一位【实际执行 AI】。目标是在不回退当前工作区改动的前提下，把 `0.5.4` 从当前候选状态推进到可正式发布状态；本轮方案 AI 不结束用户进程、不操作真实 CS2 目录、不注入签名私钥、不上传 GitHub、不写生产 D1/R2。

仓库：`E:\CS2AS05`

基线提交：`main@ba51d0b`；当前 `0.5.4` 改动仍在 dirty worktree，尚未形成新提交。

上游：`ed0ard/CS2-Bot-Improver v1.4.3` / `d1d83982db88fbdb686b2bf13aa8c6f9d65a4604`。

前置方案：

- [`docs/upstream-v1.4.3-release-plan-0.5.4.md`](./upstream-v1.4.3-release-plan-0.5.4.md)
- [`docs/release-readiness-plan-0.5.4-defaults-recovery-20260727.md`](./release-readiness-plan-0.5.4-defaults-recovery-20260727.md)

不要使用 `git reset`、`checkout`、`restore`、`clean` 或新建分支。先审阅当前 dirty 文件，再增量修正。

## 2. 当前事实快照

### 2.1 已完成

- `src-tauri/src/services/panel.rs` 已有 `DEFAULT_KNIVES = [507, 508, 515, 519, 525]`。
- 新安装 Nades fallback 已为 `less`。
- 新安装默认模式/难度/Aim/Bot Items 逻辑分别为 BOT/Low/mixed/八项全开。
- README 的 0.5.4 默认值文字已更新为 `less` 和五把刀。
- 定制 ZIP SHA256：`862021C84EECD32D2E332430E82CD921C56D93015A27DED4156A32FE373F2637`。
- Panel SHA256：`3FD93DC7AF2702C50B9A7E4FCF1BB11387B107ABC863EE8A3067255022408CCD`。
- NadeSystem DLL SHA256：`87E68BF9C0B4C46845F36A16981C7F4A0A5754ECDA9A1D5CCC016F73C1AF490A`。
- 未签名候选安装器：

```text
workspace/runtime/cs2-bot-improver/release-evidence/0.5.4-candidate-20260726_205151/CS2人机增强助手_0.5.4_x64-setup.exe
size   69,389,566
sha256 C3D42B1B5501044ABE766C4F9E4FC2BBBC27F2F6B2C8488CC6162303A4C398C8
```

### 2.2 当前阻塞和环境注意事项

- `TAURI_SIGNING_PRIVATE_KEY`、`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` 当前均不存在。
- 候选没有 `.sig`，不能用于生产 self-update。
- `dist-release/cs2-bot-improver/updater-prod.json` 仍是 `0.5.3`，必须由最终签名构建重新生成。
- `docs/panel-v1.4.3-bot-items-contract.json` 尚未生成；如无法补齐，报告中必须明确这是方案证据缺口。
- 本次复核时检测到 `cs2.exe`、Steam 和隔离验证残留的 `CS2BotImproverAssistant` 仍运行。Cargo 两个测试因 `[CS2_RUNNING]` 保护而失败，不能判定为代码失败，也不能在未确认归属时强制结束用户进程。
- 本次误用 `npm test -- --runInBand`；Vitest 不支持该 Jest 参数。正确命令是 `npm test` 或 `npx vitest run`。

## 3. 默认值和兼容性契约

### 3.1 全新安装

必须从磁盘 snapshot 验证：

```text
mode       bots
difficulty Low
aim        mixed
nades      less
botItems   profiles/agents/music/weapons/knives/gloves/stickers/charms = true
dropKnives 507,508,515,519,525
bind       \\
```

检查两个 cfg 的唯一受管行、`core.json` 八个 bool 和两个文件回读，禁止只检查 Vue 显示。

### 3.2 覆盖升级

旧用户已经明确设置的值必须保留，包括 `normal` Nades、全关 Bot Items 和空刀具。只有字段缺失或从未初始化时才使用新默认。旧 `skins`、旧 `bot_randomizer_options.json` 不迁移、不删除。

### 3.3 已知游戏兼容性限制

探员模型取消和丢刀/刀具取消在部分用户环境可能无游戏内效果；用户已确认官方 Panel 在相同环境也可能如此。本版本不新增修复、不绕过、不把配置回读成功写成游戏必然生效。发布说明应使用：

> 部分用户的 CS2/插件环境可能无法在游戏内应用“探员模型”和“丢刀/刀具”开关；官方 Panel 在相同环境也可能存在限制。助手仍会写入并回读配置，实际效果以用户游戏环境为准。

## 4. 实际执行步骤

### 阶段 A：收敛环境并重跑自动化

1. 记录 `git status --short`、进程列表、候选摘要；不要自动结束 `cs2.exe` 或未知桌面进程。
2. 让用户确认已退出 CS2，或仅结束本次隔离验证创建且明确属于本次任务的助手进程。
3. 正确执行：

```powershell
npm run verify
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo test --manifest-path .\src-tauri\Cargo.toml
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings
dotnet run --project .\third_party\CS2-Bot-Improver-v1.4.3\nades-pacing\tests\NadePacingPolicy.Tests.csproj -c Release
git diff --check
```

4. 保留完整 stdout/stderr。Rust 应达到 `25 passed / 1 ignored` 或当前实际数量全部通过；ignored 的真实目录测试必须明确原因。Vitest 使用当前项目实际输出，不添加 `--runInBand`。
5. 如果 CS2 进程关闭后仍有失败，才进入代码诊断；禁止用修改测试或跳过测试消除失败。

### 阶段 B：补齐固定证据

1. 运行 `E:\破解\Panel-v1.4.3-recovered\verify-recovery.ps1`，保留 `OK target_sha256=... ipc=20 bot_items=8 nades=5` 输出。
2. 生成 `docs/panel-v1.4.3-bot-items-contract.json`，至少包含：Panel hash、八项顺序、每个 core key、默认值、读取/写入前后 JSON 摘要、unknown fields 保留结果和测试命令。
3. 不把恢复工程 Rust 代码并入产品；它只作为行为等价交叉证据。恢复工程自己的 `Medium/normal/空刀具` 默认不覆盖本项目新契约。

### 阶段 C：签名最终构建

这是需要用户/发布环境提供密钥的步骤：

1. 注入与 `src-tauri/tauri.conf.json` 公钥匹配的 Tauri 私钥和密码，执行 AI 不打印值。
2. 确认：

```powershell
Test-Path Env:TAURI_SIGNING_PRIVATE_KEY
Test-Path Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```

3. 执行 `npm run bundle:desktop`，确认产生 `0.5.4_x64-setup.exe` 和同名 `.sig`。
4. 校验 `.sig` 可由 Tauri 公钥验证；Authenticode 状态单独记录，不能与 Tauri 签名混称。
5. 生成正式 manifest：

```powershell
$env:RELEASE_CHANNEL = "prod"
npm run release:manifest
```

6. `dist-release/cs2-bot-improver/updater-prod.json` 必须是 `version=0.5.4`，installer path、size、SHA256 与最终 EXE 一致，signature 来自最终 `.sig`，不能残留 0.5.3 字段。

### 阶段 D：最终桌面隔离验证

对签名后的最终 EXE 重跑，而不是沿用未签名候选报告：

1. 全新安装、启动 30 秒、正常退出。
2. `0.5.3 -> 0.5.4` 升级，确认旧进程关闭、sentinel/app-data 保留。
3. 卸载时分别报告程序文件删除和用户 sentinel 保留；不能把保留 sentinel 的目录存在报告为纯失败。
4. 安装后重新核对 marker、ZIP、Panel、DLL 的摘要。

### 阶段 E：用户真实 CS2 验收

只能由用户完成，执行 AI 不得模拟结论：

1. 退出 CS2 后完整备份实际 `game/csgo`，路径以用户机器为准。
2. 安装最终签名版，确认首次默认显示 BOT/低/混合/较少/八项全开/五把刀。
3. 控制台执行 `bot_nades less`，再执行 `bot_nades`，确认回显 `less`、无 usage error。
4. 完成至少三个正常回合，保存 `game/csgo/console.log`，观察开局预算、同 Bot 间隔、紧急投掷和 less 上限。
5. 逐项切换八项 Bot Items，重启后回读；记录探员模型取消和刀具取消是否只写盘成功但游戏内无效果。
6. 搜索、复制、执行 `br_reroll`，确认下一次安全出生是否重抽外观。
7. 退出 CS2 后重开助手，确认 less、八项和刀具状态回读。

### 阶段 F：提交与线上发布

只有自动化、签名隔离验证和用户实机报告齐全后执行：

1. 审阅所有 dirty 文件，排除私钥、真实游戏备份、临时构建目录和恢复源码副本。
2. 创建可追溯 `0.5.4` 提交；普通 push 最多一次，不 force push；可创建 `v0.5.4` tag。
3. 保存生产 D1/R2 latest、settings、对象清单和 feed 快照。
4. 保持 updater 总开关关闭，先上传最终 EXE/签名并写完整 artifact 元数据。
5. 验证 R2 HEAD/GET、custom release API、Tauri feed、CORS、夸克回退和下载哈希。
6. 验证通过后再切 D1 latest 和 updater 总开关；旧对象最后处理。
7. 用户确认后创建 GitHub Release，附 EXE、`.sig`、SHA256、上游来源和已知兼容性限制。
8. 群公告、内测邀请由用户执行。

## 5. 回退与停止条件

- 签名失败：保留未签名候选，不切生产，不伪造 manifest。
- 真实 CS2 写盘或恢复失败：停止实机测试，使用用户完整备份恢复；不递归删除未知文件。
- D1/R2/feed 任一 hash、size、signature 不一致：保持 updater 关闭，恢复旧 latest。
- Git 工作区出现无法归属的用户改动：暂停提交，保留现场，不使用 reset/clean 解决。
- 探员模型/刀具游戏内无效但配置回读正确：按兼容性限制报告，不阻塞版本，不新增未经批准修复。

## 6. 最终报告格式

```text
结果：0.5.4 本地候选/签名候选/正式发布
默认值：bots / Low / mixed / less / 八项全开 / 507,508,515,519,525
自动化：npm、Rust、Clippy、Nade、diff-check 完整输出路径
候选 EXE：路径、size、SHA256、Authenticode
updater：.sig、验证结果、0.5.4 prod manifest
隔离安装：新装、30 秒、升级、退出、卸载、sentinel
真实 CS2：备份路径、console.log、less、节奏、八项、br_reroll、回读
兼容性：探员模型/刀具是否出现“写盘成功但游戏内无效果”
Git：commit、tag、push
线上：D1/R2/feed/API/GitHub 是否执行
回退：快照路径和剩余限制
```

在 `.sig`、0.5.4 manifest 和用户真实 CS2 报告都完成前，只能称为本地候选版。

