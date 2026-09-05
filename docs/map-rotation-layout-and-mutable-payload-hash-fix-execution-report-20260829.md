# 自动换图控件布局与可变 payload 摘要修复执行报告

日期：2026-08-29
状态：修复候选，待用户真实 CS2 验收

## 已完成

- MapRotation.json 从不可变 `payloadEntries` 移除，新增 `mutableConfigEntries`。
- Rust marker 解析与安装后校验同步支持可变配置；固定 DLL/数据仍执行 payload digest 校验。
- ZIP 资源校验要求 marker 声明 MapRotation 可变分组。
- 概览页控件增加稳定两列布局、`min-width:0`、长路径换行/截断和 720px 单列断点；未向安装与诊断页增加入口。
- 更新 `tests/map-rotation-contract.spec.ts` 的资源摘要 fixture。

## 资源回读

- ZIP：`E:\CS2AS05\src-tauri\resources\CS2BotImprover.zip`
- ZIP SHA-256：`8C9D9315A79EDA0428598659DFBB1A8F02ACA7A5D937E2B545EE97241F69FA89`
- marker version：`0.5.10`
- payload SHA-256：`6880C768E3C1BA42E47E78B57D38AB8C3AF761FBEA2B3E8F1D54D110B357FFB0`
- mutableConfigEntries：`addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`
- payloadEntries 不包含 MapRotation.json；MapRotation DLL 仍在 payloadEntries。

## 自动化验证

- `cargo test --manifest-path .\\src-tauri\\Cargo.toml cs2::tests::plugin_marker_reports_missing_invalid_and_tampered_payload`：1 passed。
- `npm test -- --run tests/map-rotation-contract.spec.ts tests/bot-plugin-gate.spec.ts`：4 passed。
- `npm run typecheck`：通过。
- `npm run build:web`：通过。
- 工作区既有 `cargo fmt` 已执行；其他历史脏改动未清理。

## 尚未完成

- 尚未补充组件级 `map-rotation-settings.spec.ts` 和 Rust MapRotation 专项单元测试。
- 尚未在真实 CS2 目录验证 enabled=false 后 BOT 启动、运行时命令不回写 JSON、15 秒自动换图及 `lbtv_map_next`。
- 本轮未重新构建安装器；上一轮未签名 NSIS 候选仍需在资源更新后重新打包。
- 未取得四种视口截图，状态不能写为正式发布或彻底修复。
