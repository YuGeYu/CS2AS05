# Panel v1.4.3 融合契约

固定来源：`ed0ard/CS2-Bot-Improver` tag `v1.4.3`，提交 `d1d83982db88fbdb686b2bf13aa8c6f9d65a4604`。官方 Windows ZIP 为 67,801,513 bytes，SHA256 `F4330B13F6154A36EEB2820FB79AF015014397C7E06C7791AE0AF51F762A3652`；其中 `Panel v1.4.3.exe` 为 5,844,480 bytes，SHA256 `3FD93DC7AF2702C50B9A7E4FCF1BB11387B107ABC863EE8A3067255022408CCD`。

## 首次默认与升级

全新安装且没有受管状态时，默认写入 `bots / Low / mixed / less`、八项 Bot Items 全部 `true`、丢刀键 `\\` 和刀具 `507, 508, 515, 519, 525`。覆盖升级只恢复已明确捕获的合法值；显式 `normal` Nades、Bot Items 全关和空刀具不会被新默认覆盖，缺失或未初始化字段才使用上述默认。

兼容性提示：部分用户的 CS2/插件环境可能无法在游戏内应用“探员模型”和“丢刀/刀具”开关。官方 Panel 在相同环境也可能出现相同限制。助手仍会按契约写入并回读配置；本提示不代表每台电脑都能得到对应的游戏内效果。

## 磁盘契约

| 输入 | 改动文件 | 写后状态 | CS2 运行要求 |
| --- | --- | --- | --- |
| Low/Medium/High | `overrides/<Level>/botprofile.vpk` 复制到 `overrides/botprofile.vpk` | 字节等于对应源文件 | 可写，运行中提示待重启 |
| online/bots | 对应 `backup/Online|WithBots/gameinfo.gi` 复制到活动 `gameinfo.gi` | 活动文件字节等于所选备份 | 必须停止 CS2 |
| head/mixed/body | 两个 `cfg/my_bot_*_config.cfg` 中唯一 `bot_aim <value>` 管理行 | 两文件值一致 | 可写，运行中待重启 |
| max/more/normal/less/off | 两个 cfg 中唯一 `bot_nades <value>` 管理行 | 两文件值一致 | 可写，运行中待重启 |
| 八项 Bot Items | `addons/counterstrikesharp/configs/core.json` | 八个官方 bool 键独立，未知 JSON 字段保留 | 可写，运行中待重启 |
| 丢刀按键和 subclass | 两个 cfg 中完全由允许的 `subclass_create` 组成的 bind | 去重并按固定白名单排序 | 可写，运行中待重启 |

八项顺序固定为 `profiles, agents, music, weapons, knives, gloves, stickers, charms`。`profiles` 映射 `bot_hider github.com/XBribo all`，其余项映射 `bot_randomizer github.com/ed0ard <item>`。缺失的受管键按官方行为视为 `true`；类型不是 bool 时拒绝写入。旧 `bot_randomizer_options.json` 和旧状态中的 `skins` 不再参与活动状态，未知字段不删除。

## 安全与迁移

- Nades 仅接受 `max|more|normal|less|off`，非法值在 Rust 写盘前拒绝。
- 刀具 ID 和 bind key 继续使用固定白名单；写入使用备份、同目录临时文件、同步、替换和回读校验。
- `0.5.3` marker 会触发 `0.5.4` 安装；合法更高核心版本不覆盖，摘要不可信的更高版本阻止启动。
- 旧 `official-panel-v1.4.2` app-data 目录保留，新 Panel 使用独立 `official-panel-v1.4.3` 目录。
- 覆盖安装捕获并恢复可识别偏好，不清理插件目录中的未知文件。

## 静态数据

`src/data/panel/commands.txt` SHA256 为 `E0D7C5F1247DE1E766A75CA656AFBED22BF9309FB104FFA2E83AD505E53101E7`，解析为 178 项内容和 40 支完整 CT/T 队伍。`bot_nades less` 与 `br_reroll` 各出现一次，并通过 Tauri 剪贴板插件复制。

## 发布门槛

本文记录自动化和固定资产契约，不替代真实 CS2 行为验收。`less`、八项 Bot Items、`br_reroll`、三类投掷入口和实际回合节奏仍需用户在备份后的真实游戏树验证后才能切换生产 latest 或发布 GitHub Release。

## CS2 更新后的 gameinfo 动态基线

资源包中的 `gameinfo.gi`、`backup/Online/gameinfo.gi`、`backup/WithBots/gameinfo.gi` 是由开发阶段当前官方主 mod 文件制作的静态资产。安装时写入 `cfg/cs2as05-gameinfo-state.json` 和 `gameinfo.gi.official.bin`，记录三份资源摘要、版本与生成日期；运行时只校验这些静态摘要并原子切换，不读取用户文件生成或编辑 gameinfo。若活动摘要与官方/BOT 变体均不匹配，快照状态为 `recoveryRequired`，Online 启动被阻止，用户必须先退出 CS2 并在 Steam 中验证文件，再重新制作发布资源。
