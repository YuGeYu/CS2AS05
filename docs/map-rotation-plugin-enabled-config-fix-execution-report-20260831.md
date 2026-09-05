# MapRotation 插件 enabled 配置解析修复执行报告

日期：2026-08-31  
工作区：`E:\CS2AS05`  
状态：插件修复候选，代码/资源验证完成，真实 CS2 重启待验收。

## 根因与修改

助手写入的是小写 JSON 键 `enabled`，而原插件使用 `System.Text.Json` 反序列化到大写属性 `Enabled`。默认大小写敏感时，小写键不会绑定，属性初始值保持 `true`，导致用户写入 false 后插件仍以 enabled=1 启动。

已修改：

- `third_party/CS2-Bot-Improver-map-rotation/addons/counterstrikesharp/plugins/MapRotation/MapRotation.cs`
  - 新增严格 `ParseDefaultConfig(ReadOnlySpan<byte>)`。
  - 仅根对象且存在小写 `enabled` 且值为 JSON boolean 时视为有效。
  - 缺失、字符串、数字、null、数组和损坏 JSON 分别进入明确 reason：`enabled-missing`、`enabled-not-boolean`、`root-not-object`、`invalid-json`。
  - 无效配置记录 `[MAP_ROTATION_CONFIG_INVALID]`，带 canonical 根目录、绝对路径、SHA-256、原因和 fallback enabled=1。
  - 有效配置记录 `[MAP_ROTATION_CONFIG_READ]`，带 canonical 根目录、绝对路径、SHA-256、`enabled=0|1`、`source=config`。
  - 保持单次 `Load()` 初始化、运行时命令只改内存、`_enabled` 禁用门控、`lbtv_map_next` 显式换图能力不变。
- `tests/map-rotation-plugin-config.spec.ts`
  - 新增严格小写 schema、失败可见性、单次加载和命令边界静态契约。

## 构建与资源

- MapRotation Release DLL 构建：0 warnings / 0 errors。
- 当前 DLL SHA-256：`41DABB3BB551323E9A655D8A5419B8B0624AFBB024C84354493A0345DAA76FF3`
- `add-map-rotation.ps1` 已将 DLL 写入 `src-tauri/resources/CS2BotImprover.zip`。
- 写入后 ZIP SHA-256：`AD908DA8B6B0F7517599C364022957DA49327858A4EE2889F89DB90D6D7C5B0A`。
- 随后 manifest 重新生成：
  - `payloadSha256`：`11FEA4E1A5AED71A0C0722DF58682BFA5703121AE4620B5A1BCC4C5F88304085`
  - 最终 ZIP SHA-256：`0C53DFC60A7FF4DAB446285233AD99BDDFD30EAAD36053802DA01D1048F85611`
  - `mutableConfigEntries` 仍仅包含 `addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`。
- 写前备份：`src-tauri/resources/CS2BotImprover.zip.before-map-rotation-20260831-182751.bak`、`src-tauri/resources/CS2BotImprover.zip.before-plugin-manifest-20260831-182813.bak`。

## 自动化结果

- `dotnet build ...MapRotation.csproj -c Release --nologo`：通过，0 warnings / 0 errors。
- `npm test -- --run tests/map-rotation-contract.spec.ts tests/map-rotation-plugin-config.spec.ts`：4 tests passed。

## 待真实验收

当前未运行 CS2/BOT，未取得安装后实际 DLL 路径/SHA、插件 Load 日志、`lbtv_map_rotation` false/true 两次重启回读、自动换图阻断/恢复日志及运行中命令不回写 JSON 的 bytes/SHA 证据。因此不能宣称真实游戏链路已完成；用户应按方案在 CS2 完全退出后分别验证 `{"enabled":false}` -> `enabled=0`、`{"enabled":true}` -> `enabled=1`，并确认 false 时不出现自动 `Scheduling next map`。
