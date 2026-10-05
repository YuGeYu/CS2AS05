# CS2 人机增强助手 0.6.4

【新增】
- 基于 CS2 BuildID `25640462` 重建官方 `gameinfo.gi` 资源基线，覆盖 Online、BOT 和只开换肤三种运行模式。
- 增加启动前的官方游戏层完整性检查，按当前 CS2 根 `gameinfo.gi` 实际声明动态校验依赖。

【优化】
- 优化新旧 CS2 游戏布局兼容性，支持新版将 `csgo_imported`、`csgo_core` 内容合并到根 `gameinfo.gi` 的官方结构。
- 优化资源清单校验，四份 `gameinfo.gi` 均记录大小和 SHA-256，减少错误资源被写入游戏目录的风险。
- 资源选择改为完整校验后再使用，旧安装缓存缺少 Panel v1.4.5 时会自动跳过并从当前版本内嵌官方资源修复缓存。
- 删除概览页“本地对局记录”卡片和助手自动录制逻辑；“对局复盘”仍支持手动导入已有 Demo，迁移说明见 [`docs/remove-overview-local-match-recording-0.6.4.md`](../remove-overview-local-match-recording-0.6.4.md)。

【修复】
- 修复 CS2 游戏更新后启动 BOT 模式可能出现 `Failed to load layered mod 'csgo_imported'` 的兼容性问题。
- 修复启动检查无条件要求历史核心层目录、导致新版完整游戏被错误拦截的问题。
- 修复旧版 `CS2BotImprover.zip` 被误选导致 `[PANEL_ASSET_INVALID]` 的问题；现在严格要求官方 `Panel v1.4.5.exe`，并兼容上游 ZIP 的 `./` 条目路径。

【其他】
- 更新版本至 `0.6.4`；真实游戏内 BOT、Online 与只开换肤效果请以本机验收结果为准。
