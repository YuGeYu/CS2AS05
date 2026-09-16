# 真人闪光影响 BOT 修复与插件校验降敏记录

## 根因

直接对照 ed0ard/CS2-Bot-Improver `v1.4.4`（提交 `7491e175f83e612dbb1c742c2241d454ed4c15ad`）源码确认，真人闪光影响 BOT 的逻辑在 `addons/counterstrikesharp/plugins/BotState/BotState.cs`，不是 NadeSystem 的 BOT 投掷概率。

BOT 只在飞行中的闪光弹同时满足 FOV 与世界 LOS 时进行一次躲闪判定；随后 `OnPlayerBlind` 和 `CCSBot::Blind` hook 共享该判定。上游默认躲闪概率为：剩余引爆时间 `>600ms` 为 95%，`401–600ms` 为 90%，`251–400ms` 为 50%，`151–250ms` 为 20%，`<=150ms` 为 5%。因此真人通常提前出现在 BOT 视野内时，BOT 有约 95% 概率躲开，造成真人闪白成功率约 5%。

## 本次修复

只调整真人闪光对 BOT 的躲闪数值，不改变 FOV、LOS、引爆匹配、事件处理或原生 hook：

| 剩余引爆时间 | 上游躲闪 | 本次躲闪 | 真人成功机会 |
|---:|---:|---:|---:|
| >600ms | 95% | 70% | 约 30% |
| 401–600ms | 90% | 55% | 约 45% |
| 251–400ms | 50% | 35% | 约 65% |
| 151–250ms | 20% | 15% | 约 85% |
| <=150ms | 5% | 5% | 约 95% |

该梯度不会让 BOT 变成“极易被闪”：BOT 仍可在提前看见时以 70% 概率躲闪，在临近引爆时才明显难以反应；隔墙、背向、超出 FOV 的闪光仍按游戏原生逻辑正常生效。

## 插件降敏

- 内置 ZIP 仍执行严格固定 SHA-256、ZIP 结构、gameinfo 清单和 marker 身份校验。
- 已安装目录不再用 NadeSystem payload 的逐文件摘要阻断启动；玩家自定义 JSON、配置、PDB 或非关键插件文件差异不会再直接触发敏感报错。
- 仍要求 marker 身份/版本可解析，以及 BotState.dll、NadeSystem.dll 等关键插件存在；关键文件缺失时自动进入修复安装。
- 高于当前版本但 marker 可解析的异常状态不再单独阻断，统一走自修复安装路径。
- BotState 编译使用现有 `InjectUsercmd/CancelUsercmdInjection` ABI，未引入 v1.4.4 源码中与当前 BotControllerApi 不存在的实验接口，降低加载失败风险。

## 产物与验证

- BotState 源码同步保存于 `third_party/CS2-Bot-Improver-v1.4.4/.../BotState.cs`。
- 已成功执行 BotState Release 构建：0 错误，1 个上游已有未使用字段警告。
- 已替换内置 ZIP 中 BotState.dll、BotState.deps.json、BotState.pdb。
- 当前资源包 SHA-256：`54252EB5D19FCDA6BFA276F0AFFD8C37B7D5A664B41BE30E3F67FBBF1AC82854`。
- 运行 Rust 全量库测试：90 通过、1 失败、4 忽略；失败为既有 `mutations_converge_to_the_aggregated_disk_snapshot` 临时路径错误，发生在本次插件门禁改动无关的 Panel 测试路径，未将其伪装为通过。
- `cargo check` 通过；前端此前 `npm run typecheck` 与 `npm run build:web` 已通过。

## 尚未宣称

- 尚未在真实 CS2 对局中统计 `player_blind`，因此不能宣称真人闪白成功率已达到某个固定百分比。
- 未构建安装器、未上传、未提交、未推送、未发布。
