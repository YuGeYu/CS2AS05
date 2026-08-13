# CS2 Demo：运行中禁止提前解析紧急修复方案（2026-08-04）

> **联合续作：** 本 P0 解析闸门通过后，下一位执行 AI继续执行 demo-score-observer-kda-bot-viewer-execution-plan-20260804.md，使用同一最终 de_nuke Demo 修复比分/回合一致性、观战阵营、KDA Rating 和 BOT 回放颜色。两份方案必须按顺序完成，不能在半成品 report 上单独修 UI。

> 交接对象：下一位实际执行 AI。
>
> **优先级：P0，先修复再继续用户验收。** 用户真实 BOT 验收暴露出 `auto-20260804-1210-de_nuke-advent.dem` 在 CS2 尚未结束时被提前解析：`core` job 在 `12:10:18` 即标记 done，但文件随后继续写入到 `12:55:26`，最终 `demo_files.status=parsing`、`total_rounds=0`，且没有新的 queued job。当前必须建立不可绕过的运行中解析闸门，禁止再让用户重复完整对局。

## 1. 绝对不变量

对所有触发路径统一执行以下规则：

```text
CS2 进程存在
  => 不调用 demoparser
  => 不调用 wait_until_stable 后的 import/parse
  => 不创建或 claim core/spatial analysis job
  => 不把 Demo 标记为 parsing/done/error
  => 不弹出报告
```

这里的“所有路径”包括：

- notify watcher 的 `.dem` 文件事件；
- `scan_demo_roots` / UI“立即扫描”；
- `import_demo_file` / UI 手动导入；
- `retry_demo_parse`；
- `observe_assistant_launch` 的退出扫描；
- 应用启动恢复、迁移重算和任何后台 worker。

CS2 运行中允许只保留内存中的 pending candidate，或返回明确的 `DEMO_PARSE_BLOCKED_CS2_RUNNING`；禁止为了显示“解析中”而写入一个会误导用户的半成品状态。

## 2. 当前事实与根因

当前现场必须保留在 evidence，不删除或修正成假成功：

- Demo：`auto-20260804-1210-de_nuke-advent.dem`。
- `demo_files.status=parsing`、`total_rounds=0`、`kills=0`、`report_json` 约 1361 bytes。
- 同 Demo 的 core job：`12:10:17` 创建、`12:10:18` done、attempts=1。
- 文件 mtime 后移到 `12:55:26`，size 达到 `70,829,082 bytes`。
- 这证明 parser 读取的是写入中的半成品，而不是“完整 Demo 解析耗时过长”。

已确认的代码风险：

- [demo.rs](E:/CS2AS05/src-tauri/src/services/demo.rs:658) watcher 延迟 300 ms 后直接调用 `import_file`，且忽略结果。
- [demo.rs](E:/CS2AS05/src-tauri/src/services/demo.rs:779) `wait_until_stable` 遇到一次变化就返回错误，没有持续重试队列。
- [demo.rs](E:/CS2AS05/src-tauri/src/services/demo.rs:1099) 指纹变化时若同版本 job 已 done，`INSERT OR IGNORE` 不会重置或重排队。
- [demo.rs](E:/CS2AS05/src-tauri/src/services/demo.rs:2378) 助手启动观察只有结束后扫描，不能作为运行中保护。

## 3. 实现顺序

### 3.1 建立统一解析闸门

新增 `src-tauri/src/demo/parse_gate.rs`，或在现有 demo service 中抽出同等职责，禁止每条入口各自实现一套判断。

建议 API：

```rust
pub struct DemoParseGate(pub Mutex<ParseGateState>);

pub struct ParseGateState {
    pub pending: HashMap<PathBuf, PendingDemo>,
    pub parser_active: usize,
}

pub fn cs2_running() -> Result<bool, AppError>;
pub fn assert_parse_allowed() -> Result<(), AppError>;
pub fn defer_until_cs2_exit(path: PathBuf, reason: &'static str);
pub fn begin_parse(path: &Path) -> Result<ParsePermit, AppError>;
```

`begin_parse` 必须在真正打开 Demo、计算 checksum、创建/更新 `demo_files` 或插入 analysis job 之前执行，并再次调用 `cs2::check_cs2_process()`。返回 `DEMO_PARSE_BLOCKED_CS2_RUNNING` 时只进入 pending，不进入数据库解析状态。

不要把 process check 放在 UI 层；所有 Tauri command、watcher 和 worker 都必须经过 Rust gate。

### 3.2 watcher 改为“候选排队”，不直接解析

修改 `src-tauri/src/services/demo.rs::refresh_watcher`：

1. 文件事件只规范化路径、记录 generation 和 pending timestamp。
2. 不在 notify callback 的线程直接调用 `import_file`。
3. 由单一 coordinator 每 500-1000 ms 处理 pending：
   - CS2 运行中：保留 pending，记录 `deferred_cs2_running`，不访问 parser；
   - CS2 已退出：对每个候选做稳定检查；
   - size 和 mtime 连续至少 3 次一致，且最后写入年龄至少 3 秒，才允许进入 gate；
   - 稳定检查中任意变化，回到 pending 并重新计数，而不是失败一次就丢弃。
4. coordinator 在每个候选真正开始前和 parser 调用前都再次检查 CS2。
5. import 成功、失败、被阻断都保留结构化结果；禁止 `let _ = import_file(...)` 吞掉关键错误。

默认重试可持续到文件稳定或应用退出，不设 300 ms 一次性失败窗口。每个 pending 只允许一个 active attempt，避免同一路径并行解析。

### 3.3 所有手动入口也必须被闸门拦截

以下命令开始时先调用 `assert_parse_allowed()`：

- `scan_demo_roots`；
- `import_demo_file`；
- `retry_demo_parse`；
- migration/recovery 中的强制重算；
- coordinator 对 core/spatial job 的 claim。

CS2 运行中：

- watcher candidate：返回/记录 deferred，不写 `demo_files.status=parsing`；
- 手动“立即扫描”：返回 `DEMO_PARSE_BLOCKED_CS2_RUNNING`，UI 显示“CS2 运行中，已暂停 Demo 解析，退出后自动继续”；
- 手动导入/重试：同样拒绝，不把用户选择的文件写成 parsing。

退出后 coordinator 自动继续；用户不需要再次点击扫描。UI 提示可以增加，但必须使用现有蓝色 SaaS / Data-Dense Dashboard 语义 token、明确 loading/paused 状态、focus visible、reduced motion；不得用持续装饰动画掩盖等待。

### 3.4 修复指纹变化后的 job 生命周期

在 `import_file` 或 repository helper 中将“当前文件指纹”和“当前 job 指纹”作为同一事务处理：

1. 文件已有 done report，但 size/mtime/checksum 变化：旧 report 保持可读，新增/重置一个当前版本 core job 为 queued。
2. 不允许 `INSERT OR IGNORE` 静默复用已 done job。
3. 更新 `demo_files` 为 `queued` 或专用 `deferred`，但只有在 CS2 已退出且稳定 gate 通过后才写 `parsing`。
4. job claim 成功后保存 checksum/size/mtime snapshot；parser 完成提交前再次比较文件指纹。
5. parser 完成时若文件指纹已变化，丢弃本次 report 写入，回到 queued/deferred，重新等待稳定；绝不把早期结果标记 done。
6. 同 checksum + parser/schema/metrics 只允许一个 active core job；旧错误/半成品不覆盖新稳定结果。

如果无法新增 job fingerprint 字段，至少使用 `demo_files.checksum + size_bytes + mtime_ms` 与 job 的 `log_tail`/结构化字段保存 snapshot，并测试重复事件幂等。

### 3.5 处理应用重启和 CS2 生命周期

- 应用启动时若 CS2 正在运行：不恢复、claim 或重算任何 Demo job；仅建立 watcher/coordinator pending。
- CS2 从 running 变为 stopped 后，coordinator 才开始稳定检查；不能仅依赖一次 `observe_assistant_launch`。
- 助手启动 BOT 前可记录 session baseline，但 session watcher 不能绕过统一 parse gate。
- 外部手动启动 CS2 也必须被 gate 识别，不能只保护助手启动的 CS2。
- 如果 CS2 进程在解析开始后重新出现，下一次解析提交前必须再次拒绝/废弃结果；至少在 parser job claim、parser open、提交事务三个边界检查。

无法完全阻止外部进程在 parser 已开始后启动时，必须在报告中明确竞态边界，并优先让助手自身 `launch_cs2` 在存在 active parser 时等待或拒绝启动；不要宣称“绝对”却只做一次进程检查。

## 4. UI 状态契约

用户必须能区分：

| 状态 | 允许的数据库状态 | UI 文案 | 是否解析 |
| --- | --- | --- | --- |
| CS2 运行中 | 不新增 parsing job | `CS2 运行中，退出后自动解析` | 否 |
| 等待稳定 | queued/deferred | `等待 Demo 写入完成` | 否 |
| 已稳定并开始 | parsing + active job | `正在解析` | 是 |
| 完成 | done | `完成` | 是 |
| 错误 | error | `解析失败` + error code | 尝试已结束 |

禁止把 CS2 运行中或文件仍在增长显示为“正在解析”。“正在解析”必须有一个 active job，并且该 job 的输入 fingerprint 已经稳定。

## 5. 自动化测试（先写失败测试）

### 5.1 必须覆盖的 Rust 测试

1. `cs2_running_blocks_import_without_db_mutation`：CS2=true 时调用所有 import/scan/retry 入口，确认没有 `demo_files`、`analysis_jobs`、report 写入。
2. `watcher_event_while_cs2_running_is_deferred`：事件到达不调用 parser，不创建 job。
3. `candidate_parses_only_after_cs2_exit_and_three_stable_samples`：文件增长期间不解析，CS2=false 且连续稳定后才解析。
4. `stable_check_retries_after_size_change`：一次变化不会丢掉候选。
5. `changed_fingerprint_requeues_done_job`：旧 job done + 新 size/mtime/checksum 必须生成新的 queued attempt。
6. `parser_commit_rejects_changed_fingerprint`：解析过程中 Demo 继续增长，不能提交 done report。
7. `duplicate_watcher_events_are_idempotent`：同路径多事件只产生一个 active job。
8. `startup_with_cs2_running_does_not_claim_jobs`：启动恢复在 CS2=true 时不 claim。
9. `external_cs2_transition_is_observed`：不是助手启动的 CS2 也能阻断。
10. `parse_errors_are_not_silently_discarded`：watcher 错误写入结构化日志/状态，不能只 `let _ =`。

### 5.2 前端测试

- CS2 运行中显示 paused/deferred 文案，不显示 parsing 进度。
- 手动扫描/导入被拒绝时显示明确错误，按钮不进入永久 loading。
- CS2 退出后自动刷新到 queued/parsing/done；不要求手动点击扫描。
- old report 保持可读，new fingerprint 完成后原子切换。

## 6. 现场 Demo 的恢复方案

不要直接修改用户 DB 来“快速变 done”。执行 AI实现修复后，使用现有 Demo 文件重新触发稳定导入：

1. 关闭当前助手和 Vite，确认没有 `ai_pc_fac`/`cs2.exe` 进程。
2. 备份并 hash 当前 SQLite；保留原有半成品记录和 evidence。
3. 启动修复后的候选，确保 CS2=false。
4. 让 coordinator 发现 `auto-20260804-1210-de_nuke-advent.dem` 的最终 fingerprint。
5. 旧的半成品 report 不作为最终结果；新 job 必须从最终稳定文件重新解析。
6. 验证 `demo_files.status=done`、`total_rounds>=5`、玩家和事件不为空；必要时空间 job 另行 queued，不阻塞 core report。
7. 保存新的 DB/report evidence，不覆盖本次失败现场。

不要要求用户重新玩一把来验证这个修复，除非执行 AI无法利用已有最终 `.dem` 证明。若必须复测，最多再请求用户一次针对性短对局。

## 7. 验证顺序与 Gate

执行 AI按以下顺序执行：

```powershell
npm test -- --pool=threads --maxWorkers=1
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo test --manifest-path .\src-tauri\Cargo.toml --lib
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings
cargo build --manifest-path .\src-tauri\Cargo.toml
git diff --check
```

然后使用修复后的最终 hash 做一次真实 Tauri + 现有 `de_nuke` 恢复验证：

- CS2=true 时 watcher/scan/import/retry 全部不产生 parser/job/状态写入。
- CS2=false 但 Demo 继续增长时保持 deferred/queued，不解析。
- 文件最终稳定后只解析一次最终 fingerprint。
- core report 变为 done，回合/玩家/K-D-A不为半成品空值。
- 旧报告在新报告成功前仍可读取。
- 没有重复 job、永久 parsing、静默错误或旧报告覆盖新结果。

Gate 未全部通过前，结论只能是：

```text
Demo watcher P0 修复未完成，不请求用户再次 BOT 验收。
```

通过后才更新用户验收方案，允许用户做一次针对性复测；不得把“旧 Demo 手动扫描成功”冒充 watcher 通过。

## 8. 禁止事项

- 不通过延长等待时间掩盖状态死锁。
- 不在 UI 中把 `parsing` 改名为“等待中”掩盖数据库错误。
- 不删除半成品 Demo、旧 report、DB 或 evidence。
- 不只修 watcher 而遗漏 scan/import/retry/worker claim。
- 不只检查一次 CS2 进程就声称无竞态。
- 不用用户重新玩一把替代自动化回归。
- 不 commit、push、release、deploy 或构建 installer，除非另有明确授权。
