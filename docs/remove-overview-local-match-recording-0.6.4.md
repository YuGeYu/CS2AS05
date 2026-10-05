# 删除概览页本地对局记录功能

日期：2026-10-05
适用版本：`CS2 人机增强助手 0.6.4`

## 变更范围

核心工作台的“概览”页面不再展示“本地对局记录”卡片、自动录制状态摘要或录制开关。页面只保留启动模式、BOT 难度、目录环境和启动 CS2 等核心操作，启动台说明也改为“本地状态和启动入口”。

本次删除的是助手自动录制能力：不再通过 Panel 配置写入 `tv_enable` / `tv_autorecord`，移除了 `get_demo_settings`、`set_demo_recording_enabled` 两个 Tauri 命令、前端 Demo 录制设置状态和对应 Rust 录制状态模型。

“对局复盘”页面及手动导入已有 `.dem` 文件的能力仍保留。这样玩家仍可查看已有录像和报告，但助手不会因为启动 BOT 对局而主动创建本地 Demo。

## 旧版本迁移

旧版本可能在 `cfg/my_bot_normal_config.cfg` 或 `cfg/my_bot_ffa_config.cfg` 中留下由助手写入的完整标记块。切换模式时会只删除以下明确标记范围：

```text
// CS2AS05 DEMO RECORDING BEGIN
...
// CS2AS05 DEMO RECORDING END
```

标记范围之外玩家自行配置的 `tv_*` 行会被保留。迁移逻辑包含单元测试，避免误删用户配置。

## 验证边界

- 概览页面契约改为断言录制卡片、录制摘要和 `recordingEnabled` 均不存在。
- Rust Panel 单元测试覆盖旧录制标记清理和玩家 `tv_*` 配置保留。
- 后续应运行前端验证、Rust 测试和一次桌面构建；真实 CS2 验收只需确认启动 BOT 后不会新增助手录制配置，已有 Demo 仍能在“对局复盘”中手动导入。
