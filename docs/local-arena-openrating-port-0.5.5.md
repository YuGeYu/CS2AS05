# Local-Arena 战报契约参考记录

- 上游：https://github.com/numakkiyu/Local-Arena
- 固定提交：`fad7e4ab7441f6bb95ebe9f6186169dc424ff008`
- 许可：AGPL-3.0；本项目为 AGPL-3.0-or-later。
- 参考文件：`OpenRating.cs`、`MatchStatistics.cs`、`TradeTracker.cs`、`open-rating-3.0-proxy-v1.json` 与结果表格信息层级。
- 落地文件：`src-tauri/src/services/simple_rating.rs`、`src-tauri/src/models/demo.rs`、`src/components/scoreboard/PostMatchScoreboard.vue`。

本项目仅参考结果信息架构，没有整体合并上游项目，也不在用户界面展示上游品牌。最终模型是本项目独立的 `simple-rating-v1` 简易 Rating，不是 OpenRating，也不声称为 HLTV 官方 Rating。
