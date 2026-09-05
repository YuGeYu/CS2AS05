# CS2 人机增强助手 0.5.10 MapRotation 修复版发布记录

日期：2026-08-31  
版本：0.5.10

## 正式更新日志

更新日志已写入 `docs/release-notes-0.5.10.md`，按【新增】、【优化】、【修复】、【其他】分类，采用正式发布口径。新增修复项为：MapRotation 严格识别助手写入的下标 `enabled` JSON boolean，确保 `{"enabled":false}` 在下一次插件加载后得到 `enabled=0`，无效配置进入明确 fallback 并记录原因。

## 资源对账

- MapRotation Release DLL SHA-256：`41DABB3BB551323E9A655D8A5419B8B0624AFBB024C84354493A0345DAA76FF3`
- 内置 ZIP SHA-256：`0C53DFC60A7FF4DAB446285233AD99BDDFD30EAAD36053802DA01D1048F85611`
- marker：`version=0.5.10`
- 固定 payload SHA-256：`11FEA4E1A5AED71A0C0722DF58682BFA5703121AE4620B5A1BCC4C5F88304085`
- `mutableConfigEntries` 仅包含 `addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`
- Rust `CUSTOM_ZIP_SHA256` 与前端契约测试均已同步最终 ZIP 摘要。

## 验证

- MapRotation `dotnet build -c Release --nologo`：0 warnings / 0 errors（用户提供的插件侧结果）。
- MapRotation 前端契约：4 tests passed。
- 本轮新增/修复后的前端与安装契约：11 tests passed。
- Rust `performance_radar_query_enforces_demo_keys_limit_and_stable_order`：1 passed；测试内存数据库已补齐评分列及显式插入字段。
- 此前完整 `npm run verify`：45 个文件、162 项测试通过；本轮改动未触及业务验证失败项。

## 最终签名产物

```text
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.10_x64-setup.exe
size: 117407027 bytes
sha256: 88A37CA04BC42FE925B219A9B6F1D09D1861DA67DCD0F80E34FB0A4480EE7095

E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.10_x64-setup.exe.sig
size: 436 bytes
sha256: B09B8DF2DD75E854E99527C26170DD82CF71CE1DAC6F82177224C6A39095D8B0
```

构建时使用 DPAPI 内容 `Trim()` 后的值解密密码，并将私钥内容注入 `TAURI_SIGNING_PRIVATE_KEY`；构建结束后已确认私钥和密码环境变量均不存在。未设置 `TAURI_SIGNING_PRIVATE_KEY_PATH` 作为唯一签名入口。

本地 updater manifest：`E:\CS2AS05\dist-release\cs2-bot-improver\updater-prod.json`，已重新生成并包含最终安装器 SHA-256/大小。

Windows Authenticode 回读为 `NotSigned`；它与 Tauri updater `.sig` 分属两套机制。

## 发布边界

本轮未上传 R2/D1、官网、GitHub 或夸克，未提交/推送，未覆盖安装用户环境。真实 CS2 重启后 `enabled=0/1`、禁用时不自动换图、启用时按 15 秒延迟换图仍需用户在真实环境验收；当前状态为本地最终签名候选。
