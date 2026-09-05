# 0.5.8 Demo 对局表现雷达执行报告

## 实施结果

Demo 对局报告现已在“记分板”之后、“回合”之前提供“表现雷达”页。页面用六个彼此独立的维度展示玩家本场画像：火力、输出、生存、参战、协同、先手影响。该页面只做可视化，不计算新的总 Rating，不参与 MVP、排序或旧评分流程。

后端、Tauri、TypeScript 和 Vue 统一使用 `performance-radar-v1`。查询只读取当前 Demo 的 `match_players`、`players` 和 `player_round_stats`，使用绑定参数隔离 Demo；最多同时请求三名玩家，未知 stable key 和人数超限均返回结构化错误。

## 文件落点

新增：

- `src/features/demo/components/MatchPerformanceRadar.vue`
- `src/features/demo/components/PerformanceRadarChart.vue`
- `src/features/demo/radar/performance-radar.ts`
- `tests/performance-radar.spec.ts`
- `tests/match-performance-radar.spec.ts`

修改：

- `src-tauri/src/models/demo.rs`
- `src-tauri/src/services/demo.rs`
- `src-tauri/src/commands/demo.rs`
- `src-tauri/src/lib.rs`
- `src/types/demo.ts`
- `src/services/tauri/demo.ts`
- `src/views/DemoReviewView.vue`
- `src/styles/main.css`

## 计算与缺失值

所有图形分数限定在 `0..100`，不计算六轴平均值。逐回合 K/D/A、伤害、生存和 KAST 优先来自 `player_round_stats`；首杀、首死与交易读取当前 Demo 的规范化数据。零正式回合整名玩家 unavailable；单轴输入缺失只让该轴为 null，不补造零值。SVG 遇到 null 轴时断线，HTML 精确数据表保留原始值、基准和质量状态。

## UI 与无障碍

图表使用项目内原生 SVG，没有新增图表库、CDN 或外部字体。包含五圈网格、六轴标签、分数与原始值、玩家图例、线型差异、SVG `title/desc`、多边形 aria-label，以及图外精确数据表。进入和切换仅动画路径与 opacity，支持 `prefers-reduced-motion`，没有全局循环 RAF。

玩家选择稳定按队伍编号和 stable key 排列，不依据 Rating 或 KD。默认不选择 observer；第四名按钮明确禁用并说明上限。Demo 或玩家切换时使用请求序号屏蔽旧响应，避免旧请求覆盖新页面。

## 上游说明

设计调查参考 `TablemanLiu/Rock-Radar` 固定提交 `3d7b358b163a3cb15e930ad2224c98b994310f70`（MIT），实现仅重写六轴几何和交互思想，没有复制上游源码或片段，因此没有新增 `NOTICE.md` 条目。

## 验收与证据

完整证据位于：

`E:\CS2AS05\workspace\release-evidence\performance-radar-0.5.8-20260816-140751`

真实历史 Demo 2 已在 Tauri WebView 中完成 JSON 回读和 1440×900、980×640 窗口截图。自动测试、hash、命令 exit code、基线工作树和验收边界详见该目录中的 `acceptance-summary.md` 与各日志文件。

完整仓库 `npm run lint` 仍会扫描基线未跟踪目录 `artifacts/thank-you-video/edge-profile-*`，其中浏览器扩展生成代码产生 2004 个既有错误。本功能相关文件的 Oxlint 与 ESLint 均为零错误、零警告；未为了本功能删除基线产物或调整全局 ignore。

## 交付边界

已于 2026-08-16 15:39 构建 Windows x64 NSIS 安装程序：

`E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.8_x64-setup.exe`

安装包为 114641481 bytes，SHA-256 为 `303E5FD2DC4C570D786F04DD6B68522F7B1CA43D5283A5F121F788B23BAE666B`。同批生成 436-byte Tauri updater `.sig`；Windows Authenticode 仍为 `NotSigned`。本轮未执行安装、提交或推送 Git，未上传、部署或发布。
