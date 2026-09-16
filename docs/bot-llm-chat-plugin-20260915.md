# BOT LLM 发言插件接入记录（2026-09-15）

本项目新增 `CS2BotLlmChat` CounterStrikeSharp 插件，代码基于已获许可的上游项目：
`https://github.com/unicbm/cs2-bot-llm-chat`。上游源码、GPL-3.0 许可证和来源记录保存在 `third_party/cs2-bot-llm-chat-upstream/`，可发布插件源码位于 `third_party/CS2-Bot-Improver-v1.4.4/addons/counterstrikesharp/plugins/CS2BotLlmChat/`。

## 行为

- 每个回合开始，每个有效 BOT 独立以 80% 概率请求一次“唯一友好开场白”。
- BOT 获得 MVP：40% 概率嘲讽对面。
- BOT 被击杀：10% 概率发言；BOT 击杀真人：20% 概率发言。
- 真人或 BOT 发言时，插件保留公开聊天、最近上下文、BOT 阵营和战场事件，并先请求 AI 判断 `YES/NO`，只有 `YES` 才再次请求实际内容。
- 模型顺序：`swe-2-high`、`deepseek-v4-flash-0731`、`glm-5.3-flash`、`doubao`、`lingbao`、`脑力自算`；全部不可用或返回空内容时静默放弃，不向游戏聊天输出错误。
- 发言使用 BOT 自身 `say`，模型严禁生成 `bot2:` 等前缀。

## 密钥

配置只引用环境变量 `CS2_BOT_LLM_API_KEY`，不在仓库保存用户提供的密钥。服务器启动前设置该环境变量即可。API 地址默认 `https://api.600318.xyz`。

## 本地证据

- `dotnet build .../CS2BotLlmChat.csproj -c Release`：通过，0 警告、0 错误。
- 已将 DLL、deps、PDB、配置写入 `src-tauri/resources/CS2BotImprover.zip`。
- 候选包哈希与条目记录：`artifacts/bot-llm-chat-20260915/candidate-sha256.txt`。
- 尚未进行真实 CS2 对局中的发言验收；因此不能将本地构建等同于实机行为已验证。
