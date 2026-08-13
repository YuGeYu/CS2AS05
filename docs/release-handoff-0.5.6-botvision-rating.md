# 0.5.6 发布交接方案：BotVision 0.2.2 + 多因子 Rating

> 给下一位实际执行 AI 的独立交接文档。本文基于 2026-08-05 的 `E:\CS2AS05` 工作树调查；执行时以实时文件、构建输出和线上回读为准。不要回退当前工作树已有改动，不要提交 Demo、数据库、日志、`workspace/` 证据或任何密钥。

## 1. 当前基线与已确认事实

- 工作目录：`E:\CS2AS05`，当前分支 `main`，HEAD 为 `8552b55`（已有大量未提交的 0.5.6 Demo/KDA 改动）。保留这些改动并在其上收敛。
- `package.json`、`src-tauri/tauri.conf.json` 当前版本已是 `0.5.6`；发布目标为 Windows x64 NSIS 和 updater manifest。
- 当前内置资源是 `src-tauri/resources/CS2BotImprover.zip`，SHA-256 `18A22B161858B698198F0DAFB88433F94DB459B05EFC45C346AA0C8C64329A8F`。它包含 CounterStrikeSharp、Panel v1.4.3、NadeSystem 和 `CS2AS05.plugin.json` marker。
- 用户提供包 `E:\dow\BotVision-Windows-0.2.2.zip` SHA-256：`40B596D34BF336D9E59E663DAC2F94BD7C61D951C56E421EF66B5190B8787290`，内容仅为：
  - `addons/BotVision/gamedata.json`（3619 bytes）
  - `addons/BotVision/bin/win64/BotVision.dll`（1501184 bytes）
  - `addons/metamod/BotVision.vdf`（95 bytes）
- BotVision 是原生 MetaMod 二进制插件，不能覆盖或替换现有 `addons/counterstrikesharp`；目标是同包并行安装。必须先检查其 `gamedata.json` 是否要求特定 CS2/MetaMod 版本，检查 DLL 是否有 Authenticode 签名并记录结果；未签名不是自动阻塞，但要在 provenance 中如实注明。
- 当前 `src-tauri/src/services/simple_rating.rs` 已切换为 `simple-rating-v1`；最终采用 KDA 主导的多因子公式，主比率为 `(K + A/5) / max(D,1)`，避免退回只看 K/D/A。
- 发布密钥目录已确认存在：`C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key`、`updater-password.dpapi`、`updater.key.pub`。不要打印、复制、提交或写入本文密钥内容。

## 2. 交付目标与停止条件

### 必须交付

1. 内置 `CS2BotImprover.zip` 同时包含现有 BOT 组件和 BotVision 0.2.2 三个文件，安装事务、版本检查、payload 摘要和回滚仍有效。
2. 报告中的“简易 Rating”采用 `simple-rating-v1` 多因子公式，接近用户要求的 KDA 思路但同时使用伤害、生存代理和助攻；缺字段不伪造 0。
3. 前后端 DTO、缓存 schema、UI、测试、NOTICE/provenance 和 0.5.6 发布元数据一致。
4. 通过自动测试、真实 Demo 战报窗口、安装/覆盖升级、签名 updater manifest 检查；真实 CS2/BOT 效果由用户最终确认，不能把自动化结果写成真实游戏验收。

### 立即停止并报告用户

- BotVision 与当前 MetaMod/CS2 运行时冲突、加载崩溃、gamedata 版本不兼容，或无法在隔离测试中确认可安全共存。
- 发布密钥不可读、密码解密失败、updater 公钥不匹配、安装包签名或 manifest 不对应同一文件。
- 工作树出现非本次任务的覆盖风险，或需要删除/重置已有用户改动。
- 真实发布平台已有不可逆的 0.5.6 资产/客户端状态而需要强行覆盖；先保留现场，改发增量版本或请用户决定。

## 3. BotVision 合入方案

### 3.1 资源处理

1. 先将源 ZIP 复制到 `workspace/release-evidence/<timestamp>/incoming/`（证据目录不提交），记录大小、mtime、SHA-256 和压缩包条目。
2. 以临时目录解压，拒绝绝对路径、`..`、重复条目和符号链接；确认只出现上面三个文件。
3. 将三个条目合并进现有 `CS2BotImprover.zip`，保留原有目录和文件，不覆盖同名文件。建议新增脚本 `scripts/merge-botvision.ps1`，使用临时 ZIP + 原子替换；失败时保留原 ZIP 和备份，不直接破坏资源。
4. 资源清单/marker 需要从“仅 NadeSystem payload”扩展为组件清单，例如：

```json
{
  "schema": 2,
  "product": "cs2-bot-improver",
  "pluginId": "cs2as05-custom-package",
  "version": "0.5.6",
  "components": [
    {"id":"cs2as05-custom-package","source":"CS2-Bot-Improver-v1.4.3"},
    {"id":"botvision","version":"0.2.2","source":"XBribo/CS2-Bot-Vision","sourceSha256":"40B596...8787290"}
  ],
  "payloadSha256": "<按现有算法重新计算>",
  "payloadEntries": ["<现有 NadeSystem 条目>","addons/BotVision/gamedata.json","addons/BotVision/bin/win64/BotVision.dll","addons/metamod/BotVision.vdf"]
}
```

实际字段命名应沿用现有 Rust serde 风格；不得把截断 hash 写入正式 marker。旧 schema=1 marker 可识别为旧版本并触发一次覆盖安装/迁移，不得误报为当前有效。

### 3.2 安装与校验改动

- 更新 `src-tauri/src/services/cs2.rs` 的常量、`PluginMarker`、必需 ZIP 条目和 payload digest。payload digest 必须覆盖 BotVision 三项，并按规范化路径排序，避免 ZIP 条目顺序导致摘要漂移。
- 安装前 staging 解压必须验证 BotVision 三项存在；安装后从实际 `game/csgo` 读取并验证相同摘要。保留现有事务备份、回滚和“退出 CS2 才能安装”门禁。
- 版本检查仍以应用版本 `0.5.6` 为包版本；BotVision 自身版本单独记录为 `0.2.2`，不能把它误当成应用版本。
- `NOTICE.md` 新增 XBribo/CS2-Bot-Vision release v0.2.2 的来源、下载 URL、源 ZIP SHA-256、三项路径、许可证/署名核查结果和“原生 MetaMod 并行组件”说明。若上游许可证未随 ZIP 提供，先从 GitHub tag 仓库读取许可证并保留链接；未核实前不得宣称已满足再分发条件。
- README 的内置包说明、安装诊断和版本回读文案同步更新，明确显示 `CS2AS05 0.5.6 + BotVision 0.2.2`。

### 3.3 测试门禁

- Rust：marker schema 2 接受当前完整 payload；缺任一 BotVision 文件、篡改 DLL/gamedata/VDF、旧 schema=1、路径穿越均失败；旧合法包升级决策为 Install。
- PowerShell/Node：对合并 ZIP 做条目白名单、重复条目、大小/hash 快照测试。
- Windows 真实验收：在 `-insecure` 本地 BOT 启动中回读 `addons/metamod/BotVision.vdf` 和 BotVision 加载日志，确认不影响 CounterStrikeSharp/NadeSystem；异常时抓取日志和进程退出码，不上传二进制日志。

## 4. simple-rating-v1 公式与实现契约

输入必须同时存在：`K=kills`、`D=deaths`、`A=assists`、`DMG=damage`、`R=completedRounds`，且 `R>0`。缺一项返回 `rating=null`、玩家状态 `unavailable`；不以 0 替代缺失。

```text
KPR = K / R       DPR = D / R       APR = A / R       ADR = DMG / R
kill       = clamp((K + A / 5) / max(D, 1), 0, 3.0)
damage     = clamp(ADR / 82.0, 0, 2.5)
survival   = clamp((1 - clamp(DPR, 0, 1)) / 0.68, 0, 2.5)
assist     = clamp(APR / 0.20, 0, 2.5)
rating     = clamp(0.80*kill + 0.10*damage + 0.05*survival + 0.05*assist, 0, 3)
```

Rust 使用 `f64`，component 和结果四舍五入 4 位；UI 结果 2 位、ADR 1 位。DTO 为 `modelVersion=simple-rating-v1`、`killComponent`、`damageComponent`、`survivalComponent`、`assistComponent`、`rating`。删除或停止序列化旧 `openRating/economy/multi/swing` component，KAST/Swing/经济仍可作为 `Option` 高级字段但本版不参与评分。

固定 Rust 向量必须断言：

- average `K=14,D=6,A=4,DMG=1640,R=20` => `kill=2.4667`、`rating=2.1748`；components `2.4667,1,1.0294,1`。
- zero `0,20,0,0,20` => `0.0000`。
- dominant `40,0,20,5000,20` => components `3.0,2.5,1.4706,2.5`、`rating=2.8485`。
- `R=0` 或任一输入缺失 => `None`；极端 u32 输入结果仍在 `[0,3]`。

解析顺序必须是：先聚合 `round_end` 得到 `completed_rounds`，再 `build_scoreboard` 和 calculate。`round_officially_ended` 不得重复计数。controller totals 优先，event aggregate 回退；spectator/unknown 不参与 complete 判定。报告状态为 `complete/partial/unavailable`。

缓存/数据库：当前报告 schema 沿用工作树已有 `REPORT_SCHEMA_VERSION=6`，`METRICS_VERSION=simple-rating-v1`；cache hit 同时检查 report schema、parser adapter、metrics version，旧 `openrating-demo-v1` 必须重算而非复用。

## 5. UI 与文件级执行清单

- `src-tauri/src/services/simple_rating.rs`：替换算法、版本常量、单元测试。
- `src-tauri/src/models/demo.rs`、`src/types/demo.ts`、`src/services/tauri/demo.ts`：同步 DTO/serde/status。
- `src-tauri/src/services/demo.rs`：回合先建、再算 Rating；更新 schema/cache。
- `src/components/scoreboard/PostMatchScoreboard.vue`：表头使用“简易 Rating”，显示数值 + 高/中/低/不可用文字；MVP/排序读 `rating.rating`；“报告回合”；footer 写 `simple-rating-v1 · 非 HLTV 官方模型`。
- `src/styles/main.css` 及相关测试：保持现有数据密集灰色科技风，确保 860x600 不溢出，不新增营销卡片。
- `src-tauri/src/services/cs2.rs`、`scripts/generate-plugin-manifest.ps1`、`src-tauri/resources/CS2BotImprover.zip`：BotVision 合包、marker、摘要和安装门禁。
- `NOTICE.md`、`README.md`：来源、许可证、版本和安装说明。
- 新增/更新 `tests/*rating*`、`tests/bot-plugin-gate.spec.ts`、必要 Rust 集成测试。

## 6. 自动化与真实验收顺序

在项目根目录依次运行并保存 exit code、stdout/stderr、耗时：

```powershell
npm run workspace:check
npm run typecheck
npm run lint
npm test
npm run build:web
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo test --manifest-path .\src-tauri\Cargo.toml
git diff --check
```

使用已有真实 Demo（若路径存在）执行 ignored scoreboard 测试，要求 10 名 T/CT 玩家、回合数 >0、Rating 0..3；证据只写入 `workspace/release-evidence/`。随后由用户确认真实 CS2 BOT：BotVision 加载、至少 3 回合、正常退出后 watcher 解析新 Demo 并弹出独立战报窗口；自动测试不能代替这一项。

## 7. 发布所需文件与安全构建

### 源码/资源必须进入发布提交

`src-tauri/resources/CS2BotImprover.zip`（合入 BotVision 后的新 hash）、`src-tauri/src/services/cs2.rs`、Rating/DTO/parser/UI/测试、`NOTICE.md`、`README.md`、本方案、必要脚本和 `package.json`/`tauri.conf.json` 的 0.5.6 自有版本字段。不要提交 `target/`、`dist-release/` 产物、Demo、SQLite、日志、证据目录、`*.dpapi`、`updater.key`。

### 构建输入（密钥只从本机读取）

```powershell
$keyDir = 'C:\Users\GOPtZ\Documents\CS2AS05-release-keys'
$secure = Get-Content (Join-Path $keyDir 'updater-password.dpapi') -Raw | ConvertTo-SecureString
$credential = [PSCredential]::new('tauri-updater', $secure)
$env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content (Join-Path $keyDir 'updater.key') -Raw).Trim()
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $credential.GetNetworkCredential().Password
try { npm run bundle:desktop; $env:RELEASE_CHANNEL='prod'; npm run release:manifest }
finally { Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY,Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD,Env:RELEASE_CHANNEL -ErrorAction SilentlyContinue }
```

不得把 key 内容输出到终端或证据。最终预期文件：

- `src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.6_x64-setup.exe`
- 同路径 `.exe.sig`
- `dist-release/cs2-bot-improver/updater-prod.json`

为三者记录 size、mtime、SHA-256；manifest 的 version/channel/projectId/path/size/hash/signature 必须指向本次同一 EXE。`Get-AuthenticodeSignature` 的 Authenticode 状态与 Tauri `.sig` 分开记录，不能互相冒充。

## 8. 发布前最终回读

```powershell
git status --short
git rev-parse HEAD
git diff --check
gh release view v0.5.6 --repo YuGeYu/CS2AS05 --json url,isDraft,publishedAt,assets
```

首次发布前确认没有同名线上资产；若已有下载/更新客户端，不覆盖线上版本，停止并报告。发布顺序：审阅 diff -> 本地安装 -> 覆盖 0.5.5（配置和历史 Demo 保留）-> updater 被动安装/重启 -> GitHub Release/更新服务 -> 重新回读公开资产 hash 和 manifest。不要在未完成真实 BOT 验收时宣称“已发布可用”。

## 9. 交付给下一 AI 的结果格式

最终报告必须分开写：

1. 已实现：代码、资源、marker、公式、测试文件和版本字段。
2. 自动化证据：每条命令 exit code、关键摘要、安装包 hash、manifest 校验。
3. 真实机器证据：BotVision 加载、BOT 对局、watcher/战报窗口、安装升级（由用户确认的部分单独标注）。
4. 未验证/阻塞：签名、Authenticode、线上回读、许可证或兼容性疑点，不能用“应该”替代证据。
