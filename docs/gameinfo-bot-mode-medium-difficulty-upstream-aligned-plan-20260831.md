# 0.5.10 BOT 难度异常与 gameinfo.gi 上游对齐修复交接方案

日期：2026-08-31  
工作区：`E:\CS2AS05`  
交接对象：实际执行 AI  
状态：调查与方案；本轮未修改 gameinfo、botprofile、安装资源或版本号。

## 1. 用户现场与目标

用户在助手中选择“中等难度”进行 BOT 对局：

- 进入对局时聊天框确实显示 Medium；
- 道具能力仍然存在；
- 但 BOT 的反应、射击强度与预期不一致；
- 用户怀疑当前 `gameinfo.gi` 改法不正确。

本轮目标不是直接采用上游旧 release 文件，而是学习上游当前项目如何切换 `gameinfo.gi` 和难度资源，基于当前 Steam 官方 `gameinfo.gi` 重新生成正确资源：

1. Online 官方变体：不加载 Metamod、不加载 overrides botprofile，保持官方搜索路径；
2. BOT 基础变体：在官方基线基础上加入上游所需的 Metamod 与 overrides SearchPath；
3. Medium/Low/High：只切换对应 `botprofile.vpk`，不为难度复制或改写另一份 gameinfo；
4. 助手显示的 Medium、磁盘 active `botprofile.vpk`、CS2 实际加载的 VPK 必须是同一个版本和同一 SHA；
5. 反应/射击强度问题必须通过真实 BOT 行为和加载日志验收，不能以聊天框文案或文件存在作为成功证明。

本轮不新增版本号；即使工作树 `package.json` 当前显示为 `0.5.11`，执行 AI 必须先核对当前用户要求的 0.5.10 资源/构建边界，不得自行借机升级版本。

## 2. 已调查代码与上游证据

### 2.1 当前本地实现

- Panel 服务：`src-tauri/src/services/panel.rs`。
- 当前模式资源路径：
  - `game/csgo/gameinfo.gi`：active 文件；
  - `game/csgo/backup/Online/gameinfo.gi`：Online 备份；
  - `game/csgo/backup/WithBots/gameinfo.gi`：BOT 备份；
  - `game/csgo/overrides/{Low,Medium,High}/botprofile.vpk`：三档难度；
  - `game/csgo/overrides/botprofile.vpk`：当前 active 难度文件。
- `write_mode_at()` 按模式复制整份 Online/WithBots `gameinfo.gi`；`write_difficulty_at()` 按难度复制 VPK。
- `disk_mode()` 通过 active 文件与备份字节相等或包含 `csgo/addons/metamod` 判断 BOT；`disk_difficulty()` 通过 active VPK 与三档文件字节相等判断难度。
- 当前受控工作树资源 ZIP 包含上述三份 gameinfo，且 WithBots 文件比 Online 多出 SearchPath 行；必须重新审计其完整文本，而不能只看大小。
- 现有资源 sidecar/manifest 将 gameinfo 三份摘要作为固定资源校验；若重新生成，必须同步 `gameinfo.manifest.json`、sidecar 与安装测试。

### 2.2 上游当前主分支证据

截至 2026-08-31 查询：

- 仓库：`https://github.com/ed0ard/CS2-Bot-Improver`
- 当前主分支 commit：`5ef2235bf83fdd563384e30355eb547ce2c47166`（`Updated sigs and offsets on Linux`）。
- 当前源码树没有可直接复制的 `gameinfo.gi` 文件；上游 README 的安装说明是把包内容复制到 `game/csgo`，Panel 通过模式切换管理文件。
- 上游 README 明确：Medium 是“基于 HLTV stats 的 mixed difficulty（默认）”，Low/High 分别是低/极高难度；难度资源位于 `game/csgo/overrides`。
- 上游 README 的手工切换说明：BOT 模式使用带插件 SearchPath 的 `gameinfo.gi`；恢复 Online 时复制 `backup/Online/gameinfo.gi`，BOT 时复制 `backup/WithBots/gameinfo.gi`，并配合 `-insecure`。
- 上游主分支只作为结构和切换方法参考；不得下载旧 release ZIP、旧 `botprofile.vpk` 或旧 `gameinfo.gi` 直接覆盖当前资源。

### 2.3 高概率根因

当前 Medium 聊天提示只证明助手状态/配置显示为 Medium，不能证明 CS2 加载了目标 VPK。需要优先排查：

1. `gameinfo.gi` 的 SearchPath 顺序或语法被改坏，导致 `csgo/overrides/botprofile.vpk` 没有被 VPK 文件系统优先加载；
2. BOT 变体没有同时保持官方原始 SearchPaths，导致资源覆盖顺序改变，Metamod 或 BotImprover 依赖加载异常；
3. `write_difficulty_at()` 复制了 Medium 文件，但 active `overrides/botprofile.vpk` 在 CS2 启动前又被安装/升级流程覆盖；
4. Medium VPK 本身来自旧 release/错误构建，聊天文案正确但 aim/反应参数不是当前上游 Medium；
5. 游戏启动参数、active gameinfo、active VPK 和实际插件目录不是同一 CS2 根目录。

## 3. gameinfo.gi 生成契约

### 3.1 三份文件，不复制旧 release

执行 AI 必须从用户当前已验证的官方文件建立 `officialBase`，并生成：

```text
gameinfo.gi                         # active，最终由 set_mode 选择
backup/Online/gameinfo.gi           # officialBase 的字节副本
backup/WithBots/gameinfo.gi         # officialBase + 上游 BOT SearchPath 差异
```

`backup/Online/gameinfo.gi` 与官方基线必须逐字节一致；`gameinfo.gi` 在 Online 模式下也必须逐字节等于该文件。不得把当前已被修改的 active 文件直接当成官方基线。

### 3.2 WithBots 最小、可审计的差异

在官方基线 `FileSystem { SearchPaths { ... } }` 中，按上游方法加入两条 BOT 必需路径，并保持顺序稳定：

```text
Game    csgo/overrides/botprofile.vpk
Game    csgo/addons/metamod
```

推荐顺序是在 `Game_LowViolence csgo_lv` 后、官方 `Game csgo` 前，和当前受控 WithBots 结构一致。执行 AI 必须使用结构化 KeyValues/受控行级 patch，禁止用全局字符串替换、重复插入或凭位置猜测。

必须满足：

- 保留官方 `Game csgo`、`csgo_imported`、`csgo_core`、`core`、`Mod csgo` 等原有条目和所有其他官方块；
- 只新增一份 overrides VPK SearchPath 和一份 Metamod SearchPath；
- 不把 `csgo/addons/counterstrikesharp`、BotRandomizer、NadeSystem 等插件目录直接写入 gameinfo；这些由 Metamod/CounterStrikeSharp 的正常加载链处理；
- 不把 `-insecure`、难度值、聊天文案或自动换图配置写入 gameinfo；这些属于启动参数/active VPK/独立插件配置；
- Online 变体绝不能包含 `csgo/addons/metamod` 或 overrides botprofile SearchPath。

### 3.3 难度 VPK 契约

- Medium 的实际来源必须是当前上游源码/构建对应的 `overrides/Medium/botprofile.vpk`，不得使用 releases 中旧文件；Low/High 同理。
- `overrides/botprofile.vpk` 只是当前 active 副本，切换难度时原子复制目标档案；不能把 Medium 逻辑写进 gameinfo。
- 每档必须记录大小、SHA-256、来源 commit/构建时间和 VPK 可读性/条目摘要（如工具可读）；不能只记录文件名。
- 如果当前上游只发布 `.db`/其他中间格式而本项目使用 `.vpk`，必须确认本项目已有转换/打包脚本和格式契约；不能把 `.db` 改名为 `.vpk`。

## 4. 实际执行步骤

### P0：建立官方基线与当前资源对账

1. 退出 CS2，记录用户目录 `D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo` 的 active gameinfo、Steam 验证状态、文件大小和 SHA-256。
2. 对比当前 `backup/Online`、`backup/WithBots` 与官方基线，输出结构化差异：SearchPaths、块顺序、重复项、缺失项、非官方插入项。
3. 保存旧文件到受控 evidence 目录，不覆盖原始备份；任何重写前必须可恢复。
4. 从当前 Steam 官方基线生成 `backup/Online/gameinfo.gi`，再通过最小结构化 patch 生成 WithBots；同步生成 `gameinfo.manifest.json` 和 sidecar。

### P0：核对上游当前难度来源

1. 读取上游当前主分支/明确 commit 的 `overrides/Medium/botprofile.db`、Low、High 及相关构建脚本/文档，记录 SHA；不要读取 releases ZIP 作为最终资源。
2. 与本地 `overrides/{Low,Medium,High}/botprofile.vpk` 逐档比较；如果本地无法证明来源，重新按本项目既有构建流程生成 VPK。
3. 检查 Medium VPK 是否真的包含 mixed/HLTV 相关参数，而不是复制 Low 或 High；若格式不可直接审计，至少用上游构建输入、输出 hash 和受控 CS2 行为验证闭环。
4. 保持三档文件与 active 文件的原子切换和备份策略；升级时不覆盖用户选择之外的档案。

### P0：修正 Panel 模式/难度切换顺序

1. `set_mode("bots")`：先确认 CS2 已退出，再写 active WithBots gameinfo，回读 SHA/差异；不改 active difficulty。
2. `set_difficulty("Medium")`：确认 CS2 已退出，复制 Medium VPK 到 active 路径，回读 SHA；不改 gameinfo。
3. `launch_cs2_inner("bots")`：在启动 Steam 前最后一次回读 active gameinfo=WithBots、active VPK=Medium、sidecar/manifest 均匹配；启动参数保留 `-insecure`、`-console`、`-condebug` 等既有契约。
4. 启动后收集 CS2/Metamod/CounterStrikeSharp 日志，证明实际插件加载了当前目录和当前 VPK；聊天框 Medium 仅作辅助证据。
5. `set_mode("online")` 必须恢复 Online 官方字节，不得残留 overrides/metamod SearchPath；切换失败不能启动 Steam。

## 5. 自动化测试

新增/更新：

- `tests/gameinfo-upstream-contract.spec.ts`：Online 与官方基线逐字节一致；WithBots 只增加两条规范 SearchPath；无重复 Metamod/overrides；Online 不含 BOT 路径。
- `tests/gameinfo-difficulty-contract.spec.ts`：Low/Medium/High active 复制来源、hash 回读、切换不改变 gameinfo；Medium 不等于 Low/High。
- Rust `panel` 测试：模式与难度独立切换、CS2 运行时拒绝、安装/升级前后 active 文件和用户难度保持；启动前最终 snapshot 与返回 `LaunchResult` 一致。
- 资源 manifest 测试：ZIP 中三份 gameinfo 和三档 VPK 的 hash/大小与 manifest/sidecar 一致；当前构建不引用旧 release ZIP。
- 受控启动 fixture：命令行包含 `-insecure` 时 active gameinfo 必须是 WithBots；Online 不得包含该参数。

建议命令：

```powershell
Set-Location E:\CS2AS05
npm test -- --run tests/gameinfo-upstream-contract.spec.ts tests/gameinfo-difficulty-contract.spec.ts tests/installer-contract.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml panel cs2
```

若需要更新上游快照，必须把 commit SHA、下载 URL、文件 hash 和生成脚本写入 `third_party/CS2-Bot-Improver-v1.4.3/UPSTREAM.txt` 或新建当前版本 provenance 文件；不得只在执行报告中口头说明。

## 6. 真实 BOT 验收

真实验收由用户/执行 AI 在已确认的 BOT 环境进行，证据保存到：

```text
E:\CS2AS05\artifacts\gameinfo-medium-bot-fix-20260831\
```

1. 退出 CS2，确认 active `gameinfo.gi` 的 SHA 等于 WithBots，包含且只包含一份 overrides/metamod SearchPath；active VPK SHA 等于 Medium。
2. 从当前构建启动 BOT，保存启动参数、Panel 最终 snapshot、gameinfo/VPK hash、Metamod/CounterStrikeSharp 加载日志。
3. 进入 BOT 对局，确认聊天框 Medium 与启动前 active VPK 同时成立；再进行至少一个固定的反应/射击对比场景（同地图、同武器、同距离、同 BOT 数量、同设置），记录 BOT 首次发现目标、瞄准/开火延迟和连续射击行为。不能只凭主观“感觉强/弱”。
4. 对 Low、Medium、High 各启动一次，确认每档 active VPK hash 不同（如上游确实不同），行为强度顺序符合产品预期；道具能力不能因 gameinfo 修复消失。
5. 切回 Online，确认 active gameinfo 恢复官方基线、无 Metamod/overrides 路径；不要进入官方匹配进行测试。
6. 每次退出后检查 active gameinfo、VPK 没有被 Steam 或安装流程静默覆盖；发现覆盖立即回到资源/启动顺序调查。

## 7. 停止条件与交付报告

遇到以下任一情况停止，不得宣称 Medium 已修复：

- 直接使用上游旧 release 的 gameinfo/VPK，或无法提供当前来源 commit/hash；
- Online 与官方基线不一致，WithBots 出现重复/错误 SearchPath，或 Metamod/overrides 路径顺序无法解释；
- 聊天框显示 Medium 但 active VPK/hash、启动日志或真实行为不匹配；
- Low/Medium/High 切换互相覆盖 gameinfo、道具功能消失、BOT 插件未加载；
- 真实行为只有主观描述，没有固定场景、日志和前后 hash；
- 为让测试通过而放宽官方基线校验、删除 `-insecure`/Metamod 门禁、覆盖用户文件或降低资源版本检查。

执行 AI 最终需新增执行报告，包含：上游当前 commit/provenance、官方/Online/WithBots 三份 gameinfo 的结构化 diff 与 hash、Low/Medium/High VPK 来源与 hash、Panel 启动前 snapshot、CS2/Metamod/CounterStrikeSharp 加载证据、固定反应/射击场景记录、道具回归结果、Online 恢复证据、自动化测试和未完成的真实玩家验收项。若只能证明文件链路正确而不能证明 BOT 行为，状态必须写“资源修复候选/待真实 BOT 验收”。
