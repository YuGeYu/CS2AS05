# 0.5.10 BOT 难度与 gameinfo 上游对齐执行报告

日期：2026-08-31  
状态：资源修复候选，待真实 BOT 验收。

上游结构参考 commit：`ed0ard/CS2-Bot-Improver@5ef2235bf83fdd563384e30355eb547ce2c47166`。该源码树未提供可直接复制的 gameinfo.gi；本次未使用旧 release gameinfo 或旧 release VPK。

## 当前官方基线与资源

基线来源：`D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\gameinfo.gi`。当前文件大小 `9433` bytes，SHA-256：
`B1391E73DBEC2E078BDBAF7279C2B955084CF2B38A47E3D8181662E7948679B8`。

已执行 `scripts/refresh-gameinfo-from-steam.ps1`，从该官方 bytes 生成 ZIP 内三份静态资源：

- `gameinfo.gi`：官方字节，9433 bytes，SHA-256 `B1391E73DBEC2E078BDBAF7279C2B955084CF2B38A47E3D8181662E7948679B8`
- `backup/Online/gameinfo.gi`：官方字节，9433 bytes，同上
- `backup/WithBots/gameinfo.gi`：官方 bytes 加两条且仅两条 SearchPath，9501 bytes，SHA-256 `04B867124656BC9E768ED50B45193225D799E273D2ADC193632ED03543B37EB1`

WithBots 新增顺序为：

```text
Game csgo/overrides/botprofile.vpk
Game csgo/addons/metamod
```

插入点位于官方 `Game_LowViolence csgo_lv` 后、官方 `Game csgo` 前；Online 不含这两条路径。manifest 已同步为 `resourceVersion=0.5.10`。

## 难度资源

ZIP 当前三档 VPK 摘要：

- Low：115353 bytes，SHA-256 `5EC7F50BCD97E3678C7545306407110E7E87C6E09CBC604DA70CFA56A612206A`
- Medium：406110 bytes，SHA-256 `495FCAE5DFAABB1B47CEDF636FD48E906085823C09BEC4C6D5589A33BEBEF695`
- High：115601 bytes，SHA-256 `EA7D7FC6DEF8E0342AA817F64A7E9BC3844E3A6372F1567C3C60C74F6AECE233`

当前工作树能证明三档文件存在且内容互不相同，但尚未建立来自上游 commit `5ef2235bf83fdd563384e30355eb547ce2c47166` 的逐档 VPK 构建 provenance，也未执行真实行为强度验证；不能据此宣称 Medium 行为已修复。

## 自动化与备份

- `npm test -- --run tests/gameinfo-upstream-contract.spec.ts`：2 tests passed。
- `scripts/refresh-gameinfo-from-steam.ps1` 生成并保存 ZIP 写前备份：`src-tauri/resources/CS2BotImprover.zip.before-gameinfo-refresh-20260831-203834.bak`。
- 刷新结果记录：`workspace/runtime/gameinfo-refresh/result.json`。

## 未完成的真实验收

尚未在 CS2/BOT 中读取启动前 snapshot、Metamod/CounterStrikeSharp 日志、实际加载 VPK、Medium 聊天提示与固定反应/射击场景；尚未执行 Low/Medium/High 行为对比、道具回归和切回 Online 后官方字节恢复。因此当前结论为“资源修复候选/待真实 BOT 验收”。
