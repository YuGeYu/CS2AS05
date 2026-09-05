# 为什么 0.5.9 必须迁移到 cs2-css-inventory-simulator

日期：2026-08-17

## 结论

当前皮肤工坊不是简单的“完成度不够”，而是核心抽象选错了：助手承担了一整套饰品数据库和编辑器，游戏插件却只在武器生成后反复改属性。它能在部分路径显示外观，但不是稳定的 CS2 库存实现。

`ianlucas/cs2-css-inventory-simulator@5e3c96283b3d3f5aeba44822a38031df2e213376` 直接接管 CS2 库存槽、`CEconItemView` 和按 SteamID 的玩家库存。0.5.9 应停止继续扩建当前 PlayerSkinMod 路线，迁移到 Inventory Simulator。保留旧实现只用于迁移回滚，不再作为双引擎长期选项。

## 当前实现为什么失败

### P0：把“写入 JSON”错误地呈现成“装备已经应用”

当前助手保存成功的权威状态只是：

```text
player_loadout.json 写入成功
JSON 可解析
SHA-256 回读一致
```

`SkinForgeView.vue` 显示“应用装备”和“已写入”，`skinForge.ts` 也在写盘后提示“装备已写入游戏”，但没有来自 CounterStrikeSharp 或 CS2 实体的应用回执。文件正确不等于当前武器、重生路径、刀、手套或角色已经应用。产品把中间状态当作最终状态，是根本性的可信度问题。

Ian Lucas 的插件在玩家连接时按 SteamID 获取库存，调用 `CCSPlayerInventory::SendInventoryUpdateEvent`，并在 `GiveNamedItem` 前置 hook 与 `GetItemInLoadout` 后置 hook 中提供对应库存槽的 `CEconItemView`。它的决定性状态位于游戏库存层，而不是面板文件层。

### P0：当前插件在实体生成后补写，稳定性依赖竞态

PlayerSkinMod 自己的注释已经承认一次应用会输给客户端初始快照。当前路径是：

```text
GiveNamedItem Post
-> 立即 ApplySkinForPlayer
-> 下一帧再次 ApplySkinForPlayer
-> 250 ms 后第三次 ApplySkinForPlayer
```

玩家重生时，刀和手套也会立即执行并在 100 ms 后再次执行；已有枪械在 200 ms 后再补一次。代码用计时器次数覆盖时序不确定性，而不是消除不确定性。这就是默认皮肤偶发回退、刀/手套延迟和后续不断增加“再补一遍”的根源。

Inventory Simulator 在物品创建前把正确的自定义 `CEconItemView` 传给 `GiveNamedItem`，并在 CS2 查询库存槽时直接替换返回值。正确物品从创建时就成立，不需要靠三次事后覆盖赌执行顺序。

### P0：玩家身份使用 slot 0，而不是 SteamID

助手序列化默认固定为：

```ts
toPlayerSkinModFile(loadout, slot = 0)
```

PlayerSkinMod 运行时又用 `player.Slot` 读取 `_playerLoadouts`。这把配置绑定到临时连接槽位，而不是玩家身份：

- 假设本地主玩家永远是 slot 0。
- 多人本地房间无法自然区分玩家库存。
- 断线、重连、槽位复用时没有身份连续性。
- 配置不能随同一个 Steam 玩家在不同实例中稳定识别。

Inventory Simulator 的远端和本地文件模式都以 SteamID64 为键；`CEconItemView` 缓存键也是 `(SteamID, team, slot)`。这是库存系统必须具备的身份模型。

### P1：宣称 CT/T 独立，实际静默合并关键字段

界面写着“CT 与 T 阵营各自保存”，但适配器和 store 会主动把同一 defindex 的以下字段同步到另一阵营：

```text
stickers
keychain
nameTag
statTrak
```

`player-skin-mod-adapter.ts` 又用 `seen = new Set<number>()`，只保留先遇到的那份详细配置。这意味着玩家在 CT/T 看似编辑两件物品，实际部分字段只有一个共享值。UI 表达和持久化契约不一致。

Inventory Simulator 的 API 把 `ctWeapons`、`tWeapons`、`knives`、`gloves`、`agents` 分开，每个装备项拥有自己的 `uid`、`hash`、paint、wear、seed、StatTrak、贴纸和挂件集合，没有必要让面板偷偷合并两份物品。

### P1：命令反馈声称持久化，实际只改内存

当前帮助页告诉玩家使用：

```text
skin_random
skin_reset
```

但插件实现只是：

```text
skin_random -> loadout.UseRandom = true
skin_reset  -> _playerLoadouts.Remove(player.Slot)
```

它们都没有更新 `player_loadout.json`。下一次文件 watcher 重载、`skin_menu` 重读文件或插件重启，旧配置可以再次回来。“已启用随机”“已重置全部皮肤”的聊天反馈并不代表持久状态。

Inventory Simulator 的 `!ws` 语义是重新获取按 SteamID 保存的权威库存。它仍需在 0.5.9 中显式启用 `invsim_ws_enabled true`，但命令的状态模型是明确的远端刷新，不是临时修改一个 slot 字典。

### P1：StatTrak 是会丢失的进程内计数

PlayerSkinMod 击杀时只执行 `statTrak.Count++` 并更新当前武器属性，没有回写 JSON。CS2/插件重启或文件重载后，计数回到面板最后保存的值。

Inventory Simulator 的装备有稳定 `uid`，击杀后既更新当前 `CEconItemView`，也可以按该 `uid` 调用服务端 StatTrak 增量接口，并自带无密钥请求限流。0.5.9 首版是否启用远端增量可以单独决定，但它至少具备正确的物品身份和持久化契约。

### P1：助手重复维护一个庞大且易过期的饰品数据库

当前前端生成目录约 `3.93 MB` TypeScript 源码，包含：

```text
35 武器
1456 武器涂装记录
20 刀型
576 刀面
94 手套涂装
63 角色
94 音乐盒
10461 贴纸
78 挂件
```

与此同时插件还带一份 `skins_en.json` 和自己的 C# 静态表。助手、插件、Steam 图片地址和上游生成脚本形成多个真相源。每次 CS2 新增饰品或改变 schema，我们都要同步数据、生成代码、处理图片缓存、重新构建安装器并验证适配器。

Inventory Simulator 把物品目录、检视链接解析、库存制作和 Steam 登录集中在专门网站；游戏插件只消费版本化的 `EquippedV5Response`。我们不应该在 CS2 人机助手里继续维护一个较差的饰品数据库产品。

### P1：当前模型不是完整库存模型

当前 `WeaponLoadout` 只有 defindex、paint、wear、seed、名称、StatTrak、最多五张贴纸和一个挂件；根模型只覆盖武器、刀、手套、角色、音乐盒。它没有稳定物品 UID/hash、收藏品、涂鸦、喷漆次数等库存语义。

Ian Lucas 的 `InventoryItem` 包含：

```text
uid / hash / def / paint / wear / seed
stattrak / nametag / stickers / keychains
musicId / graffiti tint / charges
```

响应模型还原生区分 CT/T 武器、刀、手套、角色、音乐盒、收藏品和涂鸦。以后继续给当前模型打字段补丁，只会再次追赶它已经解决的问题。

### P2：维护成本与产品价值倒挂

为了支撑 PlayerSkinMod，我们已经额外维护：

- Vue 图文工坊和多组编辑弹窗。
- 多版本 loadout migration 和双向 adapter。
- 近 4 MB 生成目录。
- Steam 图片代理、2 MiB 单图限制、256 MiB 缓存和并发控制。
- Rust 插件哈希、部署、备份、回滚和 JSON 验证。
- fork 后的 C# 插件、版本 patch、静态表和构建记录。

这些工程量没有换来库存层的正确性。继续投入相当于自行重写 Ian Lucas 已经提供的“网站 + 库存 API + CounterStrikeSharp 插件”，而且我们的项目主业并不是饰品模拟器。

## Ian Lucas 实现正确在哪里

1. **以库存槽为核心。** 使用 `CCSPlayerInventory::GetItemInLoadout` 和默认 loadout slot，而不是只看生成后的武器名称。
2. **以 `CEconItemView` 为载体。** 通过游戏 item schema 构造/复制物品视图，统一应用 defindex、quality、AccountID 和动态属性。
3. **以 SteamID 为身份。** 配置属于玩家，不属于临时 slot。
4. **以物品 UID/hash 为状态。** 可以判断物品是否变化并持久更新 StatTrak、喷漆等物品状态。
5. **生命周期完整。** 连接拉取、刷新、库存更新事件、断线清理、实体删除清理和热卸载都有明确路径。
6. **处理客户端材质缓存。** 它记录 `(paint, wear)` 与贴纸组合，必要时微调 wear，解决相同材质键导致的贴纸/纹理不刷新问题。
7. **功能面更完整。** 武器、刀、手套、角色、音乐盒、收藏品、涂鸦、贴纸、挂件和检视链接都落在同一个库存模型里。
8. **上游维护能力更强。** `3.1.0` 已对齐 CounterStrikeSharp `1.0.371`，固定源码在本机以 .NET 10 构建为 0 warning / 0 error。
9. **许可清晰。** 固定提交带完整 MIT `License.txt`，便于合规打包和后续跟随上游。

## 为什么是“必须迁移”，而不是“两套并存”

两套插件同时 hook `GiveNamedItem`，并都会写 `CEconItemView`、动态属性、刀、手套、角色和 StatTrak。PlayerSkinMod 还会在后续帧反复覆盖 Inventory Simulator 提供的物品。双引擎没有稳定优先级，只会制造新的竞态。

长期保留两套产品模式也会迫使我们同时维护两套目录、两套 UI、两套配置、两套诊断和两套玩家说明。旧方案的唯一合理用途是迁移失败时恢复，而不是继续面向玩家提供。

## 迁移后的产品边界

0.5.9 应把“皮肤工坊”改为“库存换肤”服务页：

1. 检测 CounterStrikeSharp、Inventory Simulator、gamedata、受管配置和服务连通性。
2. CS2 退出后备份并停用 PlayerSkinMod，再原子部署固定 Inventory Simulator。
3. 使用系统默认浏览器打开 `inventory.cstrike.app` 完成 Steam 登录和饰品制作；助手不承载登录页面，不接触 Steam 密码。
4. 显示连接自动同步、`!ws` 刷新、冷却和重生生效状态。
5. 将 `invsim_ws_enabled` 显式设为 `true`；首版保守保持 `invsim_ws_immediately false`。
6. 第三方服务不可用时只影响换肤同步，不阻断 BOT、Demo、启动和其他助手功能。
7. 保留一次可验证的“恢复迁移前版本”，直到新方案真实 BOT 验收通过。

## 不得虚构的部分

- 网站/API 是新的外部依赖，可靠性低于纯本地文件，必须显示网络故障状态。
- 玩家需要接受在第三方网站进行 Steam OpenID 登录和按 SteamID 同步库存。
- 插件含多项 Windows gamedata signature，CS2 更新后仍可能失效，必须跟随上游并做加载诊断。
- `!ws` 在 3.1.0 默认只打印地址，必须由我们显式启用后才会刷新。
- 在真实 `-insecure` 本地 BOT 中完成枪、刀、手套、角色、音乐盒、贴纸/挂件、断网和回滚验证前，只能称为迁移候选。

这些代价是真实的，但它们比继续维护一个身份错误、状态不持久、依赖时序补写的自制库存系统更可控。

## 决策

0.5.9 后续换肤开发以 `cs2-css-inventory-simulator 3.1.0` 固定提交为新基线。停止给 PlayerSkinMod 增加字段、目录、图片缓存和游戏内补写逻辑；下一阶段直接做可回滚迁移、受管配置、服务状态和真实本地 BOT 验收。

本报告是架构决策与责任复盘，不代表迁移已经实现或已经通过真实游戏验收。
