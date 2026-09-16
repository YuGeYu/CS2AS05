# 只开换肤模式缺少 SkinOnly gameinfo 调查与修复方案

日期：2026-09-09  
版本目标：0.5.13 后续修复构建

## 用户反馈

选择“只开换肤”时失败：

```text
[PANEL_IO] 读取模式源文件失败：D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game/csgo\backup/SkinOnly/gameinfo.gi 系统找不到指定的路径。 (os error 3)
```

## 调查证据

1. 当前源码在 `src-tauri/src/services/panel.rs::write_mode_at()` 将 `skin_only` 映射为：
   `backup/SkinOnly/gameinfo.gi`。
2. 当前内置 `src-tauri/resources/CS2BotImprover.zip` 已包含该条目，且 `verify_custom_zip()` 将其列为必需资源；资源包本身不是缺失原因。
3. 安装事务 `src-tauri/src/services/cs2.rs::install_game_files_transactionally()` 只在执行“安装/更新 BOT 包”时解压 ZIP。应用升级不会自动对每个既有 CS2 目录重新执行该事务。
4. 玩家实际目录存在：
   - `game/csgo/backup/Online/gameinfo.gi`
   - `game/csgo/backup/WithBots/gameinfo.gi`
   - `game/csgo/overrides/...`
   但不存在 `game/csgo/backup/SkinOnly/gameinfo.gi`。
5. 因此这是“新增资源未向既有安装执行迁移”的兼容性缺陷，而不是路径大小写、Steam 库盘符或玩家 BOT 难度文件损坏。

## 根因

0.5.13 把 SkinOnly 文件加入了发布 ZIP，并在运行时直接读取目标目录中的该文件，但没有在切换模式前执行“资源存在性与摘要迁移”。对于已经安装过旧版本资源包的玩家，目标目录仍只有旧版三状态布局，`set_mode("skin_only")` 直接读文件必然返回 `os error 3`。

## 修复目标

- 既有 0.5.12/早期安装无需用户手动卸载重装，即可首次选择“只开换肤”。
- 只在缺失或摘要不匹配时补齐 `backup/SkinOnly/gameinfo.gi`，不改动玩家现有 `overrides`、`botprofile.vpk`、自定义难度和活动 `gameinfo.gi`。
- 补齐后仍通过 manifest/sidecar 摘要校验，再允许切换模式。
- 安装/更新事务继续保持原子写入和失败回滚。
- 保留 Inventory Simulator 检查和 BOT 插件隔离逻辑。

## 推荐实现

### 1. 新增按需资源迁移函数

在 `src-tauri/src/services/cs2.rs` 增加面向既有安装的 `ensure_skin_only_gameinfo(app, root_path)`（或等价内部函数）：

- 解析内置 ZIP 并执行现有 `verify_custom_zip()`。
- 从 ZIP 读取 `backup/SkinOnly/gameinfo.gi` 到内存，校验 manifest 中的 SHA-256 和长度。
- 目标路径固定为 `game/csgo/backup/SkinOnly/gameinfo.gi`。
- 若目标文件已存在且摘要一致，直接返回 `unchanged`。
- 若目标缺失或摘要不一致，先写入同目录临时文件，再原子替换；必要时保留旧文件的受控备份。
- 不调用完整 BOT 包安装事务，避免覆盖其它受管文件和触发不必要的插件更新。
- 写入后再次读取并校验摘要，失败返回明确错误码，例如 `[SKIN_ONLY_GAMEINFO_MIGRATION_FAILED]`。

### 2. 在所有 SkinOnly 入口前调用迁移

至少覆盖：

- `panel::set_mode(root_path, "skin_only")`
- `launch_cs2_inner(..., "skin_only")`

调用顺序建议为：退出状态检查 → 规范化 CS2 根目录 → Inventory Simulator 存在性检查 → `ensure_skin_only_gameinfo()` → `write_mode_at()` → 插件隔离 → 保存偏好。

这样即使用户从旧版直接升级，也会在第一次使用功能时自动完成兼容迁移。

### 3. 修复状态识别与提示

- `gameinfo_state()`、`snapshot_at()` 对缺少 SkinOnly 文件时仍允许 `online`/`bots` 正常显示，不把旧安装误判为整体恢复失败；只有用户请求 SkinOnly 时才执行迁移。
- 若内置资源无法读取或 manifest 不匹配，提示“请重新安装 0.5.13 或在安装页更新资源”，不要显示底层 `os error 3`。
- 迁移成功后重新生成/补齐 sidecar 中的 `skin_only_sha256`，避免旧 sidecar 因可选字段缺失而持续显示恢复状态。

### 4. 保持玩家文件边界

- 不删除或改写 `overrides/Low|Medium|High` 和活动 `overrides/botprofile.vpk`。
- 不复制玩家当前 BOT 难度到 SkinOnly；SkinOnly gameinfo 只负责移除 `csgo/overrides/botprofile.vpk` 的原生加载入口。
- 仍由 `set_skin_only_plugins()` 隔离 CS2AS05 BOT 插件，切回 `bots`/`online` 时恢复受管目录。

## 测试方案

### Rust 单元/集成测试

新增或扩展临时 CS2 根目录夹具：

1. 仅创建 `backup/Online`、`backup/WithBots`，不创建 `backup/SkinOnly`；调用 SkinOnly 迁移，断言目录和文件被创建、摘要正确。
2. SkinOnly 文件已存在且摘要正确，断言迁移幂等且内容不变。
3. SkinOnly 文件存在但内容错误，断言被原子替换为内置摘要，且不触碰 `overrides`。
4. 模拟 ZIP 缺失/manifest 摘要错误，断言返回 `[SKIN_ONLY_GAMEINFO_MIGRATION_FAILED]` 或对应资源校验错误，不写入半成品。
5. 迁移失败后断言活动 `gameinfo.gi`、Online/WithBots 备份和玩家难度文件保持原样。

### 静态契约检查

- ZIP 必须包含 `backup/SkinOnly/gameinfo.gi`。
- manifest、`CUSTOM_ZIP_SHA256`、SkinOnly SHA-256 和测试夹具保持一致。
- `cargo check`、`cargo fmt --check`、`npm run typecheck`、`git diff --check` 通过。

### 真实 Windows/CS2 验收

- 使用已安装 0.5.12/旧资源目录直接升级到修复版，不重新安装 BOT 包。
- 选择“只开换肤”成功，确认 `backup/SkinOnly/gameinfo.gi` 自动出现。
- 启动后 Inventory Simulator 可用，选定 BOT 难度不改变 SkinOnly 行为。
- 创意工坊闯关地图中 BOT 全灭后下一关正常推进。
- 切回 BOT 模式后原有 BOT 插件、难度和玩家配置恢复。

## 发布边界

本方案只解决 SkinOnly 模式资源向既有安装的兼容迁移。修复完成并重新签名构建前，不应把当前 0.5.13 候选包标记为已解决该报错；真实 CS2 验收仍是发布结论的必要条件。
