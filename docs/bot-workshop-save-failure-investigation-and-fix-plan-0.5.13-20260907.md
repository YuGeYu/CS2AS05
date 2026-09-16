# 人机强度工坊保存失败调查与修复计划（0.5.13）

日期：2026-09-07  
关联版本：`0.5.11`、`0.5.12`  
关联错误：`BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID`、`BOT_WORKSHOP_EXTRACT_FAILED`

## 1. 当前判断

这不是一个已经被单一根因解释的问题，至少存在两条失败链路：

1. **v0.5.11 提取目标争用/写入失败**：反馈路径是固定的
   `workspace\\open-...\\botprofile.db`。旧实现把 VPKEdit 退出码 1、输出文件无法创建、权限/安全软件拦截等情况统一包装成
   `BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID`，因此仅凭这条错误不能证明工具缺失或 VPK 损坏。
2. **编辑内容导致重打包或回读失败**：玩家反馈“改错了，之后就不报错”，与当前 `save()` 将编辑器全文直接写入
   `candidate.db` 再交给 VPKEdit 的行为一致。大小写本身不应被假定为错误，但错误的 key、引号、括号、编码、换行或输入法混入的不可见字符都可能使 DB 无法被上游工具接受。

v0.5.12 已加入独立提取目录、提取互斥、有限重试、写入探测和异步 Tauri command；这些改动降低了旧路径争用和窗口卡顿风险，但尚未证明能覆盖“非法编辑内容”或所有用户环境的文件写入拦截。不能把本地构建与自动化测试当作受影响玩家机器上的修复证据。

## 2. 证据分层

### 已确认

- v0.5.11 工单的错误目标路径与旧 `open()`/`extract_db()` 固定输出路径逐字一致。
- 错误发生在 VPKEdit 将 `botprofile.db` 写入输出目标时；错误文本没有证明原始 BOT 数据损坏。
- v0.5.12 当前代码使用 `extract-<时间戳>-<尝试号>` 新目录，并在提取前执行写入探测与最多 3 次重试。
- 当前保存流程会先生成候选 VPK，再提取回读并逐字节比较；比较失败不会覆盖原 VPK，但 VPKEdit 退出码仍主要显示为工具/保存类错误。
- 玩家“改错后恢复正常”的口述是线索，不是可复现证据；不能据此断言具体是大小写或中英输入法。

### 必须从现场补齐

- 实际安装器 SHA-256、应用内版本和 `src-tauri` 资源中的 VPKEdit 版本/SHA。
- 失败操作是“打开内置/自定义档案”还是“保存编辑后的档案”。
- 输入 VPK 的路径、大小、SHA-256；输出目录是否已存在、是否通过写入探测、剩余空间。
- VPKEdit 完整阶段摘要：命令阶段、退出码、stdout/stderr、尝试次数、Win32 错误类别。
- 编辑文本的 UTF-8 SHA-256、字节长度、行尾/BOM/控制字符摘要；不得收集完整玩家档案到公共日志。
- 失败后候选 VPK、原 VPK、`botprofile.db` 的存在性和 SHA；确认是否发生过覆盖或回滚。
- Defender/受控文件夹/第三方安全软件事件，以及是否有多个 VPKEdit 进程或快速重复点击。

## 3. 修复目标

### A. 把错误分类做窄

在 `src-tauri/src/services/bot_difficulty.rs` 中把以下阶段分开：

- 工具缺失、EXE SHA 不匹配、`--help` 失败：`BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID`。
- workspace 创建/写入探测失败：`BOT_WORKSHOP_WORKSPACE_UNWRITABLE`。
- VPK 输入打开或条目读取失败：`BOT_WORKSHOP_VPK_OPEN_FAILED` / `BOT_WORKSHOP_ENTRY_MANIFEST_MISMATCH`。
- `--extract` 退出码非零且输入有效：`BOT_WORKSHOP_EXTRACT_FAILED`，附带阶段、退出码、尝试次数和截断后的 stderr。
- 保存时 `--remove-file/--add-file` 失败：`BOT_WORKSHOP_REPACK_FAILED`，不要伪装成依赖无效。
- 回读字节与候选文本不一致：`BOT_WORKSHOP_VPK_ROUNDTRIP_MISMATCH`。
- 原子替换/备份/回滚失败：`BOT_WORKSHOP_SAVE_ROLLBACK`，返回旧文件和候选文件摘要。

日志只记录路径类别、文件名、大小、哈希和 Win32 错误码；不记录完整 DB、token 或用户目录之外的敏感内容。

### B. 约束编辑内容，但保留未知内容

保存前增加“语法安全检查”，不按大小写做武断拒绝：

- 验证 UTF-8、BOM/行尾策略和最大长度；显式拒绝 NUL、异常控制字符和明显的输入法不可见字符。
- 按 `botprofile.db` 的实际文本结构检查括号/引号/section 配对、key/value 形态和重复 key；未知 section/key 原样保留并只发出警告。
- 对受控字段使用 `textTokenOnly` 或带范围/步长的字段字典，禁止把未经上游或实机证明的数值规则写死。
- 前端显示具体行号和可操作提示；保存按钮在检查失败时不启动 VPKEdit。
- 后端必须重复校验，不能依赖前端校验结果。

### C. 让提取与重打包具备可恢复性

- 每次操作使用随机、不可碰撞的工作目录；目录创建后先写入探测，并在调用 CLI 前确认目标文件不存在。
- 退出码 1 只对“可重试的输出写入失败”做一次短退避重试；输入解析/条目不存在不重复执行。
- 保存始终写 `candidate.db`、`candidate.vpk`，完成 round-trip 后才备份并原子替换原档案；失败候选保留到受控诊断目录并标记为无效。
- 启动时清理仅限应用自己的 `workspace\\open-*`、`extract-*`、`verify-*` 等临时目录，不触碰 profiles、backups、active VPK。
- 同一进程内统一使用工坊互斥；快速重复打开/保存时取消旧请求，旧结果禁止回写当前编辑器。

## 4. 实施顺序

1. **复现夹具**：用四份内置 VPK 做无修改 extract、合法小改、非法 token、中文/中英输入法混入、重复 key、异常换行和目标文件预存在场景，保存完整阶段证据。
2. **后端协议**：先完成错误码拆分、诊断摘要、语法安全检查和 candidate/round-trip 状态机；补 Rust 单测。
3. **前端反馈**：把错误码映射为“内容格式”“目录权限/安全软件”“工具异常”“需要重新打开”四类中文提示，显示行号/重试次数/下一步，不展示冗长 stderr。
4. **并发与取消**：核对 `BotDifficultyWorkbench.vue` 的请求序列保护，确保打开、切换、保存、应用之间不会交叉覆盖状态。
5. **回归构建**：执行 `npm run typecheck`、定向 Vitest、`cargo test --manifest-path .\\src-tauri\\Cargo.toml bot_difficulty`、Web 构建和桌面构建。
6. **真实验收**：由用户在真实 CS2 BOT 环境验证 Low/Medium/High 打开、创建自定义、合法保存、非法输入提示、回读、应用、删除/恢复；至少在一台曾失败的玩家机器上完成一次打开与保存回读。

## 5. 测试与发布门禁

- 四份内置 VPK 无修改 round-trip 全部通过，`botprofile.db` 唯一且 bytes 保持一致。
- 合法编辑保存后：候选 DB/VPK 回读 bytes 一致，原 VPK 只在校验完成后替换，备份可读。
- 非法编辑保存前被拦截，返回内容错误码，不启动 VPKEdit，不改变原档案。
- 目标文件预存在、快速重复操作和模拟权限失败时，分别返回窄错误码并完成有限重试/回滚。
- 打开、切换、保存、应用期间窗口保持响应，旧请求结果不能覆盖新档案。
- 真实受影响机器通过前，不把“10% 玩家问题已修复”写入更新日志或发布说明；0.5.12 已发布资产不覆盖，修复应进入新版本（计划为 0.5.13）。

## 6. 停止条件

- 无法获得受影响玩家的实际 v0.5.12 版本与失败阶段证据时，只能完成本地夹具和错误分类，不能认定现场根因。
- VPKEdit 对合法上游 DB 仍无法稳定 round-trip 时，停止继续扩大编辑器字段范围，先回退到只读/复制档案流程。
- 任一失败路径可能覆盖原 VPK、删除用户备份或绕过 CS2 运行中保护时，停止发布并先修复事务边界。

## 7. 本轮已实施（2026-09-07）

- 保存前加入后端 `botprofile.db` 结构安全检查，拦截 NUL/不可见控制字符、未闭合引号、空 key/value、多余 `End` 和未闭合区块；大小写与未知 key 不因规则不明而被拒绝。
- 保存错误拆分为 `BOT_WORKSHOP_DB_INVALID`、`BOT_WORKSHOP_REPACK_FAILED` 和 `BOT_WORKSHOP_VPK_ROUNDTRIP_MISMATCH`，前端增加对应中文提示。
- 修正候选 VPK 生成方式：不再复制后原地修改 `candidate.vpk`，改为对只读源 VPK 使用 VPKEdit `--output` 生成独立候选输出，再选取生成的主 VPK 执行回读校验。
- 本地用 Medium 档案实际修改 `ReactionTime`/`AttackDelay` 完成 VPKEdit 重打包与提取回读；Rust 定向单测 3/3、工坊契约测试 5/5、`cargo check` 均通过。
- 真实受影响玩家设备尚未回读验证；0.5.12 已发布资产未覆盖，后续需以新版本构建交付。
