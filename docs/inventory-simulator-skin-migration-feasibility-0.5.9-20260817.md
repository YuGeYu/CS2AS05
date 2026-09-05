# 0.5.9 Inventory Simulator 换肤迁移可行性调查

日期：2026-08-17

## 结论

**有条件可行，建议进入隔离原型和真实本地 BOT 验收；不建议直接把教程 ZIP 覆盖进当前安装包。**

教程采用 `ianlucas/cs2-css-inventory-simulator`。它可以替代当前 `PlayerSkinMod 1.8.2`，并且与项目现有的 Metamod、CounterStrikeSharp `1.0.371` 和 `-insecure` 本地 BOT 场景方向一致。固定调查版本 `3.1.0` 正好以 `CounterStrikeSharp.API 1.0.371` 编译，本机从固定源码构建成功，0 warning / 0 error。

它不是当前皮肤工坊的“配置格式升级”，而是把配置中心从助手本地 `player_loadout.json` 改为 `inventory.cstrike.app`：玩家用 Steam 登录网页制作库存，插件按 SteamID 从公开 API 拉取装备并在 CS2 中应用。迁移会改变产品的数据所有权、联网依赖、隐私说明、错误处理、部署资源和 UI 主流程。

## 调查对象与固定证据

- 教程原文：<https://www.bilibili.com/opus/1230050132217561090>
- 短链：<https://b23.tv/dkyic7n>
- 教程标题：《如何在CS2中利用开源插件5分钟学会本地自定义皮肤》
- 教程更新时间：2026-07-28 20:58
- 插件仓库：<https://github.com/ianlucas/cs2-css-inventory-simulator>
- 固定 tag：`3.1.0`
- 固定 commit：`5e3c96283b3d3f5aeba44822a38031df2e213376`
- 许可证：MIT，仓库文件名为 `License.txt`
- Release ZIP：`InventorySimulator-v3.1.0.zip`
- ZIP size：`72,342` bytes
- ZIP SHA-256：`B42A716E331C89DCCD079DB78677B3087FC43947757D75B5AE43A9C4823912D6`
- 插件目标框架：`.NET 10.0`
- 编译依赖：`CounterStrikeSharp.API 1.0.371`
- 公共服务：<https://inventory.cstrike.app>
- 公开装备接口：`GET /api/equipped/v5/{SteamID64}.json`

2026-08-17 实测官网返回 HTTP 200；使用无库存测试 SteamID 请求公开装备接口返回 HTTP 200 和 `{}`。这只能证明调查时服务可达，不代表未来可用性 SLA。

## 教程实际流程

1. 安装 Metamod 和 CounterStrikeSharp。
2. 设置 `addons/counterstrikesharp/configs/core.json` 中 `FollowCS2ServerGuidelines: false`。
3. 把 release ZIP 中的 `addons` 覆盖到 `game/csgo/addons`。
4. 加载 `plugins/InventorySimulator` 和 `gamedata/inventory-simulator.json`。
5. 玩家在 `inventory.cstrike.app` 使用 Steam 登录，按 SteamID 保存自定义库存。
6. 使用 `-insecure` 启动本地 BOT 对局；连接时插件自动拉取库存。
7. 玩家输入 `!ws` 请求重新拉取，或换图、重启 CS2 后同步。

教程强调仅用于社区服或 `-insecure` 本地离线/BOT，不适用于官匹、5E、完美等环境。

## 教程遗漏与源码修正

教程把 `!ws` 描述为开箱即用，但 `3.1.0` 源码默认值是：

```text
invsim_ws_enabled false
invsim_ws_immediately false
invsim_ws_cooldown 30
```

因此默认输入 `!ws` 只会显示网站地址，不会执行强制刷新。0.5.9 若采用教程体验，必须由受管配置显式设置：

```text
invsim_ws_enabled true
```

是否启用 `invsim_ws_immediately` 应在真实 BOT 中单独验证。启用后插件会删除并重新发放当前武器以立即应用变化，虽然会恢复弹匣和备用弹药，但仍需验证投掷物、C4、拾取武器、切枪和 Bot Improver 行为不回归。保守默认应继续为 `false`，让装备在重生/换图时稳定应用。

插件默认还会对官方公共 API 发送无密钥 StatTrak 增量和喷漆消耗请求。若 0.5.9 只提供本地 PVE 外观，建议首版显式关闭：

```text
invsim_public_api_stattrak_increment false
invsim_public_api_spray_consume false
invsim_wslogin false
invsim_require_inventory false
```

这样可以缩小远程写操作、认证和连接阻塞面；保留只读装备拉取。

## 与当前方案的差异

| 项目 | 当前 PlayerSkinMod 1.8.2 | Inventory Simulator 3.1.0 |
| --- | --- | --- |
| 配置来源 | 助手本地 `player_loadout.json` | `inventory.cstrike.app` 按 SteamID 提供 |
| 联网要求 | 应用皮肤不需要联网 | 首次拉取/刷新依赖公网服务 |
| 登录 | 不需要 Steam 网页登录 | 玩家需在第三方网站使用 Steam 登录 |
| 编辑 UI | 助手内中文图文皮肤工坊 | 外部网页制作库存 |
| 同步 | FileSystemWatcher + 重生 | 连接拉取、`!ws`、重生/换图 |
| 身份模型 | 本地 player slot | SteamID64 |
| 功能范围 | 武器、刀、手套、角色、音乐盒、贴纸、挂件等 | 另含收藏品、涂鸦和更完整的库存语义 |
| 运行时钩子 | `GiveNamedItem` Post、属性写入 | `GiveNamedItem` Pre、`GetItemInLoadout` Post、多项内存签名 |
| 失效模式 | 本地文件或签名失效 | 还增加 DNS、TLS、Cloudflare、API、Steam 登录和远端数据失效 |
| 许可 | 当前 vendor 来源存在 GPL 标注与缺独立 LICENSE 的记录 | MIT，独立 `License.txt` 完整 |

## 不能并存的原因

当前 `PlayerSkinMod` 与新插件都会接管同一玩家装备：

- 两者都 hook `GiveNamedItemFunc`。
- 两者都会清理或写入 `CEconItemView`/动态属性。
- 两者都会修改 paint、wear、seed、StatTrak、贴纸、刀、手套和角色。
- `PlayerSkinMod` 在生成后、下一帧及 250 ms 后重复应用；Inventory Simulator 在生成前和库存槽查询后返回自定义物品。

如果同时加载，最终外观取决于 hook 顺序、帧时序和重生路径，可能出现配置互相覆盖、刀/手套延迟、StatTrak 错位或客户端材质缓存异常。迁移必须执行“二选一”门禁：部署 Inventory Simulator 前先备份并停用/移除 `PlayerSkinMod`；回滚时反向恢复，不能保留双插件并让玩家自行判断。

## 对 0.5.9 的推荐产品方案

建议把现有“皮肤工坊”改成两层能力，而不是简单跳转网页：

1. **换肤服务状态**：检测固定插件版本、资源哈希、gamedata、CounterStrikeSharp 版本、`core.json` 和网络连通性。
2. **一键迁移**：CS2 退出门禁、备份旧 `PlayerSkinMod` 和 loadout、部署固定 `3.1.0` 资源、写入保守配置、部署后哈希回读；失败自动恢复旧插件。
3. **装备制作入口**：用系统默认浏览器打开 `https://inventory.cstrike.app`，不在 Tauri WebView 中承载 Steam 登录，不收集 Steam 凭据。
4. **同步指引**：明确显示“连接时自动同步；需要即时刷新时在游戏聊天输入 `!ws`；默认重生后生效”。
5. **离线与故障状态**：公网不可用时显示“暂时无法同步”，不阻断 BOT、Demo、启动或其他助手功能。
6. **可回滚**：提供明确的“恢复本地换肤”操作，恢复迁移前 `PlayerSkinMod` 和配置。

如果产品要求玩家继续在助手内完成完整可视化编辑，则不能只采用教程原方案。需要额外取得或实现 Inventory Simulator 的库存写入 API/授权契约；公开插件仓库只证明了读取与少量状态更新接口，不能据此假定助手可以代表玩家创建或修改网站库存。

## 必须通过的原型 Gate

1. 固定 release ZIP、MIT License、commit 和所有部署文件 SHA-256，写入 NOTICE/来源页。
2. 在隔离 CS2 副本或可恢复目录中只加载 Inventory Simulator，确保 `PlayerSkinMod` 不存在。
3. CounterStrikeSharp 日志确认插件加载且所有 Windows gamedata signature 解析成功。
4. 使用测试 Steam 账号在网站创建至少一把枪、刀、手套、CT/T 角色、音乐盒、贴纸/挂件。
5. 游戏内验证连接自动拉取、`!ws` 冷却、重生、换图、重启、断网、API 500/超时和空库存。
6. 验证 Bot Improver 的出生、购买、移动、射击、投掷物、拾取/丢弃武器、回合推进和正常退出不回归。
7. 验证插件请求最长 30 秒、最多 3 次重试时不会阻塞 CS2 主线程或导致助手无响应；记录 CounterStrikeSharp 日志时间线。
8. 验证旧插件备份、迁移失败恢复和一键回滚均可复现。
9. 由玩家确认第三方 Steam 登录和 SteamID/装备数据同步的产品说明可接受。

## 最终判断

- **技术可行性：高。** API/运行时版本对齐，源码和 release 完整，固定源码本机构建通过。
- **直接替换可行性：中。** 需要重做部署、状态、UI、联网故障和回滚，不能只替换 DLL。
- **保持助手内完整编辑体验：当前不可直接实现。** 教程把编辑体验外包给网站，公开插件没有提供已验证的第三方写入契约。
- **可靠性：低于当前纯本地配置。** 核心外观同步新增第三方服务可用性依赖。
- **推荐决策：进入 0.5.9 隔离原型，但在真实 BOT 验收和回滚 Gate 通过前，不删除当前 PlayerSkinMod 代码与资源，不构建为正式更新。**

本次仅调查并新增本文档；未修改 0.5.9 程序代码、未部署插件到真实 CS2、未更改玩家 Steam/网站数据、未构建或发布安装程序。
