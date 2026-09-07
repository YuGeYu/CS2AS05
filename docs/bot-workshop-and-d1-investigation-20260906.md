# 人机强度工坊与线上 D1 查询调查（2026-09-06）

## 结论摘要

1. **短暂无响应的主因已定位为本地同步工作过重，而不是 Vue 渲染本身。** `list_bot_profiles`、`open_bot_profile`、`create/save/apply` 都在 Tauri 命令调用链中同步执行 VPKEdit 子进程、轮询等待、SHA-256、提取和文件读写。每次打开还会同时触发 `list_bot_profiles + get_bot_workshop_state`，随后再执行一次 `open_bot_profile`；`list` 和写操作内部又会重复执行 `inspect_vpk_tool()`（读取 EXE、启动 `--help`）。这会让命令线程在本地磁盘、杀毒扫描或 VPKEdit 较慢时持续占用，表现为助手窗口短暂无响应。

2. **少数玩家的 `extract ... botprofile.db` 失败，最可信的组合原因是文件锁/并发争用，错误码被错误归类。** `open()` 没有调用 `require_cs2_closed()`，所以 CS2 正在运行时仍会尝试读取可能被游戏或杀毒软件占用的 VPK。前端档案按钮只用 `action` 禁用，`busy` 期间仍可重复点击；而 `open()` 的 workspace 固定为 `open-{profile_id}-{进程 PID}`，并发打开会把多个 VPKEdit 提取进程导向同一个输出路径。任一占用、权限或并发冲突都被统一包装成 `[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID]`，因此当前提示不能证明 VPKEdit 本体损坏。 

3. **线上 D1 查询存在可压缩的固定成本。** 客户端启动时 `App.vue` 无条件执行一次公告请求和一次更新检查。Worker 的公告接口每次执行 `SELECT * FROM site_notices ...`；更新接口每次最多读取 50 条 `software_releases`，随后再次读取 `software_update_settings`，并在自更新状态检查中对最新 R2 对象执行 `HEAD`。公开 JSON 响应没有 `Cache-Control`/`ETag`，边缘无法复用结果。现有索引基本覆盖筛选条件，但没有消除“每个启动都打 D1”的问题。

## 证据位置

- 强度工坊同步子进程与 30 秒轮询：`src-tauri/src/services/bot_difficulty.rs` 的 `run_cli()`。
- 打开时重复调用：`src/components/BotDifficultyWorkbench.vue` 的 `load()` 先 `Promise.all(list, state)`，之后 `selectProfile()` 再调用 `open`。
- 固定 workspace：`src-tauri/src/services/bot_difficulty.rs` 的 `open()` 使用 `open-{profile_id}-{std::process::id()}`。
- 打开未阻止 CS2：同一文件的 `open()` 没有 `require_cs2_closed()`；只有 `create/save/apply` 调用了该检查。
- 前端并发窗口：`BotDifficultyWorkbench.vue` 的档案按钮只绑定 `:disabled="!!action"`，读取阶段 `busy=true` 时仍可触发 `selectProfile()`。
- 公告查询：`E:\cs2as\src\worker.ts` 的 `getSiteNotices()`。
- 更新查询：`E:\cs2as\src\worker.ts` 的 `getSoftwareUpdatePayload()`、`selfUpdateAvailability()`、`getSoftwareReleases()`。
- 响应头：`E:\cs2as\src\worker.ts` 的 `json()` 只设置 `content-type`，公开公告/更新分支没有缓存头。
- 本地 CLI 重现：同一 VPK、同一输出文件连续提取两次均返回 0，因此“目标文件已经存在”不是单独的充分原因；报错仍应优先调查锁、权限、并发和输入文件状态。

## 分项判断

### 1. 无响应

当前链路一次打开大致为：

`list_bot_profiles` → `inspect_vpk_tool`（读 EXE + 启动 `--help`） → 前端 `get_bot_workshop_state`（再次做同样自检） → `open_bot_profile` → VPKEdit `--extract` → 读取/校验 DB。

写入后还会执行复制 VPK、`--remove-file/--add-file` 重包、再次提取回读、SHA-256、备份和原子替换，最后 `load()` 再重复上述读取链路。`run_cli()` 使用同步 `std::thread::sleep` 轮询，任何子进程慢、磁盘忙或安全软件扫描都会直接延长 Tauri 命令完成时间。短暂无响应与该链路的同步性质一致。

### 2. 少数机器提取失败

建议按以下优先级收集玩家现场证据：

1. 错误发生时 CS2 是否仍在运行；若是，先退出 CS2 后重试。
2. 同一时间是否重复点击档案、快速切换档案或连续打开/关闭工坊。
3. 失败 workspace 中 VPK 的 ACL、文件属性和是否被 Defender/第三方安全软件隔离或扫描。
4. 记录 VPK SHA-256、VPKEdit SHA-256、进程 PID、输入文件大小、输出目录 ACL 和 Win32 错误信息。

代码层修复顺序应为：为每次提取生成唯一 workspace；给 `open()` 增加 CS2 运行门禁或先复制输入 VPK 到临时快照；在前端以 `busy` 统一禁用档案按钮并丢弃过期请求；将“文件被占用/权限不足/输入 VPK 不可读/VPKEdit 退出码”拆成独立错误码。不要继续把所有失败显示为 `TOOL_DEPENDENCY_INVALID`。

### 3. D1 压力

当前公开启动流量至少包含：

- 更新接口：一次 `software_releases` 列表查询（最多 50 行），若有最新版本，再查询设置并做一次 R2 `HEAD`。
- 公告接口：一次 `site_notices` 查询，当前返回全部匹配公告，没有 `LIMIT`。

推荐的低风险优化顺序：

1. 在 Worker 对公开公告和更新 JSON 增加短 TTL：`Cache-Control: public, max-age=60, s-maxage=300, stale-while-revalidate=600`，并按查询参数生成稳定 ETag；客户端不要使用 `no-store` 请求更新接口。
2. 更新接口只返回最新一条和必要的历史条目，或把历史限制为 10 条；公告接口限制为合理数量（例如 20 条）。
3. 将公开更新状态拆为轻量路径：先读取最新 release；只有客户端明确需要下载元数据时才做设置查询和 R2 `HEAD`，避免每次普通检查都触发 R2 检查。
4. 客户端进程内增加单飞请求（同一时刻只允许一次公告/更新请求）和短时内存缓存；不要使用 `localStorage` 保存业务数据。
5. 保留现有索引并检查线上 `EXPLAIN QUERY PLAN`；不要通过全表缓存或把 D1 数据复制到客户端来掩盖权限/发布一致性问题。

## 本轮边界

本轮仅完成代码与本地 CLI 调查，未修改强度工坊实现，未修改 `E:\cs2as` Worker，未执行线上 D1 查询、迁移或部署。要把上述判断转成修复，下一轮应先做本地并发/锁重现和 Worker 缓存响应测试，再分别验证 Windows/Tauri 与真实 CS2。
