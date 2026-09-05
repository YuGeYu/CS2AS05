# CS2 人机增强助手 0.5.10 UI 临时下线版签名构建记录

日期：2026-08-31  
版本：0.5.10

## 本轮发布内容

- 概览页移除 `MapRotationDefaultControl` 挂载，启动时不再因该卡片后台调用 MapRotation IPC。
- 主题设置保留“自动换图默认状态（开发中）”入口，打开只读说明模态框。
- 说明模态框包含明确的“当前暂时无效”状态、ARIA 对话框语义、关闭按钮、Esc 和背景点击关闭；不提供 Toggle、保存、重置或配置写入。
- MapRotation 插件逻辑、助手写入逻辑、自动换图延迟、`lbtv_map_rotation 0|1`、`lbtv_map_next` 和配置解析均未修改。

## 验证

- `npm test -- --run tests/map-rotation-card-visibility.spec.ts tests/appearance-preferences.spec.ts tests/ui-design-contract.spec.ts tests/map-rotation-contract.spec.ts tests/map-rotation-plugin-config.spec.ts`：5 个文件、14 项测试通过。
- `npm run typecheck`：通过。
- `npm run bundle:desktop`：通过并生成 NSIS 安装器与 updater `.sig`。

## 最终签名产物

```text
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.10_x64-setup.exe
size: 117414932 bytes
sha256: B1B3E09996A50245DF766D1496A55438E973787D348909F794ED66592B74ED4C

E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.10_x64-setup.exe.sig
size: 436 bytes
sha256: 7C92807587447A0AE2F21CCC08866AD51B6FBA9F8917BC7973063C3B940A450C
```

DPAPI 内容先 `Trim()` 后解密；私钥内容注入 `TAURI_SIGNING_PRIVATE_KEY`，密码注入 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。构建结束后两个变量均确认不存在。Windows Authenticode 为 `NotSigned`，与 Tauri updater `.sig` 分开记录。

本地 updater manifest：`E:\CS2AS05\dist-release\cs2-bot-improver\updater-prod.json`。

## 发布边界

本轮未执行 R2/D1、官网、GitHub、夸克上传，未提交/推送，未覆盖安装用户环境。真实桌面多视口验收、真实 CS2 重启后的插件状态回读仍需用户执行；当前状态为本地最终签名候选。
