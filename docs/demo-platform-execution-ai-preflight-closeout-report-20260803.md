# CS2 Demo 功能：执行 AI 预验收收口报告（2026-08-03）

> **用户验收续作：** 执行 AI 预验收已经通过。下一阶段由用户按 `demo-platform-user-bot-cs2-acceptance-plan-20260803.md` 完成真实 BOT 五回合、watcher 和战报对照。当前可先验收 BOT 闭环；真实 tick 跳转因缺少可见产品入口保持未执行，禁止用 DevTools 或临时命令替代。

结果：AI 预验收通过，可以请求用户 BOT 验收

本报告执行 `demo-platform-execution-ai-preflight-closeout-plan-20260803.md`，没有重做阶段 A-C，也没有启动 CS2、构建 installer、commit、push、release 或 deploy。

## 最终候选

- EXE：`E:\CS2AS05\src-tauri\target\debug\ai_pc_fac.exe`
- SHA-256：`D9BFC7B3E30B4155D036D86CD25EF89F541F895E678BADB8507578B0C968A182`
- identifier：`com.aipc.cs2botimprover.preflight`
- HEAD：`8552b554993fec66866d2133d315342be3e0cbca`
- tracked diff fingerprint：`2eac1d8f27ff4075f7005ba8552329999bab2889`

## Gate 摘要

| Gate | 结果 | 证据摘要 |
| --- | --- | --- |
| 四尺寸真实 Tauri | 通过 | main `1440x900`、`1100x700`；scoreboard `1280x800`、`980x640`；DPI 120，全部精确命中 |
| cold start | 通过 | 5 次 interactive-ready，P95 `1419.001 ms` |
| positions IPC | 通过 | 30 次完整 Tauri invoke，8208 points，P95 `72.3 ms` |
| viewer FPS/CPU | 通过 | `164.959 FPS`；active/paused/tab-away CPU mean `1.4530%/0.2773%/0.2126%` |
| RAF/tick cleanup | 通过 | paused 与 tab-away 均 RAF `0`，tick 不变；tab-away viewer unmounted |
| 20 次 lifecycle | 通过 | mount/unmount `20/20`；WorkingSet、PrivateMemory、HandleCount 首尾与斜率均下降 |
| DB 静止性 | 通过 | 6 次 live 纯文件样本一致，WAL/SHM 不存在；副本 integrity `ok`、v7、jobs/leases 0 |
| 自动化 | 通过 | Vitest 124；Rust 54 passed/3 ignored；workspace/typecheck/lint/build/fmt/check/clippy/diff-check 全通过 |
| 最终 hash 一致性 | 通过 | 完整运行 evidence 与 recapture 均绑定 `D9BFC7...A182` |

原始 `1440x900 library` PNG 曾被外部窗口遮挡，文件保留并作废；同一候选的延迟 recapture 已单独留证。用户随后要求不再继续纠结该项，因此未再重复运行。

## Evidence

- 完整报告：`E:\CS2AS05\workspace\release-evidence\demo-preflight-closeout-20260803-193659\execution-report.md`
- 主 evidence：`E:\CS2AS05\workspace\release-evidence\demo-preflight-closeout-20260803-193659`
- 延迟 recapture：`E:\CS2AS05\workspace\release-evidence\demo-preflight-library-recapture-20260803-195113`

下一步由用户执行真实 BOT 五回合、游戏内记分板对照和 tick 跳转体验；测量整理、差异定位和代码修复仍由执行 AI 负责。
