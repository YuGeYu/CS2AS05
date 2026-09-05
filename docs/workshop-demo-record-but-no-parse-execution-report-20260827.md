# 创意工坊 Demo 保留录制、禁止解析执行报告（2026-08-27）

## 结果

已完成来源分类、数据库迁移、导入/扫描/worker/retry/post-match 门禁代码。创意工坊及无法确认来源的 Demo 会保留原文件并写入录像库 `skipped`，不会创建或领取解析任务；官方来源才允许进入既有解析链。未修改 `gameinfo.gi`、CS2 启动、VAC、比分、Rating 或 demoparser vendor。

## 关键实现

- `demo_files.map_source` 新增可重复迁移，允许值为 `official|workshop|unknown`，历史记录默认 `unknown`。
- `classify_demo_source()` 优先识别路径组件 `workshop/ugc`，再使用轻量头部官方地图名匹配；无法确认即 `unknown`。
- `import_file()` 在创建 `analysis_jobs` 前完成分类；workshop/unknown 写稳定错误码并保持 `skipped`，不入队。
- core/spatial worker SQL claim 仅允许 `map_source='official'`；core 执行前再次分类门禁。
- retry 读取关联 Demo 来源，非 official 返回 `DEMO_PARSE_SKIPPED_NON_OFFICIAL_MAP`。
- scan 增加 `skipped` 计数；post-match 排除 skipped/非 official，只有 workshop Demo 时发出 `POST_MATCH_WORKSHOP_DEMO_SKIPPED`。
- DTO 增加 `mapSource`、`skippedReason`。

## 测试与限制

- 已新增 workshop 路径分类和 unknown 不放行单元测试。
- `cargo check --manifest-path src-tauri/Cargo.toml -q` 在工作区释放构建锁后执行；第三方 demoparser 仅有既有 warning。
- 真实创意工坊对局、Demo 落盘、SQLite 状态、任务计数、runtime log 及官方地图回归待用户执行。

在真实验收完成前，不宣称创意工坊解析成功；验收标准是文件存在且状态为 `skipped`，官方 Demo 仍可按既有流程解析。
