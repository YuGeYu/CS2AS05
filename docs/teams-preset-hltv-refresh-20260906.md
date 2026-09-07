# 队伍预设数据核对记录（2026-09-06）

本次只更新人机预设中的 `ADD TEAMS` 队伍命令，不改变上游资源包、Bot 难度或其他面板行为。

## 核对结果

- 上游基线：`ed0ard/CS2-Bot-Improver` `main` 分支的 `Panel/src/data/commands.txt`。上游当前仍提供 40 支队伍，本地原有 40 支与其一致；本地额外保留的 `br_reroll`、`bot_nades less` 属于下游功能，不在本次队伍数据变更范围内。
- HLTV 9z：<https://www.hltv.org/team/9996/9z>，通过 `https://r.jina.ai/http://www.hltv.org/team/9996/9z` 读取。页面显示 2026-08-31 周榜世界排名 #11，现役阵容为 `max`、`HUASOPEEK`、`luchov`、`meyern`、`dgt`。
- HLTV FUT：<https://www.hltv.org/team/13286/fut>，通过 `https://r.jina.ai/http://www.hltv.org/team/13286/fut` 读取。页面显示 2026-08-31 周榜世界排名 #6，主阵容为 `xfl0ud`、`dem0n`、`Krabeni`、`cmtry`、`dziugss`；`lauNX` 标记为替补，因此没有写入 BOT 五人预设。
- 直接请求 `www.hltv.org` 在当前环境触发 Cloudflare JavaScript 挑战；Jina 返回的页面正文仍明确标注 URL Source 为 HLTV，故本记录保留该访问边界，不把挑战页当作数据来源。

## 实现

`src/data/panel/commands.txt` 新增编号 41、42。每支队伍均提供完整的 CT/T 两条命令，包含五个 `bot_add_*`、`mp_teamlogo_*` 和 `mp_teamname_*` 字段。解析契约从 40 支/178 条命令更新为 42 支/184 条命令。

队伍简称使用 CS2 控制台可接受的 ASCII 标识：9z 使用 `9z`，FUT 使用 `fut`。两条命令的队名显示仍使用 HLTV 的官方队名。
