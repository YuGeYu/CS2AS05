# 自动换图开启仍无法启动：Payload 摘要第二阶段修复交接方案

日期：2026-08-29  
工作区：`E:\CS2AS05`  
角色：方案制定 AI 交给实际执行 AI  
状态：调查完成，尚未在本轮直接修改业务代码或构建安装器

## 1. 本次现场结论

用户已将自动换图重新开启，但 BOT 启动仍失败：

```text
[BOT_PLUGIN_AUTO_INSTALL_FAILED]
[BOT_PLUGIN_PAYLOAD_INVALID] payload 摘要不匹配
期望：76D9C84D74368DF6727435EAE0C830C7F75056E1F6E0AB14C128CD43DC9EF7
实际：AA255EBE7F4186CF5C35627455E80CCE33D130E8AFA057420F6A99C7859EF72C
expectedVersion=0.5.10
installedVersion=invalid
zipSha256=FC8868ABD46056DA52540EB14F3BA0B1B36005928C44178BC479C3BD2A4AA64E
markerPath=D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\addons\counterstrikesharp\plugins\NadeSystem\CS2AS05.plugin.json
```

这证明问题已经不再取决于 `MapRotation.json` 的 true/false：即使开启自动换图，BOT 安装仍在固定 payload 校验阶段失败。

## 2. 当前源码与错误包的对账证据

在当前 `E:\CS2AS05` 工作树只读回读 `src-tauri/resources/CS2BotImprover.zip` 得到：

- 当前 ZIP SHA-256：`8C9D9315A79EDA0428598659DFBB1A8F02ACA7A5D937E2B545EE97241F69FA89`；
- marker version：`0.5.10`；
- 当前 marker `payloadSha256`：`6880C768E3C1BA42E47E78B57D38AB8C3AF761FBEA2B3E8F1D54D110B357FFB0`；
- marker 已有 `mutableConfigEntries`，且包含 `addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`；
- 当前 `payloadEntries` 不再包含 MapRotation JSON。

而用户错误中的 ZIP SHA-256 是 `FC8868...`、期望 payload 是 `76D9...`，与当前工作树资源不同。这是**旧安装器/旧内置 ZIP 仍在运行，或安装器资源与源码资源不同步**的直接证据。实际 `AA255...` 说明安装后固定 payload 也与该旧 marker 不一致，必须先确认运行中的 EXE 实际嵌入资源，不能继续猜测配置文件。

上一阶段虽然已经在源码中加入 `mutableConfigEntries` 和排除逻辑，但旧安装器未重新构建，且当前 Rust 资源常量/ZIP 可能在不同时间点被更新。执行 AI 必须以“构建输入 ZIP -> 编译时嵌入 ZIP -> 安装器 -> 目标目录”四段 hash 对账，不能只看工作树文件。

## 3. P0：统一并验证 payload 规范

### 3.1 固定不可变/可变边界

- `payloadEntries` 只包含固定 DLL、PDB、BotVision 文件、NadeSystem 固定 grenade JSON 等；不得包含 `MapRotation.json`。
- `mutableConfigEntries` 必须包含 `addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`。
- `MapRotation.json` 的 true/false、格式化空格和换行都不能影响 `payloadSha256`。
- `inspect_bot_plugin_version_at()` 继续严格校验固定 payload、product、pluginId、schema 和精确版本 0.5.10；不得删除或放宽校验。
- 运行时对 MapRotation JSON 只做独立 schema/布尔校验；损坏配置返回配置 warning/fallback，不伪装成 payload 摘要错误。

### 3.2 生成脚本与 Rust 算法必须同源

核对并修复 `scripts/generate-plugin-manifest.ps1::Get-Payload()` 与 `src-tauri/src/services/cs2.rs::payload_digest_from_files()`：

1. 两边必须使用完全相同的 entry 过滤集合、正斜杠路径、排序、`path\0length\0bytes` 拼接顺序和 UTF-8 编码。
2. 增加一个跨语言 golden fixture：由 PowerShell 生成 marker 后，Rust 测试对同一解压目录计算出相同 `payloadSha256`。
3. 脚本回读 marker，断言 MapRotation JSON 只在 `mutableConfigEntries`；Rust 测试断言同样条件。
4. 不得用 `verify_custom_zip` 中的固定 `CUSTOM_ZIP_SHA256` 掩盖资源变化；每次资源更新必须同步该常量，且通过脚本自动生成，避免手工复制旧 hash。

### 3.3 安装迁移

- 安装事务必须覆盖固定 payload 和新 marker；对 MapRotation JSON 继续“目标存在则保留”。
- 目标目录已有 `enabled=false` 或 `enabled=true` 均应保留，并在新 marker 校验后通过。
- 对旧 marker 中误把 MapRotation JSON 列入 payload 的安装，先保存用户 JSON 值，再安装新固定资源/marker，最后恢复 JSON 并独立校验。
- 安装失败时回滚固定文件和 marker；用户 JSON 恢复失败必须报告路径，不得删除整个插件目录。

## 4. P0：构建输入与运行安装器四段对账

执行 AI 必须停止使用现有旧候选安装器，按以下步骤生成新未签名候选：

1. 记录 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 版本均为 0.5.10。
2. 运行 `scripts/generate-plugin-manifest.ps1`，保存 `workspace/runtime/plugin-manifest/result.json`。
3. 记录生成后 ZIP hash、marker JSON、payload hash、payload/mutable entry 集合。
4. 编译前检查 `src-tauri/resources/CS2BotImprover.zip` 与 manifest 报告 hash 相同。
5. 构建 Tauri 时确认 `tauri.conf.json` 的 resources 仍指向 `resources/CS2BotImprover.zip`，没有从 `target`、旧安装目录或缓存复制同名 ZIP。
6. 构建后从 `src-tauri/target/release/bundle/nsis` 安装器中解包/定位内置资源，回读其中 ZIP hash 和 marker；必须与第 3 步一致。
7. 安装到隔离测试目录后，再回读目标 `CS2AS05.plugin.json`、固定 payload digest 和 MapRotation JSON；四者必须形成以下关系：

```text
源码 ZIP hash == 安装器内置 ZIP hash
安装器 marker payloadSha256 == 安装后固定 payload digest
MapRotation.json 不参与 payload digest
expectedVersion == installedVersion == 0.5.10
```

若安装器仍嵌入 `FC8868...` 或 marker 仍为 `76D9...`，说明构建使用旧资源，立即停止，不得让用户继续验证。

## 5. UI 布局复核（继承上一方案）

`src/components/MapRotationDefaultControl.vue` 和 `src/styles/main.css` 已有初步 Grid 修复，但必须按截图重新验收：

- 外层和内容列 `min-width:0`；
- 路径 `overflow-wrap:anywhere`/`word-break:break-word`，不撑宽父级；
- 标题内容列与开关列分离，恢复按钮固定在操作列；
- 720px 以下单列，开关仍在框内；
- 不使用绝对定位、负 margin 或 transform 把控件塞回去；
- 不在安装与诊断页新增入口，唯一入口仍是 `OverviewView.vue`。

## 6. 测试与实机门禁

### 自动化

```powershell
npm test -- --run tests/map-rotation-contract.spec.ts tests/map-rotation-settings.spec.ts tests/bot-plugin-gate.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml cs2 map_rotation panel
```

必须补齐 `tests/map-rotation-settings.spec.ts` 和 Rust map_rotation 专项测试；当前报告显示这些覆盖仍缺失，不能把“组件已存在”当作验收通过。

### 真实功能

1. 在隔离目录安装新候选，回读安装器内置 ZIP 与目标 marker hash 一致。
2. 概览页关闭自动换图，JSON 为 false；BOT 启动成功且 payload 校验通过。
3. 概览页开启自动换图，JSON 为 true；BOT 启动同样成功。
4. 运行中执行 `lbtv_map_rotation 0|1` 只改内存，不改 JSON。
5. 固定 payload 任意文件被改写时仍可靠失败；仅改 MapRotation JSON 不得失败。
6. 完成最终比分后约 15 秒自动换图，`lbtv_map_next` 仍立即执行。

## 7. 无私钥安装器构建边界

用户只需功能验证，不注入任何私钥：

```powershell
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY -ErrorAction SilentlyContinue
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PATH -ErrorAction SilentlyContinue
npm run bundle:desktop
```

如 updater 公钥配置导致最终签名阶段返回非零，但 NSIS 已生成，必须验证安装器内部资源后再交付，并在报告中标记 `unsigned/local validation only`、Authenticode `NotSigned`、无可用 `.sig`。不允许填假私钥或将旧安装器改名冒充新包。

## 8. 停止条件与交付物

停止条件：四段 hash 任一不一致、MapRotation JSON 仍出现在 payloadEntries、false 配置仍触发摘要错误、安装后 marker 版本不是 0.5.10、安装器来源无法证明、或 UI 在任一视口越界。

最终交付必须更新执行报告，包含：

- 源码 ZIP、安装器内置 ZIP、目标目录三处 hash；
- 新旧 marker/payload 对账及 mutable entry；
- true/false 两次 BOT 启动结果；
- 组件截图和视口尺寸；
- 测试命令结果；
- 安装器绝对路径、大小、SHA-256、未签名说明。

在这些证据齐全前，状态只能写“第二阶段修复候选/待真实 CS2 验收”，不能宣称问题已解决。
