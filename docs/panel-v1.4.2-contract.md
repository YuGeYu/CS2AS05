# Panel v1.4.2 融合契约

固定来源：`ed0ard/CS2-Bot-Improver` tag `v1.4.2`，提交 `97fd57d2ee1e14e408ae3ca7b1b0cae596a792cc`。内置定制 ZIP SHA256 为 `55DC504BFF8340ABE6AB317661518B512DB6FFA463F10EE3AF84149B1795C3E9`。

## 已确认磁盘契约

| 输入 | 改动文件 | 写后状态 | CS2 运行要求 |
| --- | --- | --- | --- |
| Low/Medium/High | `overrides/<Level>/botprofile.vpk` 复制到 `overrides/botprofile.vpk` | 字节等于对应源文件 | 可写，运行中提示待重启 |
| online/bots | 对应 `backup/Online|WithBots/gameinfo.gi` 复制到活动 `gameinfo.gi` | 活动文件字节等于所选备份 | 必须停止 CS2 |
| head/mixed/body | 两个 `cfg/my_bot_*_config.cfg` 中唯一 `bot_aim <value>` 管理行 | 两文件值一致 | 可写，运行中待重启 |
| max/more/normal/off | 两个 cfg 中唯一 `bot_nades <value>` 管理行 | 两文件值一致 | 可写，运行中待重启 |
| skins/profiles/agents/music | `BotRandomizer/bot_randomizer_options.json` | 四个 bool 独立，未知 JSON 字段保留 | 可写，运行中待重启 |
| 丢刀按键和 subclass | 两个 cfg 中完全由允许的 `subclass_create` 组成的 bind | 去重并按固定白名单排序 | 可写，运行中待重启 |

内置 ZIP 初始状态为 BOT、Low、Aim `mixed`、Nades `normal`、Bot 物品四项全开，并使用 `\\` 绑定全部 20 个 subclass。在线备份不包含 `csgo/addons/metamod` 搜索路径，BOT 备份包含该路径。

首次刷新会逐字段迁移到 `cfg/cs2as05-panel-state.json`。状态标记和磁盘可解析值优先于默认值；全关、空刀具及 online 等合法选择不会被刷新、覆盖安装或 BOT 自动更新重置。损坏状态会归档为 `.corrupt-<timestamp>` 后从磁盘恢复，CS2 运行中返回 `deferred` 且不写盘。

## 参数和安全约束

- 刀具 ID 固定为 `500, 503, 505, 506, 507, 508, 509, 512, 514, 515, 516, 517, 518, 519, 520, 521, 522, 523, 525, 526`。
- bind key 拒绝引号、分号、换行、控制字符和超过 16 字节的输入。
- 每次写入先创建带时间戳单文件备份，再使用同目录临时文件、`sync_all`、替换和回读校验。
- 模式和启动在检测到 `cs2.exe` 时拒绝。BOT 启动参数为 `-insecure -console -condebug`；在线模式不附加这些参数。
- `core.json` 不随 skins 修改；固定上游前端的 `reconcile_core_json` 调用没有足够证据证明需改变该文件。

## 静态数据

`src/data/panel/commands.txt` 原样取自固定 tag，SHA256 为 `185893ADB080565E77447066E42256C58E76A8343459C0C7C3ED1D723A21C139`。解析结果为 176 项用户可见内容和 40 支完整 CT/T 队伍。

## 仍需发布前实机复核

原版 EXE 与融合版的逐动作双基线报告、真实 CS2 online/bots 游戏内往返和运行中状态仍以 `scripts/panel-diff.ps1` 输出为发布闸门；本文不以源码推断替代该闸门。
