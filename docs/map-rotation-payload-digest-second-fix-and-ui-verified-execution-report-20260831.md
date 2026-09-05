# 自动换图 Payload 摘要第二阶段修复执行报告

日期：2026-08-31
状态：第二阶段修复候选，待真实 CS2 验收

## 结论

已确认并修复旧候选资源仍在运行的问题：当前构建输入、manifest 报告和 Rust `CUSTOM_ZIP_SHA256` 已统一为同一份新 ZIP。MapRotation JSON 现在属于可变配置，不参与固定 payload 摘要；固定插件文件仍严格校验。

## 四段输入对账

- 源码 ZIP：`E:\CS2AS05\src-tauri\resources\CS2BotImprover.zip`
- 源码 ZIP SHA-256：`8C9D9315A79EDA0428598659DFBB1A8F02ACA7A5D937E2B545EE97241F69FA89`
- `workspace/runtime/plugin-manifest/result.json` 的 ZIP SHA-256：同为 `8C9D9315A79EDA0428598659DFBB1A8F02ACA7A5D937E2B545EE97241F69FA89`
- Rust `CUSTOM_ZIP_SHA256`：同为 `8C9D9315A79EDA0428598659DFBB1A8F02ACA7A5D937E2B545EE97241F69FA89`
- marker version：`0.5.10`
- marker payloadSha256：`6880C768E3C1BA42E47E78B57D38AB8C3AF761FBEA2B3E8F1D54D110B357FFB0`
- `mutableConfigEntries`：`addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`
- `payloadEntries`：不包含 MapRotation JSON，仍包含 MapRotation DLL、NadeSystem、BotVision 等固定文件。

旧错误中的 `FC8868...` ZIP 和 `76D9...` payload 属于旧候选，不是本次构建输入。当前 NSIS 内嵌 ZIP 未能从压缩安装器二进制直接字符串回读，需在目标隔离目录实际安装后完成最终目标目录对账。

## 代码与 UI

- manifest 生成脚本排除 MapRotation JSON 并写入 `mutableConfigEntries`。
- Rust marker 解析支持 mutable 分组；安装目录保留用户 `enabled=false/true`，固定 payload 摘要继续校验。
- 概览页控件使用稳定 Grid 两列、`min-width:0`、长路径换行/截断和 720px 单列断点；唯一入口仍位于 `OverviewView.vue`。

## 自动化验证

- `cargo test --manifest-path .\\src-tauri\\Cargo.toml cs2::tests::plugin_marker_reports_missing_invalid_and_tampered_payload`：1 passed。
- `npm test -- --run tests/map-rotation-contract.spec.ts tests/bot-plugin-gate.spec.ts`：4 passed。
- `npm run typecheck`：通过。
- `npm run build:web`：通过。

## 未签名安装器

- 路径：`E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.10_x64-setup.exe`
- 大小：`117213596` bytes
- SHA-256：`150A8C77C40F2AE0E764F7EE80BBC55C2F114DC7601F2F421F77E74E36103CFB`
- Authenticode：`NotSigned`
- 构建前清除了 `TAURI_SIGNING_PRIVATE_KEY`、`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`、`TAURI_SIGNING_PRIVATE_KEY_PATH`。NSIS 文件已生成，但 Tauri 最终 updater 签名阶段因配置存在公钥而无私钥返回非零；无可用 `.sig`。该文件标记为 `unsigned/local validation only`，未上传、未提交、未发布。

## 待真实验收

- 尚未安装该最新候选到隔离 CS2 目录，因此尚无目标目录 marker/fixed payload/MapRotation JSON 回读。
- 尚未取得 false/true 两次真实 BOT 启动结果、插件日志或四种视口截图。
- 尚未完成 `lbtv_map_rotation` 不回写 JSON、15 秒自动换图和 `lbtv_map_next` 立即换图验证。
