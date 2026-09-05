# 快快客服跨玩家电脑 GitHub 与 PowerShell 能力修复记录

日期：2026-08-16

## 问题

快快客服虽然会预取少量公开 README，但模型缺少可连续调用的线上资料工具；早期 PowerShell 草稿还依赖开发机路径 `E:\CS2AS05`，默认连接 Key 只读取开发机用户环境变量，因此不能证明安装到玩家电脑后可用。

## 修复

- 新增 Tauri 命令 `run_ai_powershell`，在玩家自己的应用缓存目录 `quick-support` 中运行系统自带 `powershell.exe`。
- PowerShell 工具支持快快客服逐步查询 GitHub API、Raw 文件和公开网页；玩家电脑不需要安装 `git`、`gh` 或 `rg`，模型会优先使用 `Invoke-RestMethod`。
- 每次 PowerShell 最多执行 30 秒、最多回传 16000 字符；超时自动终止，凭据文件及默认连接环境变量读取会被阻止。
- Agent 每轮只执行一个工具动作，将结果回传模型后继续下一步，最多连续 6 步；工具标记和脚本不会显示在用户聊天记录中。
- GitHub README 预取改用 `/repos/{owner}/{repo}/readme` API，自动跟随仓库默认分支，不再写死 `main` 或 `master`。
- 移除 `E:\CS2AS05` 和 `CS2AS_WORKSPACE` 等开发机工作目录假设。
- 默认连接 Key 由构建期生成混淆字节并编入正式 EXE；源码、配置、日志和报告不保存明文。玩家自定义 Key 仍只保存在本机应用数据目录。

## 验证

- `npm run typecheck`：通过。
- `cargo check --manifest-path src-tauri\Cargo.toml`：通过；仅有上游 `demoparser` 既有警告。
- 快快客服 PowerShell 相关 Rust 测试：凭据阻断、公开 GitHub 查询语法、任意玩家缓存目录执行均通过。
- 在 `%TEMP%\cs2as-player-github-check` 仅使用 `Invoke-RestMethod`，读取到 `YuGeYu/CS2AS05`、默认分支 `main` 和 4870 字符 README；未使用 `git`、`gh`、`rg`。
- `npm run build:desktop`：通过，Tauri 输出正式应用路径。
- 清除运行进程中的 `CS2AS_DEFAULT_API_KEY` 后启动正式 EXE：窗口标题为 `CS2人机增强助手`，`Responding=True`，未连接 `localhost:5173`。
- 正式 EXE 的连接信息界面真实显示“已读取 14 个可用模型”，证明默认连接不依赖玩家电脑预设环境变量。
- 正式 EXE 中未发现完整默认 Key 明文。

## 当前边界

- 使用默认连接向 `gpt-5.6-sol` 发起一次端到端 Agent 协议测试时，中转站返回 `Insufficient account balance`。因此本次无法在线回读模型是否输出 PowerShell 动作标记；没有重复请求。客户端工具执行、动作解析、GitHub 网络读取和模型列表已分别验证，额度恢复或玩家配置自有连接后可按下方提示做最终对话验收。

## 产物

- 路径：`E:\CS2AS05\src-tauri\target\release\CS2BotImproverAssistant.exe`
- 大小：`46,540,800` bytes
- SHA-256：`940B1E2D97249D400473FE889854B75642A77BBE0315E70B49A2ECD7D39B1500`

## 玩家复现建议

在快快客服中发送：

> 请查询 YuGeYu/CS2AS05 GitHub 仓库的最新默认分支，并读取 README 中的主要功能，给出你实际读取的资料来源。

快快应调用 PowerShell/GitHub API，收到客户端执行结果后继续回答，而不是声称没有线上工具。
