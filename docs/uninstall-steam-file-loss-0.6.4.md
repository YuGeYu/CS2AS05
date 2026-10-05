# 0.6.4 “卸载插件”误删 Steam 官方文件修复报告

日期：2026-10-05
适用版本：`CS2 人机增强助手 0.6.4`  / 上游 `CS2-Bot-Improver v1.4.5`

## 现象与证据

用户在 Steam 验证 CS2 完整性后，使用旧版 0.6.4 安装器的“卸载插件”，再执行一次验证，Steam 显示“14 个文件验证失败，将重新获取”。截图只包含这条验证结果，没有额外的开发指令。

本机 CS2 根目录为：

```text
D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive
```

只读回读 Steam 客户端 `C:\Program Files (x86)\Steamzhen\logs\content_log.txt` 得到决定性记录：

```text
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_armsrace.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_casual.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_competitive.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_competitive2v2.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_competitive2v2_offline.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_competitive_offline.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_custom.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_deathmatch.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_dm_freeforall.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_retakecasual.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_rush.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_rush_offline.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_teamdeathmatch.cfg"
[2026-10-05 04:43:58] Validation: missing file "game\\csgo\\cfg\\gamemode_workshop.cfg"
[2026-10-05 04:48:30] Validation: full scan in "D:\\SteamLibrary\\steamapps\\common\\Counter-Strike Global Offensive" found 14/4093 mismatching files
[2026-10-05 04:48:32] AppID 730 starting commit ... : 14 updated, 0 moved, 0 deleted files
```

这 14 个路径与旧版 `remove_upstream_package` 的文件删除列表逐项一致，确认问题来自助手卸载逻辑，而不是 Steam 随机误报。旧资源包中的对应 cfg 还会追加 `exec my_bot_*.cfg`，因此仅停止删除会留下被修改的官方文件，仍会触发 Steam 验证异常。

## 根因

旧实现使用固定路径数组直接删除文件和目录，数组把 Steam 官方 `cfg/gamemode_*.cfg` 与插件文件混在一起；安装事务虽然临时备份了被覆盖文件，但成功后会清理临时备份，卸载阶段没有任何 ownership 记录可用于恢复原始字节。旧实现还可能在没有安装证据时删除整个插件目录。

## 修复逻辑

修复位于 [`src-tauri/src/services/cs2.rs`](../src-tauri/src/services/cs2.rs)：

1. 安装事务在写入资源包前记录每个被覆盖文件的原始摘要和原始字节，写入项目自己的 `cfg/cs2as05-install-ledger.json` 与 `cfg/cs2as05-install-backups/`。
2. 卸载优先读取 ownership 清单。文件仍保持助手写入摘要时，原来存在的文件恢复原始字节，安装前不存在的文件才删除；玩家或其他插件改过的文件会保留，并保留清单供后续人工处理。
3. 清单卸载只处理清单中的文件，Inventory Simulator、未知 CounterStrikeSharp 文件和其他玩家文件不受影响。
4. 没有清单的旧安装走保守回退：不删除任何 `cfg/*`、根 `gameinfo.gi` 或 `backup/*`，仅清理明确的旧插件目录、VDF/VPK 文件和助手自己的状态文件。这样旧版本即使没有恢复证据，也不会再次误删 Steam 官方文件。
5. `restore_upstream_panel_cfg_files` 不再覆盖事务中保存的原始备份，保证官方 cfg 的备份确实来自安装前状态。

代码注释明确了 Steam 官方文件保护边界；`tests/installer-contract.spec.ts` 增加了卸载源码契约，防止把官方 cfg/gameinfo 重新放回卸载删除列表。

## 自动化验证

在 `E:\CS2AS05` 执行：

```text
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm run verify
npx vitest run tests/installer-contract.spec.ts --pool=threads --maxWorkers=1
```

结果：

- Rust 主修复回归：`100 passed, 0 failed, 4 ignored`；最后增加原始备份摘要校验后，两个卸载路径定向测试再次通过（各 `1 passed`）。
- 前端完整验证：`56` 个测试文件、`197` 个测试通过；typecheck、Oxlint、ESLint、Vite build 通过。
- 卸载契约单独验证：`10 passed`。
- 新增 Rust 回归覆盖：旧安装无清单时保留 `gamemode_casual.cfg`、`bot_buy.cfg`、`gameinfo.gi` 与 backup；有清单时恢复安装前 cfg/gameinfo，删除助手文件，同时保留 Inventory Simulator 和未知 CounterStrikeSharp 文件。

自动化测试只使用临时 fake CS2 目录，没有修改用户的真实 D 盘游戏目录。真实 Steam 验证、CS2 启动和在线/机器人游戏效果仍需在用户机器上执行。

本次只读回读真实目录确认：Steam 修复后的 `cfg/gamemode_casual.cfg` 已存在（4538 bytes），`gameinfo.gi` 已存在（25985 bytes），旧安装没有 `cs2as05-install-ledger.json` 或 `cs2as05-install-backups`。因此必须使用下面的新安装器重新安装一次，后续卸载才会拥有可验证的恢复清单；本次调查没有替用户目录写入清单或其他文件。

## 新的 0.6.4 安装器

构建产物：

```text
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.6.4_x64-setup.exe
```

构建回读：

```text
大小：119,948,473 bytes
SHA-256：C748391A1756E32CC283D7D7F5463BD81E76ABC5C99760F929ED584807BEA0E5
Authenticode：NotSigned
```

`npm run bundle:desktop` 已完成 NSIS 生成；命令最后因现有 updater 公钥配置与本机缺少 `TAURI_SIGNING_PRIVATE_KEY` 返回退出码 1，未生成 updater `.sig`，不影响上述 NSIS 文件可用于本机和内部测试。

## 本机复测顺序

1. 退出 CS2，使用上面的新安装器覆盖安装 0.6.4。
2. 在助手中安装/启用 BOT 插件，确认 `game/csgo/cfg/cs2as05-install-ledger.json` 出现。
3. 退出 CS2，在助手中执行“卸载插件”。
4. 检查 `game/csgo/cfg/gamemode_*.cfg` 和 `game/csgo/gameinfo.gi` 仍存在；如要检查官方字节，再在 Steam 验证一次，预期不再出现这 14 个缺失项。
5. 验证 `addons/counterstrikesharp/plugins/InventorySimulator` 仍在，助手的插件目录已按 ownership 清单移除。
6. 最后从 Steam 启动 CS2，分别确认普通启动和 BOT 启动。若卸载时某文件在安装后被其他程序改过，助手会保留该文件并在运行日志中记录，需按提示单独处理。
