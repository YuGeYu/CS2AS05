# CS2 手动确认已关闭闸门设计方案

日期：2026-09-07  
范围：CS2 进程状态误判时，允许玩家临时确认“游戏确实已关闭”并解锁受保护操作  
状态：调查与实施方案，尚未修改产品代码

## 1. 背景与目标

玩家反馈：退出 CS2 后，助手仍显示“游戏未关闭”；任务管理器中存在一个被挂起的 `cs2.exe`，该进程无法结束，手动结束也返回“拒绝访问”。玩家再次从 Steam 启动时出现新的 `cs2.exe`，因此助手当前按进程名判断的结果与玩家实际看到的游戏实例不一致。

本方案不是简单增加“忽略 CS2 状态”开关，而是增加一个**玩家确认后完全解除 CS2 运行锁**的手动确认状态：当玩家明确确认游戏窗口和实际对局已关闭，但系统残留进程无法终止时，所有原本仅因 `cs2Running` 被锁定的玩家功能都应恢复可用。确认不会改变检测结果，也不会让助手擅自结束进程；但不能再出现“BOT 工坊解锁了、Panel/预设/物品/地图轮换仍然继续卡住”的半解锁体验。

## 2. 当前证据与问题判断

### 已确认线索

- 玩家看到任务管理器中的挂起 `cs2.exe`，且结束进程返回拒绝访问。
- 玩家描述助手检测的 PID 与实际启动的 `cs2.exe` 不是同一个，说明仅显示进程名不足以解释实例关系。
- 当前历史设计中 `check_cs2_process()` 主要按 `cs2`/`cs2.exe` 名称匹配，不返回稳定 PID、映像路径、父进程或启动时间。
- 现有写操作广泛调用 `cs2::check_cs2_process()`，包括 Panel 配置、BOT 强度工坊、MapRotation 等；单一布尔值无法表达“真实游戏已退出但残留挂起进程仍存在”的灰色状态。
- `close_cs2` 的强制关闭必须避免影响 Steam、助手自身和未知 PID；“全杀 `cs2.exe`”不是可接受的默认修复。

### 不能直接假定

- 不能把所有名为 `cs2.exe` 的进程都当成同一个游戏实例。
- 不能把无法结束的挂起进程自动当作已关闭。
- 不能因为玩家点击确认，就绕过 Online `gameinfo.gi` 安全边界、插件版本校验、活动 VPK 备份或其它写入回滚。
- 不能把一次手动确认永久保存到配置或 `localStorage`；进程状态和确认授权都属于运行时状态。

## 3. 产品行为

### 3.1 触发条件

仅在以下条件同时满足时显示“我确认 CS2 已关闭”入口：

1. 进程快照至少发现一个疑似 `cs2.exe`，状态为 `running` 或 `unresponsive/suspended`；
2. 正常关闭已经失败，或进程快照连续两次显示目标进程无窗口/无活动游戏会话迹象；
3. 当前不是启动 CS2 流程；
4. 当前不是助手自动启动、安装事务或强制结束进程的中途。

普通 `running` 状态仍优先显示“关闭 CS2”，不能一开始就建议手动确认。

### 3.2 确认对话框

对话框必须明确写出风险和范围：

> 系统仍发现一个无法结束的 CS2 进程。只有在你确认游戏窗口、对局、服务器连接和 Steam 游戏会话都已结束后，才继续。
>
> 确认后，本次会话内所有因“CS2 仍在运行”而被锁定的玩家功能都会恢复，包括配置、预设、BOT 工坊、地图轮换、录制设置以及安装/切换流程中的 CS2 状态锁。确认不会结束任何进程，也不会替你启动新的 CS2。若游戏实际仍在运行，可能导致配置要到下次启动才生效，极端情况下会与游戏读取文件产生冲突。

操作按钮：

- `返回检查`：关闭对话框，保持锁定；
- `我确认 CS2 已关闭，解锁本次会话`：创建当前会话运行时授权。

不使用模糊的“忽略”“强制继续”作为按钮文字，不使用勾选框永久记忆。

### 3.3 确认后的状态

确认成功后，顶部状态条和相关页面显示：

- `已由你确认 CS2 关闭 · 仅本次操作有效`
- 显示确认时间、疑似进程数和授权剩余时间；不显示或传播完整 PID/用户隐私路径。
- **所有因 `cs2Running` 被禁用的玩家功能恢复可操作**，包括 Panel、BOT 强度工坊、BOT Items、预设、刀具、地图轮换、Demo 录制配置和其它本地设置；不再按页面逐个保留锁定。
- 保持一个醒目的“玩家已确认”标识，但该标识是状态提醒，不是额外的二次确认按钮。
- 任何新的进程快照发现可交互的 CS2 实例时，立即撤销授权并重新锁定。

建议授权生命周期：

- 默认 5 分钟；
- 仅对当前窗口会话和当前选定 CS2 根目录有效；
- 不在完成一次写操作后立即消费；确认状态在本次会话和有效期内持续生效，玩家可以连续调整多个配置，不需要反复点击确认。
- 切换 CS2 根目录、重新启动助手、进入启动流程、用户主动点击“撤销确认”时立即失效；
- 不写数据库、配置文件或 `localStorage`。

## 4. 完全解锁的准确含义

人工确认后的原则是：**凡是原本只因为检测到 CS2 运行而被锁定的玩家功能，全部解除锁定。** 这不是“只允许一次写入”，也不是“只允许 BOT 工坊”。

| 操作类别 | 人工确认后 | 说明 |
| --- | --- | --- |
| Panel 模式/难度/Aim/Nades/BOT Items/刀具设置 | 完全解锁 | 可连续修改、保存和回读 |
| BOT 强度工坊打开/创建/编辑/保存/应用/重命名/删除 | 完全解锁 | 保留工坊自身备份、回读、回滚和内容校验 |
| MapRotation、Demo 录制、本地预设等退出 CS2 才能写入的设置 | 完全解锁 | 只要原门禁原因是 `cs2Running`，就不得继续阻塞 |
| 只读诊断、状态刷新、VPK 提取 | 完全解锁 | 不应因残留进程再次阻断 |
| 安装/升级/卸载插件 | 解除 `cs2Running` 这一层锁定 | 资源、版本、权限、备份和回滚门禁仍保留 |
| Online/BOT 模式切换 | 解除 `cs2Running` 这一层锁定 | `gameinfo.gi`、Steam、资源和恢复校验仍保留 |
| 启动 CS2 | 不接受人工确认直接放行 | 启动前必须重新检查 |
| 强制结束进程、结束 Steam、结束未知 PID | 不解锁 | 这是独立的进程破坏操作 |

## 5. 允许与禁止的操作

### 允许在确认闸门下完全解锁

- 所有因 `cs2Running` 单一条件而禁用的配置、读取、回读和应用操作；
- Panel、预设、BOT Items、刀具、MapRotation、Demo 录制、BOT 强度工坊等页面的连续操作；
- 安装/升级/卸载以及 Online/BOT 模式切换中由 `cs2Running` 造成的那一层锁定，在其它安全门禁通过后继续执行；
- 只读诊断、VPK 提取和环境状态回读。

### 即使确认也禁止

- 启动新的 CS2 或 Steam 游戏会话；
- 强制结束任何进程；
- 结束 `steam.exe`、助手自身或未知父子进程；
- Online `gameinfo.gi` 官方基线恢复、安装/卸载插件和资源迁移，除非单独完成更高风险确认与现有安装门禁；
- 绕过文件占用、权限错误、VPK 回读失败或其它事务回滚；
- 将确认状态作为下一次启动的默认状态。

## 5. 后端协议设计

### 5.1 结构化进程快照

在 `src-tauri/src/services/cs2.rs` 中统一生成快照，替代调用方自行使用布尔检查。建议字段：

```text
Cs2ProcessSnapshot {
  observedAt,
  state: running | stopped | unknown | suspended,
  processes: [{ pid, imageName, imagePath?, parentPid?, startedAt?, responsiveness }],
  confidence: high | medium | low,
  reason
}
```

匹配仍以 `cs2.exe` 为必要条件，但要记录 PID、路径可用性、父进程和响应性；查询失败必须返回 `unknown`，不能保留旧的 `running` 假象。连续两次采样间隔 150–300ms 后才发布 `stopped`，避免退出瞬间竞态。

### 5.2 手动确认 command

新增受限 command，例如：

```text
confirm_cs2_closed(root_path) -> Cs2CloseOverride
revoke_cs2_closed_confirmation(root_path) -> void
```

后端生成随机内存 token，保存确认时的根目录规范路径、进程快照摘要、确认时间和过期时间。前端只收到状态摘要，不自行生成 token 或传任意 PID。

所有需要“CS2 已退出”的写入 command 在进入实际文件事务前调用统一 gate：

```text
require_cs2_write_clear(root, operation_kind)
  1. 读取当前结构化快照
  2. stopped -> 放行
  3. suspended/unresponsive + 有效人工确认 + 根目录一致 + 未过期 -> 放行所有原本仅被 cs2Running 阻断的 operation_kind
  4. 其它状态 -> 返回 CS2_RUNNING / CS2_STATE_UNKNOWN
```

人工确认只改变“是否允许玩家继续使用功能”的决策，不改变进程检测结果。日志必须区分 `detected_running`、`manual_confirmed` 和 `write_allowed_with_override`。确认状态在有效期内持续生效，不应要求玩家每次保存重复确认。

### 5.3 操作分类

为避免误放行，命令调用方传入后端固定的 `operation_kind`，不能由前端任意填写。至少区分：

- `panel_config_write`
- `bot_workshop_write`
- `map_rotation_write`
- `install_or_uninstall`
- `online_gameinfo_recovery`
- `launch_cs2`

gate 对所有“仅因 cs2Running 而锁定”的 operation_kind 放行；安装/卸载和 Online 恢复继续执行各自的资源、版本、官方基线、备份和回滚校验；`launch_cs2`、`force_terminate_process` 永远不接受人工确认授权。

## 6. 前端状态与交互

### 状态机

```text
checking -> stopped -> 可写
checking -> running -> 关闭 CS2
checking -> suspended/unresponsive -> 关闭失败 -> 手动确认入口
unknown -> 重新检查（不显示确认入口）
manual_confirmed -> 连续配置操作 -> stopped/过期/重新检查
manual_confirmed -> 新快照发现 CS2 -> running + 撤销确认
```

### UI 位置

- 全局 `StatusStrip`：显示进程状态、关闭按钮和“无法结束？确认已关闭”次级入口；
- 概览页：环境摘要中显示确认状态，主启动按钮不能被确认授权解锁；
- BOT 强度工坊：在保存/应用锁定位置显示“检测到残留进程”，提供同一确认对话框；确认后本次会话内不再重复拦截其它配置页面；
- 安装与诊断页：显示详细 PID/路径/权限诊断，但不因确认而绕过安装门禁。

所有入口调用同一个 store/composable，不在各页面维护独立布尔值。

## 7. 文件映射

| 文件 | 计划职责 |
| --- | --- |
| `src-tauri/src/services/cs2.rs` | 结构化快照、PID/响应性分类、统一写入 gate、确认生命周期 |
| `src-tauri/src/commands/cs2.rs` | 暴露 snapshot/confirm/revoke command，保持强制关闭边界 |
| `src-tauri/src/models/cs2.rs` | 新增快照和确认 DTO |
| `src/services/tauri/cs2.ts` | TS IPC 类型和调用封装 |
| `src/stores/cs2.ts` | 单一确认状态、快照刷新、失效/消费逻辑；不使用 localStorage |
| `src/components/StatusStrip.vue` | 全局状态入口、关闭失败后的确认入口和可访问提示 |
| `src/views/OverviewView.vue` | 环境摘要和启动按钮的确认状态；启动始终不受人工确认放行 |
| `src/components/BotDifficultyWorkbench.vue` | 保存/应用锁定提示，复用全局确认对话框 |
| `src-tauri/src/services/panel.rs` | 将 Panel 写入门禁改为统一 gate |
| `src-tauri/src/services/bot_difficulty.rs` | 将 BOT 工坊打开/保存/应用门禁改为统一 gate |
| `tests/cs2-process-polling.spec.ts` | 快照状态、双采样、错误时清空旧 running 状态 |
| 新增 `tests/cs2-manual-close-confirmation.spec.ts` | 生命周期、消费、过期、根目录隔离、禁止启动/安装 |
| 新增 Rust 单测 | gate 操作分类、unknown 拒绝、suspended 确认放行、并发消费 |

## 8. 实施顺序

1. 先重读当前 `cs2.rs`、`StatusStrip`、关闭 command 和所有写入调用方，列出每一处现有布尔门禁。
2. 先实现结构化快照和统一 gate，不改 UI；用模拟 PID/unknown/suspended/running/stopped 状态补 Rust 测试。
3. 增加运行时确认 DTO、一次性消费和根目录隔离；验证确认状态不进入持久化存储。
4. 在 StatusStrip 实现确认对话框与撤销/过期状态，再接入概览和 BOT 工坊。
5. 保持安装/卸载、Online 恢复和启动拒绝人工确认；补操作分类契约测试。
6. 运行 `npm run typecheck`、相关 Vitest、`cargo test --manifest-path .\\src-tauri\\Cargo.toml cs2 panel bot_difficulty`、`npm run build:web`。
7. 在 Windows 实机用一个可正常结束的 CS2、一个延迟退出的 CS2 和一个无权限结束的挂起进程分别验证；记录 PID/快照/确认/写入结果，不能只看 UI 文案。

## 9. 验收门禁

- 正常退出 CS2 后不需要手动确认，双采样后自动解锁。
- 正常运行 CS2 时确认入口不出现，或确认后端明确拒绝。
- 只有 `suspended/unresponsive` 且玩家明确确认后，所有因 `cs2Running` 被锁定的玩家功能都可连续使用；不要求每次写入重新确认。
- Panel、BOT 工坊、预设、BOT Items、刀具、MapRotation、Demo 录制和安装/模式切换都验证“不会保留一处隐藏的 cs2Running 锁”。
- 新 CS2 实例出现、切换根目录、重新启动助手或进入启动流程后，确认立即失效。
- 任何确认都不会结束进程、杀 Steam、放行启动、绕过 Online 基线、安装/卸载或破坏性覆盖。
- Panel、BOT 工坊、MapRotation 等写入都经过同一个后端 gate，不存在页面级绕过。
- `unknown` 状态默认拒绝写入，不能用“玩家点过确认”掩盖检测失败。
- 真实 Windows 现场至少完成一次“挂起进程无法结束 + 玩家确认 + 配置保存 + 回读”闭环后，才能在更新日志中写为修复。

## 10. 停止条件

- 无法可靠区分目标 PID 与 Steam/助手进程时，停止增加强制关闭能力，只保留诊断和人工确认写入闸门。
- Windows API 查询返回 `unknown` 或权限不足时，不自动降级为 stopped，也不自动显示确认入口。
- 任意页面绕过统一 gate、确认状态可持久化、确认可解锁启动/安装/Online 恢复时，停止发布。
- 真实现场确认后仍发生文件被游戏占用、原子替换失败或配置未回读一致时，撤回“可写放行”结论，先修复事务和证据链。
