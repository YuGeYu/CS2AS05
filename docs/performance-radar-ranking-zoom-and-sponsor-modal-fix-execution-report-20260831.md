# 表现雷达排名、缩放与赞助模态框执行报告

日期：2026-08-31  
状态：代码修复完成，指定 Demo 与真实 Tauri 界面验收待执行。

## 已修改

- `src-tauri/src/models/demo.rs`、`src-tauri/src/services/demo.rs`
  - `PerformanceRadarPlayer` 新增 `rankingRating` 与 `rankingRatingModel`。
  - 直接读取 `match_players.rating/rating_model`，不再从六维 raw 或 score 推导排名字段。
- `src/types/demo.ts`、`src/features/demo/components/MatchPerformanceRadar.vue`
  - 前端排名仅以 `lb-rating-2.0` 的最终 Rating 降序为主键。
  - Rating 缺失时才进入稳定 `stableKey` 降级顺序；BOT 与真人不区分处理。
  - 文案明确排名按 LBRating 2.0，避免将雷达轴总和误认为 Rating。
- `src/features/demo/components/PerformanceRadarChart.vue`
  - 移除改变 SVG 百分比宽度的布局缩放。
  - 固定 `viewBox`、中心 `(320,320)`，仅以 `radarRadius=222*displayScale` 改变绘图区半径。
- `src/components/DonateModal.vue`、`src/components/SupportActions.vue`、`src/styles/main.css`
  - 安装与诊断页加入“赞助开发”按钮。
  - 新增 `role=dialog`、`aria-modal=true`、关闭按钮、Esc、背景点击、body 滚动锁定和微信赞赏码展示。
- 新增本地资源：`public/assets/wechat-reward.png`
  - 源文件：`E:\cs2as\public\assets\wechat-reward.png`
  - 大小：214537 bytes
  - SHA-256：`908D5D946D3059478FF0922781F6880811CDE39B08569FAA9C17B1E47BBB40D8`

## 自动化验证

- `npm test -- --run tests/performance-radar.spec.ts tests/match-performance-radar.spec.ts`：9 tests passed。
- `npm run typecheck`：通过。
- `cargo check --manifest-path src-tauri/Cargo.toml`：通过；仅保留既有依赖 warning。

## 尚待真实验收

当前环境未加载用户指定 `auto-20260826-1404-de_dust2-advent.dem`，因此未记录原始/最终 Demo hash、报告 JSON、所有玩家 Rating/排名、`advent` 参赛身份或真实冠军对账；也未启动 Tauri WebView，未生成雷达缩放前后 bounding rect、四种视口截图或赞助模态框截图。不能据此宣称指定 Demo 和真实界面验收完成。
