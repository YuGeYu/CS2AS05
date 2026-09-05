# 创意工坊地图 Demo 保留录制、禁止解析：实际执行 AI 交接方案

## 0. 交接说明

本文件给下一位“实际执行 AI”。两个 AI 不共享上下文；执行前必须重新读取本文件、当前工作树、SQLite schema、Demo watcher 和实际 CS2 录像目录。

**用户要求：** 创意工坊地图仍然正常录制 Demo、保留在录像库/磁盘中；但创意工坊地图的 Demo 不得进入 demoparser、不得创建或领取 core/spatial 解析任务、不得生成战报。官方地图 Demo 的现有录制和解析能力保持不变。

**本轮范围：** 只修改 Demo 来源判定、入队门禁、赛后候选选择、状态展示和测试；不修改 `gameinfo.gi`、CS2 启动、VAC、比分、Rating 算法或 demoparser vendor。

**成功定义：**

1. CS2/助手的自动录制设置不因地图类型关闭；创意工坊对局退出后 `.dem` 文件仍落盘并可在资源管理器中找到。
2. 创意工坊 Demo 在任何入口都不会调用完整 Demo parser，不会产生 `analysis_jobs` queued/processing 记录。
3. 创意工坊 Demo 在录像库中显示为“已保留，未解析（创意工坊地图）”，可定位/删除，但不可误显示为“待解析/解析中/解析完成”。
4. 赛后自动战报不会选中创意工坊 Demo 等待解析或弹出报告；官方地图 Demo 流程不回归。
5. 不确定来源时宁可不解析，不得因为无法识别而把可能的创意工坊 Demo 送入 parser。

## 1. 当前代码调查结果

### 1.1 入口和调用链

- `src-tauri/src/services/demo.rs::refresh_watcher()`：notify 监听所有 `.dem`，稳定后直接调用 `import_file(..., "watcher", false)`。
- `src-tauri/src/services/demo.rs::scan()`：递归收集所有 `.dem`，逐个调用 `import_file(..., "scan", false)`。
- `src-tauri/src/services/demo.rs::import_file()`：先 `parse_gate::assert_parse_allowed()`，校验 `.dem`/header/稳定性，再写入 `demo_files` 并创建 `analysis_jobs` queued；当前没有地图来源过滤。
- `src-tauri/src/services/demo.rs::execute_core_job()`：领取后调用 `parse_report()`，完整读取和解析 Demo；这是必须对创意工坊路径阻断的最晚防线。
- `src-tauri/src/services/demo.rs::process_next_job()`、`process_next_spatial_job()`：从 queued 任务领取，必须再次检查 Demo 的 workshop 状态，防止旧任务或竞态绕过入口。
- `src-tauri/src/services/demo.rs::retry_analysis_job()`：当前允许重试 error/canceled 任务，必须拒绝 workshop/unknown 来源。
- `src-tauri/src/services/demo.rs::commands::import_demo_file` 与 `retry_demo_parse`：手动导入/重试也必须经过同一判定，不能只修 watcher。
- `src-tauri/src/demo/post_match.rs::finish_session()`：先 `demo::scan()`，再按新增文件选择 candidate 并等待 `done`；必须排除 workshop/unknown candidate，否则会出现赛后超时或错误弹窗。

### 1.2 当前数据模型限制

`demo_files` 已有 `path`、`source`、`status`、`map_name`、`error_code`、`error_detail` 等字段，但 `map_name` 只在 `parse_report()` 完成后写入，无法作为解析前判定。

`validate()` 只读取前 8 字节 `PBDEMS2`/`HL2DEMO` header；它不能安全得出地图来源。不能先完整解析再决定跳过，因为那已经违反要求。

## 2. 判定策略

### 2.1 来源枚举

新增 Rust 内部枚举/字符串常量（名称可按现有风格调整）：

```text
official       官方地图，可进入解析
workshop       创意工坊地图，保留但禁止解析
unknown        无法可靠判断，保留但禁止解析
```

数据库字段建议新增：

```text
map_source TEXT NOT NULL DEFAULT 'unknown'
```

允许值必须由 Rust 白名单约束；不要让前端任意传入来源。

### 2.2 判定优先级

判定必须发生在 `import_file()` 创建 `analysis_jobs` 之前，且由单一函数复用：

```text
classify_demo_source(path, optional_session_context) -> DemoSource
```

按以下优先级执行：

1. **明确路径证据：** 路径组件（大小写不敏感）包含 `workshop`、`ugc`、`steamapps/workshop/content/730`、`workshop/content/730` 等，直接判定 `workshop`。禁止只用文件名中偶然出现的普通单词匹配。
2. **录制会话证据：** `post_match` 会话启动时保存当时地图来源；若启动/运行期已经确认是 workshop，则本次新增 Demo 一律标记 `workshop`，即使最终 `.dem` 落在普通 `replays` 目录。
3. **轻量元数据证据：** 如果 vendored `LaihoE/demoparser` 提供只读 Demo header/服务器地图字段的 API，可以只读取 header/首段元数据，不执行 entity、game event、second pass 或 `parse_report()`；地图值带 `workshop/`、`ugc/`、创意工坊 ID 或不在官方地图白名单时判定 `workshop`。该读取器必须独立命名为 `read_demo_metadata_header`，并有测试证明没有调用完整 parser。
4. **官方白名单：** 只有明确匹配项目内官方地图元数据（`src-tauri/src/demo/map_metadata.rs` 当前嵌入地图集合）且没有 workshop 证据，才判定 `official`。
5. **其余全部 `unknown`：** 不得猜测为 official。unknown 同样禁止解析，等待用户显式确认或后续增加可靠来源证据。

判定冲突时取更保守值：`workshop > unknown > official`。例如路径看似 replays，但会话上下文是 workshop，最终仍为 workshop。

### 2.3 录制与解析分离

保留现有 `recording_desired()`、`apply_demo_recording()`、`tv_enable 1`、`tv_autorecord 1` 行为，不因 workshop 关闭录制。

录制会话上下文必须扩展为可选地图来源：

```text
PendingSession/RunningSession:
  kind
  map_source: official|workshop|unknown
  map_name: Option<String>
```

如果当前项目没有可靠的运行期地图来源采集能力，默认会话来源为 `unknown`，这样仍然保证不解析创意工坊 Demo；官方地图自动解析需通过可靠的 Demo header/地图白名单判定恢复，不得把 unknown 放行。

## 3. 实现要求

### 3.1 数据库迁移

在 `open_db()` 的现有 schema/migration 链中增加一次可重复迁移：

```sql
ALTER TABLE demo_files ADD COLUMN map_source TEXT NOT NULL DEFAULT 'unknown';
```

实际实现需先检查 `PRAGMA table_info(demo_files)`，避免重复 ALTER。历史记录统一初始化为 `unknown`，不能批量猜测为 official，也不能自动解析历史 unknown。

对已有 queued/processing workshop 候选：启动恢复时将其改为 `skipped`（或项目选定的终态）并写入：

```text
error_code = DEMO_WORKSHOP_PARSE_SKIPPED
error_detail = 创意工坊地图 Demo 已保留，但按策略不解析。
map_source = workshop
```

如果不新增 `skipped` 状态，使用现有 `canceled` 但必须在 UI/查询层通过 error_code 明确显示“已保留未解析”，不能显示普通取消。

### 3.2 `import_file()` 单一门禁

在稳定性检查和 header 校验后、任何 `analysis_jobs` INSERT 之前调用 `classify_demo_source()`，并把 `map_source` 持久化。

- `workshop`：插入/更新 `demo_files` 为 `status='skipped'`，写 `DEMO_WORKSHOP_PARSE_SKIPPED`，更新 `demo_paths` 以保留文件索引；删除或取消该 Demo 已存在的 queued core/spatial jobs；返回结构化 `DemoImportResult`（建议 `status='skipped'`, `skipped_reason='workshop'`）。绝不调用 parser。
- `unknown`：同样 `status='skipped'`，错误码 `DEMO_MAP_SOURCE_UNKNOWN`；不得创建 job。文案说明“已保留，来源无法可靠确认，暂不解析”。
- `official`：沿用现有 queued/cache 逻辑。

必须保证 cache hit 不会把旧 `done` 报告继续当有效结果：如果当前记录 `map_source` 是 workshop/unknown，始终返回 skipped，不进入 `presentable_report()`。

### 3.3 任务 worker 最晚防线

在 `process_next_job()` 的 SQL claim 中加入 `d.map_source='official'` 条件；不要只依赖 Rust 取出后判断，因为 claim 已可能改变 lease/状态。

在 `execute_core_job()`、`process_next_spatial_job()` 领取后、解析前再次调用 `classify_demo_source()`：

- 非 official：立即结束任务为 skipped，写稳定错误码，不调用 `parse_report()`、空间 parser 或任何 demoparser second pass；
- official：继续现有 `parse_gate`、稳定性和 checksum 流程。

`retry_analysis_job()` 必须先读取关联 Demo 的 `map_source`；非 official 返回 `[DEMO_PARSE_SKIPPED_NON_OFFICIAL_MAP]`，不得把任务重新置 queued。

### 3.4 watcher、scan、manual import、post-match 全覆盖

- watcher：保留监听和稳定等待；调用 `import_file()` 后接受 `skipped` 为成功处理，移除 pending，发出 filesystem-changed，不能反复重试。
- scan：`DemoScanResult` 增加 `skipped` 或 `workshop_skipped` 计数；创意工坊文件算“已发现/已保留”，不算 failed。
- manual import：界面允许导入并索引文件，但结果必须显示 skipped；不能提供“继续解析”按钮绕过门禁。
- retry command：对 skipped workshop/unknown 直接拒绝并给出稳定错误码；只有官方 Demo 可重试。
- post-match：`finish_session()` 在 candidate 过滤阶段排除 `map_source != official` 和 `status='skipped'`；如果本次只有 workshop Demo，发出明确的 `POST_MATCH_WORKSHOP_DEMO_SKIPPED` 或静默完成但不得弹失败战报。建议发事件让 UI 显示“录像已保存，创意工坊 Demo 未解析”。
- schema migration/recovery：启动时扫描旧任务并按上述规则隔离，不能因恢复 queued 而绕过来源门禁。

### 3.5 UI/DTO

更新 `src-tauri/src/models/demo.rs` 的 `DemoListItem`、`DemoImportResult`、任务 DTO，增加 `map_source`/`skipped_reason`（serde camelCase）。

更新录像库视图和 toast：

- `workshop`：`已保留 · 创意工坊地图 · 未解析`；提供“打开文件位置”和“删除”操作；隐藏“播放/重试解析”或明确禁用原因。
- `unknown`：`已保留 · 地图来源未确认 · 未解析`；允许打开位置，不允许重试解析。
- `official`：保持现有“待解析/解析中/已完成”。

不得只依据 `map_name` 渲染 workshop，因为 workshop Demo 永远不会完成完整解析；必须使用 `map_source`/error_code。

## 4. 测试计划

### 4.1 单元测试

新增 `classify_demo_source` 测试：

1. `.../steamapps/workshop/content/730/123/demo.dem` -> workshop。
2. 路径含 `ugc`/`workshop` 的大小写变体 -> workshop。
3. 普通 `.../game/csgo/replays/auto.dem` + 官方地图 header/白名单 -> official。
4. 普通 replays + 无法读出地图 -> unknown，不能 official。
5. workshop 会话上下文覆盖普通 replays 路径 -> workshop。
6. 冲突证据优先级为 workshop > unknown > official。

### 4.2 解析阻断测试

1. `import_file(workshop)`：`demo_files` 有记录、状态 skipped、无 core/spatial job、无 parser 调用。
2. `import_file(unknown)`：同上，错误码为 `DEMO_MAP_SOURCE_UNKNOWN`。
3. watcher 发现 workshop Demo：稳定等待完成一次后移除 pending，不循环重试。
4. scan 统计 skipped，不计 failed。
5. manual import 与 retry command 不能绕过门禁。
6. worker SQL 不会 claim 非 official 记录；即使手工插入 queued workshop job，worker 领取后也会隔离而不调用 parser。
7. 启动恢复会隔离旧 queued/processing workshop 任务。
8. `post_match` 本次只有 workshop Demo 时不等待 `done`、不发 `report-ready`、不误报普通 parse failed。
9. 官方 Demo 的现有测试仍能创建 queued job、解析并生成 report。

### 4.3 parser 调用证据

测试中使用计数 mock/测试 hook 或隔离 fixture，证明 workshop/unknown 路径没有进入 `parse_report()`、`cs2_demoparser` second pass、空间解析和 `persist_normalized_report()`。不能只断言最终 status=skipped，因为 parser 可能已经被调用。

## 5. 真实验收步骤

真实游戏和创意工坊地图效果由用户执行；实际执行 AI 只负责静态/隔离测试和证据整理。

1. 备份当前 Demo SQLite 和 `replays` 目录，记录应用版本/HEAD。
2. 开启自动录制，进入一个创意工坊地图，完成短对局并正常退出 CS2。
3. 确认新 `.dem` 文件存在、size/mtime/hash 已稳定；确认助手未删除或移动文件。
4. 刷新录像库，确认该文件为 `workshop / skipped`，无 queued/parsing job。
5. 查看 runtime log，确认出现 `DEMO_WORKSHOP_PARSE_SKIPPED`，且没有 parser 错误或报告 ready 事件。
6. 再进行一场官方地图 BOT/本地对局，确认仍录制并按现有流程解析；这一步仅用于证明没有误伤官方地图。
7. 关闭助手后重新打开，确认 workshop Demo 仍为 skipped，任务恢复不会重新入队。

验收标准是“文件存在且未解析”，不是“创意工坊 Demo 能生成战报”。

## 6. 交付报告与证据

实际执行 AI 完成后新增：

`docs/workshop-demo-record-but-no-parse-execution-report-20260827.md`

必须记录：

- 修改文件、数据库迁移和状态枚举；
- 每个入口的阻断点；
- parser 未调用的测试证据；
- workshop/unknown/official fixture 的分类结果；
- watcher/scan/post-match/worker/retry 的测试命令和结果；
- 真实验收中的 Demo 路径、size/mtime/hash（路径可脱敏）、SQLite 状态和任务计数；
- 官方地图回归结果；
- 未完成的用户实机步骤和剩余限制。

不得把“录制成功”写成“解析成功”，不得把静态 fixture 写成真实创意工坊验收。

## 7. 停止条件

遇到以下任一情况立即停止，不得放宽为 official：

- 无法在 parser 之前可靠分类地图来源；
- 只能通过完整解析后才能知道是 workshop；
- 任一入口仍能为非 official 创建 queued job；
- worker 已调用 parser 后才发现 workshop；
- 赛后流程仍等待 skipped Demo 的 done 状态；
- 数据库迁移失败、旧任务无法隔离或状态回滚失败；
- 真实录像文件不存在或发生写入中断。

停止时保留录像和数据库备份，记录最早失效入口；不要删除用户 Demo，不要修改 demoparser vendor 来“硬猜”地图类型。

## 8. 最终方案结论

实现原则是：**录制与解析彻底分离。创意工坊地图仍然使用现有 `tv_enable/tv_autorecord` 保存 Demo；在任何进入 `analysis_jobs` 或 parser 之前，把 workshop 和无法确认来源的 Demo 标记为 skipped 并保留文件。只有有可靠官方地图证据的 Demo 才允许解析。**
