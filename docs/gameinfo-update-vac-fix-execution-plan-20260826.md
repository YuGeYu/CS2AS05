# CS2 `gameinfo.gi` 更新兼容修复：实际执行 AI 交接方案

## 0. 交接对象与目标

本文件给下一位“实际执行 AI”。两个 AI 不共享上下文；执行前必须重新读取本文件、当前工作树和实际 CS2 游戏树。

**目标：** 使用当前真实 CS2 官方文件 `D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\gameinfo.gi` 作为开发基线，开发阶段手工制作新的 `gameinfo.gi`、`backup/Online/gameinfo.gi`、`backup/WithBots/gameinfo.gi` 并静态打包进助手。用户端只安装和切换这三份已打包文件，不自行生成、编辑或适配 `gameinfo.gi`。

**成功定义：**

1. Steam 更新后的玩家官方 `gameinfo.gi` 不会被助手的旧内置文件覆盖。
2. BOT 模式只在当前官方基线之上生成最小差异；Online 模式能恢复同一份更新后的官方字节。
3. 任意失败、CS2 异常退出或助手关闭后，恢复路径不丢失原始基线。
4. Steam 默认入口启动后，经过 Steam 文件验证并由用户实测成功进入 VAC secure 在线对局。进入大厅不算通过。

**明确不做：** 不修改 Steam 客户端、不绕过 VAC、不把 `-allow_third_party_software` 用于在线、不删除用户未知插件、不将本地单元测试冒充实机通过。

## 1. 当前已确认事实

### 1.1 受影响代码

- `src-tauri/src/services/cs2.rs`
  - `REQUIRED_ZIP_ENTRIES` 要求 ZIP 含 `gameinfo.gi`、`backup/Online/gameinfo.gi`、`backup/WithBots/gameinfo.gi`。
  - `install_game_files_transactionally()` 会从 ZIP 解压并覆盖目标文件。
- `src-tauri/src/services/panel.rs`
  - `write_mode_at()` 从 `backup/Online/gameinfo.gi` 或 `backup/WithBots/gameinfo.gi` 整文件复制到活动 `gameinfo.gi`。
  - `launch_cs2_inner()` 的 Online 路径仍调用 `set_mode(root_path, "online")`，因此会复制可能过期的 `backup/Online/gameinfo.gi`。
  - BOT 路径会执行自动安装，然后切换模式。
- `docs/panel-v1.4.3-contract.md` 仍把“活动文件字节等于备份文件”写成契约；修复后必须更新为“基于当前官方基线生成/恢复”，不能继续宣称固定 ZIP 文件是权威。

### 1.2 实际游戏树证据（调查时）

游戏根目录：

`D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive`

- 活动文件：`game\csgo\gameinfo.gi`
- Online 备份：`game\csgo\backup\Online\gameinfo.gi`
- BOT 备份：`game\csgo\backup\WithBots\gameinfo.gi`
- 官方层级还存在：`game\csgo_core\gameinfo.gi`、`game\csgo_imported\gameinfo.gi`、`game\core\gameinfo.gi`。
- 调查时活动文件与 Online 备份哈希曾一致，但文件在 2026-08-26 仍被助手反复生成 `gameinfo.gi.backup-*`，说明模式切换在持续写入。
- 活动/备份内容来自旧的插件包时，会缺少 CS2 更新新增/调整的内容；不能用 `csgo_core\gameinfo.gi` 直接替代，它是 `CSGO Core`，不是 `CS2` 主 mod 的完整基线。

### 1.3 失败链条

1. Steam/CS2 更新把新的官方 `game\csgo\gameinfo.gi` 写入磁盘。
2. 助手安装/更新或切换 Online 时，从固定旧 ZIP/旧 `backup\Online\gameinfo.gi` 覆盖活动文件。
3. CS2 仍可读取并进入大厅，故启动表面正常。
4. 加入 VAC secure 在线服务器时，客户端文件内容与当前 Steam 版本不匹配，出现用户截图中的“文件无签名或签名无效”。
5. 关闭助手不会自动恢复已被覆盖的官方文件，所以随后从 Steam 启动仍失败。

## 2. 设计决策（执行 AI 必须遵守）

### 2.1 真相源与“必须携带文件”约束

- **官方基线只能来自玩家当前 `game\csgo\gameinfo.gi`，或 Steam 完整性验证后重新读回的该文件。**
- ZIP **必须**继续包含三个条目：`gameinfo.gi`、`backup/Online/gameinfo.gi`、`backup/WithBots/gameinfo.gi`。它们必须由上述开发机当前官方文件手工制作，是静态发布资产。
- 实际执行 AI 开始前必须重新读取上述开发基线并记录 SHA-256、size、mtime；禁止沿用旧 ZIP、旧 backup 或以 `csgo_core/gameinfo.gi` 冒充主 mod 基线。
- 上游逻辑是：Online 把 `backup/Online/gameinfo.gi` 复制到 `game/csgo/gameinfo.gi` 并删除 `-insecure`；BOT 把 `backup/WithBots/gameinfo.gi` 复制过去并加入 `-insecure`。本项目保留该三文件布局和切换语义，但每次资源更新都必须从当前官方基线重新生成。
- 用户端运行时不做动态基线、版本迁移或文件编辑；防护仅包括静态资源 manifest 校验、安装前备份、原子复制和失败回滚。未来 CS2 再更新时重新开发和重新打包。

### 2.2 开发阶段手工制作与静态资源

建议新增 Rust 内部服务（可放在 `services/panel.rs` 或拆为 `services/gameinfo.rs`）：

```text
capture_official_gameinfo(root) -> OfficialGameInfo { bytes, sha256, size, mtime, format_fingerprint }
copy_official_to_online(official_bytes) -> online_bytes
edit_official_to_bot(official_bytes) -> bot_bytes
write_static_package_assets()
restore_official_gameinfo(root, captured_state) -> Result
inspect_gameinfo_state(root) -> GameInfoState
```

开发阶段必须将当前官方文件原样复制为 ZIP 的 `gameinfo.gi` 和 `backup/Online/gameinfo.gi`，不得格式化或删除官方内容。再从同一份副本手工制作 `backup/WithBots/gameinfo.gi`。

`build_bot_gameinfo` 必须：

- 保留官方文件的所有未知字段、注释、换行和更新内容；
- 只在 `FileSystem.SearchPaths` 中加入当前插件所需的 BOT SearchPath，且仅加入允许的固定行（现有上游契约中的 `csgo/overrides/botprofile.vpk`、`csgo/addons/metamod`）；
- 幂等：对已经生成的 BOT 文件再次生成不能重复插入；
- 对结构不符合预期的文件返回稳定错误 `[GAMEINFO_FORMAT_UNSUPPORTED]`，不得猜测替换整文件；
- 写入前后做 SHA-256、行级/结构级检查，失败原子回滚。

可以新增一次性的开发辅助脚本，但它只能用于开发者制作静态资源，不能被编译进助手，不能在用户端调用，也不能根据用户机器的 `gameinfo.gi` 动态重建资源。最终包保留 manifest，供安装前静态自校验。

如果当前工程没有可靠的 GameInfo 结构解析器，第一阶段允许采用严格的块定位器，但必须限定在唯一 `FileSystem { SearchPaths { ... } }`，拒绝多处匹配、缺失匹配和嵌套异常；禁止全局字符串替换。

### 2.3 基线持久化

在助手可控目录保存状态，例如：

`game\csgo\cfg\cs2as05-gameinfo-state.json`

建议 schema：

```json
{
  "schema": 1,
  "officialSha256": "...",
  "officialSize": 12345,
  "officialBytesPath": ".../cs2as05-gameinfo-official.bin",
  "botSha256": "...",
  "capturedAt": "2026-08-26T...",
  "source": "live-gameinfo-after-steam-update"
}
```

实际原始字节应放在同目录的助手专用备份文件，使用临时文件 + flush/sync + 原子 rename；JSON 只记录摘要和路径。不得使用 `localStorage`。

规则：

- 当活动文件是 Online/官方状态且其哈希变化时，先把新官方字节捕获为新的基线，再生成 BOT 变体。
- 当活动文件已经是 BOT 状态时，不能把它当官方基线；必须使用已保存且哈希匹配的官方备份，否则返回 `[GAMEINFO_OFFICIAL_BASELINE_MISSING]`，提示用户先退出 CS2 并执行 Steam 文件验证。
- 绝不覆盖一个无法证明来源的 `backup/Online/gameinfo.gi`。

## 3. 代码修改边界与顺序

### 3.1 `src-tauri/src/services/cs2.rs`

1. 保留 `REQUIRED_ZIP_ENTRIES` 中三个 gameinfo 条目，继续强制校验其存在、非空、结构有效；新增资源 manifest 校验，确认三个文件来自同一个开发基线。
2. 修改 `install_game_files_transactionally()`：
   - 安装前先检测并捕获当前官方基线；
   - 安装前按现有事务机制备份用户活动文件，再写入 ZIP 的三个静态 gameinfo 条目；写入后回读并验证三个文件哈希与 manifest 一致；
   - 不读取用户当前文件来生成或改写 ZIP 资源，不做用户端动态版本比较；
   - 若静态资源自身哈希/结构不匹配，返回 `[GAMEINFO_ASSET_INVALID]`，不得写盘；
   - 用户机器上的 CS2 版本是否与本资源匹配属于发布/实机验收门槛，不由用户端自动修复。
3. 保留现有插件/配置文件原子安装和回滚逻辑，不把本次修复扩展到其他资源。

### 3.2 `src-tauri/src/services/panel.rs`

1. 替换 `write_mode_at()`：
   - `online`：复制随包携带并经静态 manifest 校验的 `backup/Online/gameinfo.gi` 到活动文件；
   - `bots`：复制随包携带并经静态 manifest 校验的 `backup/WithBots/gameinfo.gi` 到活动文件；
   - 每次复制前只校验资源包自身完整性，不读取用户文件来重建资源；
   - sidecar 记录三个携带文件的 SHA-256、开发基线 SHA-256、资源版本和生成日期。
2. 修改 `disk_mode()` / `snapshot_at()`：基于状态摘要和结构指纹判断模式，不再只比较“活动字节 == 旧备份字节”。未知/冲突状态显示 `needsRecovery` 或结构化错误。
3. 修改 `panel_transaction()` 的保护范围，纳入新的 gameinfo sidecar 和官方原始字节备份；回滚时验证摘要。
4. `launch_cs2_inner()` 的在线路径在 `set_mode` 前增加 `ensure_online_gameinfo_current()`：
   - CS2 未运行；
   - 读取当前/保存的官方基线；
   - 检查格式指纹、摘要和时间；
   - 失败则阻止 Steam spawn，返回 `[GAMEINFO_OFFICIAL_BASELINE_MISSING]` 或 `[GAMEINFO_RECOVERY_REQUIRED]`。
5. Online 启动参数只允许 `-applaunch 730` 加已有明确的非调试参数；不得为在线入口添加 `-insecure`、`-allow_third_party_software`、`-console`、`-condebug`。
6. BOT 路径继续使用 `-insecure`，但必须标记当前活动文件为 BOT 变体；BOT 退出后的恢复由下一次 Online 启动前强制执行，而非依赖助手是否仍在运行。

### 3.3 资源包/构建

- `src-tauri/resources/CS2BotImprover.zip` 必须继续携带三个 gameinfo 条目；由开发机当前官方基线手工制作后更新资源哈希、manifest 和相关测试夹具。
- 安装遍历必须明确处理这三个条目，不能因防止旧文件覆盖而跳过它们；防护点是资源新鲜度门禁和事务回滚。
- 不要修改与本次无关的插件 DLL。
- 在 `docs/panel-v1.4.3-contract.md` 增加“CS2 更新后 gameinfo 动态基线”章节，修正原有固定备份字节契约。

## 4. 测试要求

### 4.1 Rust 单元测试（必须）

新增/修改 `src-tauri/src/services/panel.rs` 或 `gameinfo.rs` 测试：

1. 使用两份不同版本的官方 fixture（旧版/新版 SearchPaths 不同），生成 BOT 后官方未知内容逐字保留。
2. BOT -> Online 恢复后字节等于捕获的官方基线，而不是 ZIP 旧文件。
3. Steam 更新模拟：改变官方活动文件哈希后，重新捕获新基线；旧 backup 不得覆盖新内容。
4. 活动文件是 BOT 且官方 sidecar 缺失/摘要不匹配时，返回 `[GAMEINFO_OFFICIAL_BASELINE_MISSING]`，不得写盘、不得启动 Steam。
5. 多个 SearchPaths 块、缺块、重复插件行、异常编码均拒绝并保持原文件不变。
6. 安装 ZIP 含旧 gameinfo 时，安装前后活动文件保持当前基线；其他允许资源仍按既有事务安装。
7. 所有写入失败都验证原始文件、sidecar 和备份摘要恢复。

### 4.2 前端/IPC 回归

- `PanelSnapshot` 增加或复用结构化 `gameinfo` 状态：`official|bots|unknown|recoveryRequired`、摘要、可写性。
- Online 按钮在 `recoveryRequired` 时禁用并显示可执行恢复提示；不得只显示“模式=Online”。
- 启动结果明确返回最终模式、gameinfo 摘要和参数，不能把 Steam 接受 spawn 当成 VAC 通过。
- 现有 `tests/` 中面板启动、安装、快照契约测试必须全部更新并通过。

### 4.3 真实游戏树验收（用户执行/实际执行 AI 记录）

在用户备份后的真实目录执行，至少记录：

1. Steam 更新后官方 `game/csgo/gameinfo.gi` 的 size/mtime/SHA-256。
2. 助手安装/启动 BOT 前后该文件与 sidecar 摘要。
3. BOT 模式能进入本地/离线 `-insecure` 场景。
4. 退出 CS2 后助手切 Online，活动文件哈希恢复为同一份更新后官方基线。
5. 关闭助手，从 Steam 默认入口启动；确认命令行无 `-insecure`、无 `-allow_third_party_software`、无 `-console/-condebug`。
6. Steam “验证游戏文件完整性”后再次记录哈希，并由用户尝试加入 VAC secure 在线对局。

**验收门槛：** 只有第 6 步成功，才能写“修复完成”；若只能证明大厅正常，报告必须写“未完成 VAC 实机验收”。

## 5. 迁移与故障处理

### 5.1 已经被旧版本覆盖的用户

不能自动猜测哪个 `gameinfo.gi.backup-*` 是官方。实际执行 AI 应：

1. 检查当前活动文件是否含 BOT SearchPath；
2. 检查 sidecar/助手专用官方原始备份是否存在且摘要匹配；
3. 若没有可信官方基线，提示用户关闭 CS2，在 Steam 执行文件验证；
4. 文件验证完成后重新捕获官方基线，再安装/启用助手；
5. 不从旧 ZIP 或随机时间戳 backup 恢复并宣称安全。

### 5.2 Steam 更新与助手并发

- 所有捕获、生成、恢复、安装操作必须先确认 `cs2.exe` 未运行。
- 使用同一进程内互斥锁保护 gameinfo 状态，避免两个启动请求交叉覆盖。
- Steam 更新后若检测到活动文件 mtime/size/hash 与 sidecar 不同，优先进入 `recoveryRequired`，不要静默重写。

## 6. 交付与报告格式

实际执行 AI 完成后必须在本项目新增执行报告，例如：

`docs/gameinfo-update-vac-fix-execution-report-20260826.md`

报告至少包含：

- 修改文件和未修改边界；
- 新旧 ZIP/资源行为差异；
- 单元测试和前端测试命令及结果；
- 真实游戏目录的脱敏路径、文件 size/mtime/SHA-256；
- BOT -> Online -> Steam 默认入口的逐步结果；
- Steam 验证和 VAC secure 对局结果；
- 若未完成实机步骤，明确列为用户待执行，不得用代码测试替代。

不得提交 Steam 账号、token、私钥、完整用户目录或未脱敏日志。

## 7. 停止条件

遇到以下任一情况立即停止写入和启动，保留现场并记录错误：

- 无法解析当前 `gameinfo.gi`；
- 官方基线缺失或摘要不匹配；
- 发现 CS2 仍在运行；
- 原子替换/回滚失败；
- Steam 文件验证仍报告失败；
- 修复后仍出现 VAC 弹窗但没有新的日志/哈希证据。

停止时不要删除旧备份、不要强制结束用户进程、不要把“在线按钮返回成功”当作最终完成。

## 8. 方案结论

本修复的本质是：**开发阶段使用指定真实官方文件手工制作新的静态资源；发布时继续携带三个 gameinfo 文件；用户端只做静态完整性校验、安装前备份、原子复制和模式切换，不自行修改或适配 gameinfo。** CS2 未来再次更新时，重新执行本开发方案并重新打包，不在用户端偷偷生成文件。

## 9. 对实际执行 AI 的强制澄清

不得把本方案实现成“用户端动态读取当前 `gameinfo.gi` 后生成 Online/BOT 文件”的系统；不得新增运行时 `build_bot_gameinfo`、`capture_official_gameinfo` 或自动迁移逻辑。这里的“根据官方基线编辑”只发生在开发者制作 `src-tauri/resources/CS2BotImprover.zip` 之前：使用 `D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\gameinfo.gi` 做输入，手工生成并审查三份静态文件，然后更新 ZIP 和 manifest。助手交付后严格按上游原有复制逻辑使用它们。
