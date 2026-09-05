# 自动换图控件布局与 MapRotation 可变配置摘要冲突修复交接方案

日期：2026-08-29  
工作区：`E:\CS2AS05`  
角色：方案制定 AI 交给实际执行 AI  
状态：调查与方案；本轮未直接修改业务代码、未重新构建安装器

## 1. 用户现场与完成目标

本轮有两个已确认问题，必须分别修复并分别验收：

1. 概览页“自动换图默认状态”控件布局失真：截图 `C:\Users\GOPtZ\Pictures\Screenshots\屏幕截图 2026-08-29 111801.png` 显示长路径直接撑开内容，开关脱离卡片右边界，标题/说明/路径与开关没有稳定的列布局。
2. 用户将自动换图关闭后启动 BOT，出现：

```text
[BOT_PLUGIN_AUTO_INSTALL_FAILED]
[BOT_PLUGIN_PAYLOAD_INVALID] payload 摘要不匹配
期望 76D9C84D74368DF6727435EAE0C830C7F75056E1F6E0AB14C128CD43DC9EF7
实际 5C8A494F228031512F854FFC4F27B9C6082243C137220C82400923C46068FB12
expectedVersion=0.5.10
installedVersion=invalid
zipSha256=FC8868ABD46056DA52540EB14F3BA0B1B36005928C44178BC479C3BD2A4AA64E
```

目标结果：

- 控件在桌面、窄桌面和移动宽度下始终在父容器内；路径可换行/截断，不得推动开关越界；开关与标题区保持明确的两列关系。
- `MapRotation.json` 是用户可变配置。用户设置 `enabled=false` 后，BOT 自动安装仍应通过插件完整性校验；不得要求把用户配置恢复为 true 才能启动。
- 不降低插件版本门禁、不关闭 payload 校验、不删除 `MapRotation.json`，而是修正不可变 payload 与可变配置的边界。

## 2. 当前真实代码与证据

### 2.1 控件结构

- `src/components/MapRotationDefaultControl.vue` 当前根节点为 `.control-group.map-rotation-control`。
- 标题、说明、`<code>{{ state.configPath }}</code>` 和恢复按钮都在 `.map-rotation-heading` 内；`ToggleSwitch` 是标题区之后的独立兄弟节点。
- 截图中的路径为很长的 Windows 路径，当前 CSS 没有确保 `min-width:0`、`overflow-wrap:anywhere`、稳定的开关列和窄屏断点，因此长文本把布局撑破。
- `src/views/OverviewView.vue` 是唯一入口；不得把入口移到或新增到 `src/views/InstallView.vue`。

### 2.2 摘要冲突的确定性根因

- `scripts/generate-plugin-manifest.ps1::Get-Payload()` 当前把 `MapRotation.json`（`$mapRotationConfigEntry`）纳入 `payloadSha256`。
- `src-tauri/src/services/cs2.rs::payload_digest_from_files()` 按 marker 的 `payloadEntries` 对安装后目录重新计算摘要。
- 安装事务对 MapRotation JSON 有“目标存在则保留”语义，目的是保留用户的 `enabled=false`。
- 因此：ZIP marker 是按默认 JSON `{"enabled":true}` 生成，用户设置 false 后安装事务保留 false，安装后重新计算必然得到不同摘要。用户提供的 expected/actual 差异与这一流程完全一致。
- 当前 `verify_custom_zip()` 只验证 ZIP 自身 hash/结构，安装后 `inspect_bot_plugin_version_at()` 再验证 payload；错误发生在后者，不是 ZIP 下载损坏的充分证据。

## 3. 摘要模型修复方案（优先级 P0）

### 3.1 选择并固定边界

采用“不可变 payload 与可变配置分离”的模型：

- `payloadSha256/payloadEntries` 只包含插件 DLL、BotVision 文件、固定配置/数据文件；**排除** `addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`。
- marker 增加 `mutableConfigEntries`，当前至少包含上述 MapRotation JSON；也可增加 `mutableConfigSchema` 或 `configSha256`（仅记录格式 schema，不把用户值纳入不可变 digest）。
- MapRotation JSON 单独通过 `validate_map_rotation_config()` 校验：文件存在时必须是 JSON object，`enabled` 缺失按 true，存在时必须是 bool；损坏文件标记为 `fallback`/warning，但不冒充 payload 损坏。
- 不能简单从 `payloadEntries` 删除文件却不更新 marker：必须按同一规范重新计算 marker，并同步 ZIP 内 marker、Rust 内置 ZIP hash 常量和相关 fixture。

### 3.2 生成脚本修改

修改 `scripts/generate-plugin-manifest.ps1`：

1. 从 `$entries` 的 payload 过滤条件中移除 `$mapRotationConfigEntry`。
2. 增加 `mutableConfigEntries = @($mapRotationConfigEntry)`，保持路径使用正斜杠且唯一。
3. 生成后立刻在临时副本中回读 marker，断言 MapRotation JSON 不在 `payloadEntries` 且在 `mutableConfigEntries`。
4. 把脚本输出的 marker JSON、payload entry 列表、payload digest 和 ZIP SHA-256 写入 `workspace/runtime/plugin-manifest/result.json`；旧 `.bak` 只作为恢复材料，不提交。

### 3.3 Rust 校验修改

修改 `src-tauri/src/services/cs2.rs`：

- `PluginMarker` 增加 `mutable_config_entries: Vec<String>`（serde 默认空数组，兼容旧 marker 但旧 marker 需要在构建资源时更新）。
- `inspect_bot_plugin_version_at()` 校验：
  - `payload_entries` 不得包含 MapRotation JSON；
  - `mutable_config_entries` 必须包含 MapRotation JSON 且路径安全；
  - `payload_digest_from_files()` 只计算不可变 payload；
  - 单独读取 MapRotation JSON，调用共享解析/校验函数；`enabled=false` 合法，不能返回 `[BOT_PLUGIN_PAYLOAD_INVALID]`。
- `verify_custom_zip()` 同样校验 marker 的 payload/mutable 分组和 ZIP 中两类条目，防止资源包漏带配置。
- 错误分类：
  - DLL/固定文件缺失或摘要不符：`[BOT_PLUGIN_PAYLOAD_INVALID]`；
  - MapRotation JSON 损坏：`[MAP_ROTATION_CONFIG_INVALID]` 或结构化 warning，并回退默认开启；
  - 用户把配置设为 false：不是错误，安装后 marker 校验必须成功。
- 仍保留 `product/pluginId/schema/version` 精确检查，仍要求安装后版本等于 `0.5.10`，不得以放宽版本或跳过摘要解决问题。

### 3.4 迁移兼容

- 对已安装的旧 marker（其 `payloadEntries` 仍包含 MapRotation JSON），执行 AI 应在安装/更新事务中先识别旧 schema，完成一次受控迁移：保留现有 JSON 值，使用新 marker 覆盖固定资源，然后重新校验。
- 若旧 marker 的固定 payload 已被篡改，仍拒绝并回滚；不能因为配置可变而放宽其他文件校验。
- 迁移前后记录 `oldMarkerPayloadSha256/newMarkerPayloadSha256` 和 MapRotation JSON 的值（只记录 true/false，不记录无关用户路径）。

## 4. 控件布局修复方案（P1）

修改 `src/components/MapRotationDefaultControl.vue` 的结构，使设置区域成为稳定的两列布局：

- 外层 `.map-rotation-control` 使用 `width:100%; min-width:0; box-sizing:border-box; overflow:hidden`，但不要裁剪需要越界的其他页面内容。
- `.map-rotation-heading` 改为 `display:grid; grid-template-columns:minmax(0,1fr) auto; gap:12px; align-items:start`。
- 标题内容包裹层必须 `min-width:0`；`h2/p/small/code` 设置 `overflow-wrap:anywhere; word-break:break-word;`，长路径最多显示两行或使用 `text-overflow:ellipsis`，完整路径继续放在 `title`/可访问名称中。
- 恢复按钮固定在第二列；开关不要脱离父级。建议将 `ToggleSwitch` 放入 `.map-rotation-toggle-row`，同样为 `display:grid; grid-template-columns:minmax(0,1fr) auto; align-items:center`，标签文本允许换行，开关列宽度固定。
- 不使用 `position:absolute` 把开关定位到卡片外，不用负 margin，不用 `transform:scale` 解决布局。
- 在 `@media (max-width: 720px)` 下改为单列：标题、路径、开关依次换行；开关靠右但仍在容器内；确保最长中文和 Windows 路径不造成页面横向滚动。
- 浅色/深色/palette 使用现有 token；修复后截图中开关、边框、文字应与 `control-group` 内其他设置一致。

## 5. 前端行为与错误展示

- `MapRotationDefaultControl.vue` 保留现有唯一入口和 IPC：`get/set/reset_map_rotation_default`。
- 设置 false 成功后显示“下一次插件载入时保持当前地图”，不要显示错误状态。
- BOT 启动遇到 payload 错误时，显示实际固定 payload 校验错误；若只是 MapRotation JSON 非法，显示配置修复提示和“恢复默认开启”按钮。
- 不在安装与诊断页增加新的自动换图按钮；不把开关状态写入 localStorage。

## 6. 测试与验收门禁

### 6.1 自动化

- PowerShell manifest 测试：MapRotation JSON 在 `mutableConfigEntries`，不在 `payloadEntries`；marker payload 可重复生成。
- Rust：
  - 默认 JSON true 通过；用户 JSON false 通过；用户 JSON true 通过；缺失字段回退 true；损坏 JSON 分类为配置 warning；
  - 修改任意固定 DLL/数据仍返回 `[BOT_PLUGIN_PAYLOAD_INVALID]`；
  - 旧 marker 迁移后保留 false 且新 marker 校验通过；
  - 安装后 `expectedVersion=installedVersion=0.5.10`。
- 前端：
  - 组件在 1280px、980px、720px、375px 宽度下无横向溢出；
  - 长路径可换行/省略，开关和恢复按钮在父级边界内；
  - CS2 运行时禁用，false/true 切换 IPC 参数正确，状态不可乐观伪造。

建议命令：

```powershell
npm test -- --run tests/map-rotation-contract.spec.ts tests/map-rotation-settings.spec.ts tests/bot-plugin-gate.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml cs2 map_rotation panel
```

### 6.2 资源包回读

重新生成 ZIP 后，用 PowerShell `System.IO.Compression.ZipFile` 回读并记录：

- ZIP SHA-256；
- marker version/schema/product/pluginId；
- `payloadEntries` 和 `mutableConfigEntries`；
- payloadSha256；
- MapRotation JSON 默认内容；
- 固定资源 DLL 的 SHA-256。

必须证明：默认 JSON 改为 false 后，安装后固定 payload digest 不变，插件校验仍通过。

### 6.3 真实 Tauri/CS2

1. 在隔离 CS2 目录安装新候选，概览页确认控件完整位于框内。
2. 关闭自动换图，回读 JSON 为 `enabled=false`。
3. 选择 BOT 启动，确认不再出现 `payload 摘要不匹配`，插件 marker 版本为 0.5.10。
4. 执行 `lbtv_map_rotation`，确认默认关闭；执行 `lbtv_map_rotation 1` 只改变运行时状态，JSON 仍为 false。
5. 退出 CS2，改回 true，下一次插件载入默认开启。
6. 浅色/深色、1280x800、980x640、移动宽度截图；检查无横向滚动、无按钮越界、焦点可见。

## 7. 构建与停止条件

资源/代码校验未通过前不得构建安装器。修复通过后，按用户此前要求构建不注入私钥的 0.5.10 本地 NSIS 候选；报告必须标注 `unsigned/local validation only`，不得声称正式签名。

遇到 marker 仍把可变 JSON 纳入 payload、用户 false 仍触发摘要错误、开关靠绝对定位越界、资源包 hash 常量未同步或安装器引用旧 ZIP，立即停止并回到资源生成步骤。不得删除 payload 校验、降低版本比较、覆盖用户配置或清空整个 `backup`/插件目录。

交付物必须包含：变更文件绝对路径、旧/新 marker 摘要对账、ZIP/安装器 SHA-256、true/false 回读、测试结果、四种视口截图路径，以及真实 CS2 BOT 启动日志。未完成真实 CS2 验收前，状态写“修复候选/待用户验收”。
