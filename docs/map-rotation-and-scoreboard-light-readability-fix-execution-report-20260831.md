# MapRotation 默认状态与亮色本局战报修复执行报告

日期：2026-08-31  
工作区：`E:\CS2AS05`  
状态：第二阶段修复候选，代码验证完成，真实 CS2/独立战报验收待执行。

## 已修改

- `src-tauri/src/services/map_rotation.rs`
  - DTO 新增 `configSha256`、`readBackEnabled`、`observedAt`、`loadSemantics`。
  - 写入后立即读取同一绝对路径并计算 SHA-256；运行中仍拒绝修改。
  - `[MAP_ROTATION_DEFAULT_UPDATED]` 日志包含 enabled、绝对路径、SHA、回读状态和 `next-plugin-load` 语义。
- `src/types/mapRotation.ts`、`src/components/MapRotationDefaultControl.vue`
  - 对齐新 DTO，并在控制面板显示回读 enabled、SHA-256 和下一次插件载入提示。
- `third_party/CS2-Bot-Improver-map-rotation/addons/counterstrikesharp/plugins/MapRotation/MapRotation.cs`
  - `Load()` 使用 `Path.GetFullPath` canonical 路径，按同一 bytes 计算 SHA-256，并记录根目录、绝对配置路径、摘要和解析结果。
- `src/styles/scoreboard.css`
  - 为 `html/body/#scoreboard-app/.scoreboard-shell` 建立显式深色前景、背景、`opacity:1`、`visibility:visible` 基线。
  - 标题、地图/比分摘要、表格、表头和单元格显式使用 `--scoreboard-text`，保留主题、调色板、密度和表格滚动行为。

## 清单与资源

已执行 `scripts/generate-plugin-manifest.ps1`。当前输出：

- `payloadSha256`: `6880C768E3C1BA42E47E78B57D38AB8C3AF761FBEA2B3E8F1D54D110B357FFB0`
- `zipSha256`: `B5C101D44C799000BDE6A0E5ECE1A1291663CA666DB4017D562702FF4401E579`
- `mutableConfigEntries`: 仅 `addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`
- 脚本保留写前备份：`src-tauri/resources/CS2BotImprover.zip.before-plugin-manifest-20260831-123628.bak`

安装事务已有保护：目标存在时跳过 ZIP 内 MapRotation JSON，因此升级不会覆盖用户 false/true 配置。

## 自动化结果

- `npm test -- --run tests/map-rotation-contract.spec.ts tests/post-match-scoreboard.spec.ts`：4 tests passed。
- `npm run typecheck`：通过。
- `cargo test --manifest-path src-tauri/Cargo.toml map_rotation --lib`：通过（当前筛选到 0 个测试，crate 编译通过）。
- `dotnet build third_party/CS2-Bot-Improver-map-rotation/addons/counterstrikesharp/plugins/MapRotation/MapRotation.csproj -c Release --nologo`：0 warnings / 0 errors。

## 待真实验收

当前机器未运行 CS2/BOT，未获得插件 `Load()` 控制台日志、`lbtv_map_rotation 0|1` 两次重启回读、运行中不回写 JSON 的 bytes/SHA 证据；也未启动独立战报窗口，因此没有生成四张截图或 computed-style/对比度 JSON。不能据此宣称 VAC、实机 MapRotation 或亮色窗口验收完成。

用户验收时应保存：

1. 助手 DTO 的绝对路径、`readBackEnabled`、`configSha256`；插件 `[MAP_ROTATION_CONFIG_READ]` 的同路径/同 SHA 日志。
2. CS2 退出后 false/true 各一次重启，首次 `lbtv_map_rotation` 分别返回 `enabled=0`/`enabled=1`；运行中命令只改变内存。
3. 亮色 1280x800、980x640、375x800 与深色 1280x800 截图，页脚均包含 `LBRating 2.0 · lb-rating-2.0`，并回读关键节点 computed color/背景/opacity/visibility。
