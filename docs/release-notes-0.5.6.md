# CS2 人机增强助手 0.5.6

## 新增

- Demo 录像库新增 CS2 播放、Explorer 文件定位、逐行 busy 状态、按钮禁用态、tooltip 和 ARIA 名称。
- 新增基于最新 Demo 的独立赛后计分板、地图资源、双向对枪矩阵和 BOT/观战身份展示。
- 新增回放 session suppression，避免回放退出触发赛后战报。

## 修改

- Demo 会话改由统一 coordinator 管理；比分按已完成回合的规范化 winner 累计。
- 报告展示改为 schema/adapter/metrics、core job、玩家、回合和 scoreboard 状态联合门禁。
- `simple-rating-v1` 保持多因子公式，以 `(K + A / 5) / max(D, 1)` 为 80% 主因子，伤害为 10%，生存代理和助攻各为 5%，最终限制在 `0..3`。
- 窄窗口录像表格改为容器横向滚动。

## 修复

- 修复 CS2 session 与 Demo 异步解析竞态，以及旧 Demo 或 parsing 中 Demo 被误选为最新战报。
- 修复未完成、损坏、零回合、空玩家和版本过期报告打开空计分板。
- 修复回放退出误触发赛后战报，以及外部 Demo 播放可能改写原文件的问题。
- 修复 Explorer 特殊路径参数处理风险。

## 其他

- 并行集成 BotVision 0.2.2；补充第三方来源、许可证和 NOTICE。
- 播放前必须退出运行中的 CS2；保留旧 Demo 兼容性限制和 Windows Authenticode `NotSigned` 状态说明。
- 比分累计路径支持多段加时，真实多次加时 Demo 样本待补，不能视为已实证。
- 播放与文件定位行为语义参考 `CS2-insight-agent` 固定 commit；未复制其源码、测试、样式、文案或资产。

## 使用限制

- 播放 Demo 前必须退出正在运行的 CS2；播放以 `-insecure` 离线模式启动。
- 旧版本 Demo 若与当前 CS2 不兼容，应用不会重写原文件。
- 安装器使用 Tauri updater 签名；Windows Authenticode 状态单独记录，不能与 updater `.sig` 混同。
- BotVision DLL 的 Windows Authenticode 状态为 `NotSigned`。其上游再分发许可证仍需在公开发布前完成最终人工复核。

## 验证状态

此文件随本地发布候选生成。自动化、真实 Explorer 定位、真实 CS2 Demo 播放、至少三回合 BOT 新局、干净安装、从 0.5.5 覆盖升级、卸载重装和旧版 updater 验证已由用户确认通过。多次加时真实样本仍需单独记录；在完成用户授权的源码提交、tag、公开发布和线上 feed 回读前，本候选仍不是正式公开发布。

## 第三方说明

播放与文件定位的行为语义参考了 `DrEAmSs59/CS2-insight-agent@17d2a213ee8c32608feee3c63f0b6d05eef8f945`。该项目使用 PolyForm Noncommercial 1.0.0；本项目没有复制其 Python、React、测试、样式、文案或资产。其他第三方来源、固定提交和许可证见 `NOTICE.md`。
