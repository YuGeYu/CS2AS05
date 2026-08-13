# CS2 Demo 功能：执行 AI 独立预验收报告（2026-08-03）

## 1. 结论

**结果：AI 预验收未完成。当前不得请求用户进行 BOT 验收，也不得称为功能候选、安装候选或发布候选。**

阶段 A、B、C 与自动化门禁已完成；真实 Tauri 已证明历史 dust2 报告、viewer 播放/暂停和 heatmap 非空，但阶段 D/E 仍缺少最终 debug EXE 的四尺寸真实截图、20 次 viewer 进入/离开趋势、播放/暂停/切页 CPU、FPS、cold start 和 positions decode P95。根据预验收方案，任何一项不得用 Web fixture、估算或旧二进制代替。

本轮没有 reset/restore/clean，没有新开分支，没有 commit/push/release/deploy，没有构建 installer，没有启动 CS2，没有触碰来源不明的历史进程。

## 2. 工作树与候选

- 工作区：`E:\CS2AS05`
- HEAD：`8552b554993fec66866d2133d315342be3e0cbca`
- 最终 tracked diff fingerprint：`ac7d4b9ecec2396faf9e6b969f56f533222fc5c9`
- 工作树：保留既有大量未提交 Demo 改动，并新增本轮实现、测试、测量脚本和证据；未回退用户改动。
- 最终 debug EXE：`E:\CS2AS05\src-tauri\target\debug\ai_pc_fac.exe`
- EXE bytes：`37008896`
- EXE SHA-256：`5371234979525F1A3159BDD6C67991A44AD3D5A91E844BC95778BE697D866848`
- EXE mtime UTC：`2026-08-03T06:43:21.0962114Z`
- `cargo build --manifest-path .\src-tauri\Cargo.toml`：exit 0。
- 限制：最终 EXE 在最后一次后端筛选/tick/cleanup 修正后只完成构建，未再次启动做真实 Tauri 回归。

## 3. 地图资源与 SQLite v7

- `cs-demo-manager` 固定 commit：`8961f5072fe4d42803dde68e8e71b3c90b216504`
- `demoparser` 固定 commit：`ba39cc44cd5abfd7f34df2b3c0a7dd3630048311`
- 导入脚本连续运行两次，均生成 44 个唯一地图、51 个雷达文件、146 个 manifest asset copy；三份核心输出 hash 两次一致。
- TypeScript metadata：44 条，SHA-256 `8A926C0FEBA9FDEB24E230E3B640562874723499FB784E6AE8EA03B109604B93`
- Rust JSON metadata：44 条，SHA-256 `56A3A337FA4BD253A6467D59DBA6EB33BEF4ADFB8A56E0269C947EBCEC32315E`
- manifest：SHA-256 `0F373982E30D8FC0704CDFADB3906BB320EE163E2105E500B37042076A4F1F2C`
- dust2 坐标、Nuke/Vertigo 上下层阈值、lower asset hash、资源稳定性自动化通过。
- 隔离 DB：`C:\Users\GOPtZ\AppData\Roaming\com.aipc.cs2botimprover.preflight\demo-review\demo-review-v1.sqlite3`
- DB：`user_version=7`、`integrity_check=ok`、`map_metadata=44`、`matches=4`、`position_chunks=14`、`spatial_done=1`。
- DB bytes：`11788288`
- DB SHA-256：`0BEFC6EDC6962C86BD081424A3933BF89B20AC3FE45C97FD2C0996484D1BD75E`
- v6 backup：`demo-review-v1.sqlite3.v6-1785734428597.bak`
- backup bytes：`6385664`
- backup SHA-256：`05249F4F577C90D5AAAFA83EACEFC4C1EC335E4D338F0C615BE0CA0320C584CE`
- 退出调试 writer 后连续 10 秒、每 2 秒共 6 次采样：DB size/mtime/SHA-256 全部一致，采样期间 WAL/SHM 均为 0。随后使用 `sqlite3` 做最终只读回读时重新创建了 32768 bytes 的 SHM；最终 WAL 仍为 0、主 DB hash 未变、没有 `ai_pc_fac` 进程。该 SHM 时间线单独记录，不把“曾为 0”冒充最终状态。

## 4. 热力图后端

- `HeatmapFilters` 覆盖 kind、round、player、team、layer、radius、opacity，并做 allowlist/range 校验。
- `HeatmapPoint` 现含 tick、x/y/z、round、player、team、weight、kind。
- 修复真实 grenade 事件身份：使用 `COALESCE(actor_key,target_key)`；此前只读 actor_key 会使 utility player/team 筛选恒为空。
- PNG 使用 embedded map metadata、固定雷达资源和 SHA-256 校验，不再使用事件 bbox 拉伸。
- 输出为 1024×1024 RGBA，稳定 radial density，蓝/黄/红 heat ramp，Windows `MoveFileExW(REPLACE_EXISTING|WRITE_THROUGH)` 原子替换，失败清理 `.part`。
- 未知地图、无 lower、资源 hash 不符、无点数据均明确失败。
- 真实 DB 事件事实：dust2 的 `player_death=48` 但坐标为 0；`smokegrenade_detonate=31` 且都有 x/y/z。因此真实导出选择 smokegrenade，不伪造死亡点坐标。
- 真实 DB 导出：input 31、rendered 31、discarded 0。
- round/player/team/upper 子集：`5 / 3 / 17 / 17`。
- 输出：`workspace\release-evidence\demo-preflight-20260803-20260803-045833\backend\real-db-smokegrenade-heatmap.png`
- 输出 bytes：`342566`
- 输出 SHA-256：`F161BA8890D9F062126FCB14A2D9386C8B76C4B7487138734B5AF20F9821D1C9`
- radar asset SHA-256：`6515AB4BA319187B2130EDB0C2F60BA64C9D422E75DE113C65BDF37C2DB1479B`
- decoder 回读：1024×1024 RGBA；真实雷达背景和局部热点非空。
- 输入点反序逐字节稳定、NaN/越界丢弃、Nuke/Vertigo lower hash 等纯函数测试通过。

## 5. Canvas 与 Web 四尺寸

- 新增纯函数边界：coordinates、frame-index、draw-viewer、draw-heatmap。
- lifecycle 测试证明：末帧 RAF=0、pause/visibility/unmount RAF=0、ResizeObserver disconnect、旧请求不覆盖最新响应。
- Web fixture 未加入 production build input；`dist` 搜索没有 fixture。
- 1440×900、1100×700、1280×800、980×640 的 viewer/heatmap 共 8 组通过。
- Canvas CSS/像素尺寸分别为 `640/460/560/400`，全部满不透明像素，各自至少统计到 8192 种颜色。
- 每个 viewport 的 document scrollWidth 等于 viewport width，无横向溢出；视觉检查未见控件重叠。
- Web 证据只用于阶段 C，不冒充真实 Tauri。

## 6. 真实 Tauri

已完成的真实窗口观察：

- 使用隔离 identifier `com.aipc.cs2botimprover.preflight`，避免历史签名实例 single-instance 干扰。
- 录像库显示 4 个真实历史 Demo。
- dust2 文件：`auto-20260729-0958-de_dust2-advent.dem`，demo_id 4，metrics-v2，5 回合，10 玩家，scoreboard quality complete。
- viewer radar 非空，坐标点 8208；Tick 从 352 推进到 424、1456、2576；暂停后再等待 1.2 秒仍为 2576。
- heatmap radar 非空，坐标点 8208，橙红热点清晰覆盖在 dust2 雷达有效区域。
- 实际捕获窗口尺寸为 Computer Use screenshot `1442×902`（目标内容约 1440×900）。

未完成并保持 `PENDING`：

- 窗口边框拖拽两次都保持 1442×902；系统菜单路径引起焦点/导航变化，停止继续使用不稳定方法。
- 1100×700、1280×800、980×640 的真实 Tauri 截图和像素检查。
- 真实窗口截图文件落盘；Computer Use 只提供本轮显示证据，未违反指南重复解码/保存 payload。
- 最终 SHA-256 `537123...D866848` 的 EXE 未重新启动回归。
- viewer 播放 10 秒/暂停 10 秒/切非空间页 10 秒的 CPU 对比。
- 20 次进入/离开 viewer 的 WorkingSet/handle trend。

阶段 D Gate 因以上 `PENDING` 未通过。

## 7. 性能与稳定性

测量脚本：`scripts\measure-demo-preflight.ps1`。

- 500 条合成 Demo library query：30 次；最终进程含 sqlite3 启动 P95 `26.274 ms`，低于 150 ms 门限。
- 真实 DB heatmap query：30 次；最终进程含 sqlite3 启动 P95 `36.392 ms`。
- 查询值包含 sqlite3 进程启动开销，因此是保守值。
- core job 历史 elapsed：949 ms 到 12,760 ms；不能当作 cold start。
- spatial job 历史 elapsed：1,158,375 ms；只记录事实，不用作 positions decode P95。
- round positions decode P95：`null`
- cold start interactive：`null`
- viewer FPS：`null`
- viewer active/paused/tab-away CPU：`null`
- 20 次 viewer loop WorkingSet/handle trend：`null`
- 主 DB 退出稳定：10 秒 hash/size/mtime 通过，最终 WAL=0；诊断回读后 SHM=32768 bytes。`ai_pc_fac` 进程和 Vite 5174 listener 均已结束。

阶段 E 因必需运行指标为 `null` 未通过。

## 8. launch_demo_at_tick 边界

- 没有实际启动 CS2。
- tick `0`、`10_000_000` 允许；`-1`、`10_000_001` 拒绝。
- 空/空白 player key 允许；非空返回 `DEMO_PLAYER_UNSUPPORTED`。
- 中文和空格 Demo path 保持为单独 `OsString` argument，无 shell 拼接。
- path 仍只来自已索引 demo id，缺失源文件沿既有 `path_for_id` 失败。
- Steam resolver 对不存在、错误 executable、优先级和去重已有测试；真实跳 tick 留给用户阶段。

## 9. 自动化门禁

最终结果：

| 命令 | 结果 |
| --- | --- |
| `npm run workspace:check` | exit 0 |
| `npm run typecheck` | exit 0 |
| `npm run lint` | exit 0 |
| `npm test -- --pool=threads --maxWorkers=1` | 33 files / 120 tests passed |
| `npm run build:web` | exit 0；fixture 未进入 dist；保留 Three.js chunk warning |
| `cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check` | exit 0 |
| `cargo check --manifest-path .\src-tauri\Cargo.toml` | exit 0 |
| `cargo test --manifest-path .\src-tauri\Cargo.toml --lib` | 53 passed / 3 ignored / 0 failed |
| 真实 DB heatmap ignored diagnostic | 1 passed / 0 failed |
| `cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings` | 最终 exit 0 |
| `git diff --check` | exit 0 |
| `cargo build --manifest-path .\src-tauri\Cargo.toml` | exit 0 |

Clippy 初次发现项目 crate 一处 `manual_inspect`，改为等价的 `inspect_err` 后通过。初次失败日志与最终通过日志都保留。vendored `third_party/demoparser/parser` 固定保留 10 条既有 warning，没有修改 vendor 以掩盖 warning。

## 10. 未执行与下一步

未执行：用户 BOT 五回合、真实 CS2 tick 跳转、正式 installer、签名、commit、push、GitHub Release、R2/D1、生产 updater。

当前不向用户下发 BOT 验收步骤。下一轮执行 AI 应先针对最终 debug EXE 完成：

1. 受控 Win32/Tauri window sizing 的四尺寸真实截图与像素检查。
2. viewer active/pause/tab-away CPU、tick/FPS 采样。
3. 20 次 viewer 进入/离开的 WorkingSet/handle 趋势。
4. cold start 和 round positions decode P95；无法测量时方案 Gate 仍不得判过。
5. 完成后重新核对 DB 10 秒静止、EXE hash、自动化结果，才可写“AI 预验收通过，可请求用户 BOT 验收”。

## 11. 证据

- 根目录：`E:\CS2AS05\workspace\release-evidence\demo-preflight-20260803-20260803-045833`
- Web 截图与 metrics：`...\web\`
- 真实 DB PNG：`...\backend\real-db-smokegrenade-heatmap.png`
- 性能：`...\performance.json`、`...\performance.csv`
- 自动化：`...\automation\results.json` 与逐命令日志
- Clippy 初次失败：`...\automation\cargo-clippy-initial.log`
- Clippy 最终通过：`...\automation\cargo-clippy.log`
- Debug build：`...\automation\cargo-build-debug.log`
