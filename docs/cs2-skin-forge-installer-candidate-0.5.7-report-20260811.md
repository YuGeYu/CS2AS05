# CS2-Skin-Forge 0.5.7 安装候选报告

日期：2026-08-11（Asia/Shanghai）。

## 候选

- 版本：`0.5.7`
- 交付安装器：`E:\CS2AS05\workspace\player-acceptance\skin-forge-0.5.7-20260811\CS2人机增强助手_0.5.7_x64-setup.exe`
- 原始 NSIS：`E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.7_x64-setup.exe`
- 大小：`112509757` bytes
- SHA-256：`20C82422FC01B9D721DDBAAFD8C71EC6EDCE012645294124CBE6EF80B06FDE23`
- FileVersion/ProductVersion：`0.5.7`
- Authenticode：`NotSigned`
- Tauri updater 签名：未生成；构建机发现公钥但没有 `TAURI_SIGNING_PRIVATE_KEY`。不得推送生产更新源。

## 已完成的非游戏验收

- `npm run verify`：通过。
- workspace check：通过。
- TypeScript typecheck：通过。
- Oxlint：0 warning / 0 error。
- ESLint：通过。
- Vitest：36 files / 133 tests 通过。
- Web build：通过。
- `npm run bundle:desktop`：Rust release 和 NSIS 均成功生成；命令最终仅因缺 updater 私钥返回非零。
- NSIS 隔离静默安装：exit `0`。
- 隔离安装目录：`E:\CS2AS05\workspace\player-acceptance\install-smoke-0.5.7`。
- 安装后主 EXE、uninstall.exe 和 `resources\skin-forge\PlayerSkinMod` 均存在。
- 安装后启动冒烟：进程路径来自隔离安装目录，窗口标题为“CS2人机增强助手”，`Responding=True`；随后正常关闭本轮启动的进程。
- 已为用户准备场景 A：确认 `cs2.exe` 未运行后，将工程阶段的 PlayerSkinMod 完整移动到 `E:\CS2AS05\workspace\player-acceptance\skin-forge-0.5.7-20260811\pretest-PlayerSkinMod-backup`；CS2 插件目标当前不存在，其他 CounterStrikeSharp/Bot 插件未改动。

安装后 PlayerSkinMod 资源：

| 文件 | 大小 | SHA-256 |
| --- | ---: | --- |
| `PlayerSkinMod.dll` | 75776 | `47BF3733D3091D3EAB9E4B86052CBF77A002CFEBAACDEB8D8C43136FF2ADFDEA` |
| `PlayerSkinMod.json` | 413 | `16751DE81C54CAE5505B5FC0A3FC85C7E975750CD69C9AF3DE72919165C9072C` |
| `skins_en.json` | 334893 | `21A67383A87B9DFD1C83D85EE7096D1AC459AAE519572149265E5B5B57A41251` |

## 尚待用户真实游戏验收

- 未部署 PlayerSkinMod 时原有 BOT/助手功能。
- 通过安装版助手一键部署 PlayerSkinMod。
- CT/T 武器、刀具、手套、角色、音乐盒、命名和 StatTrak。
- watcher 热重载、`skin_menu`、`skin_random`、`skin_reset`。
- PlayerSkinMod 与现有 Bot Improver 运行时共存。
- 当前 UI 的贴纸/挂件完整操作入口仍可能阻断完整 PASS，必须按实际界面记录，不能手改 JSON 冒充通过。

本候选只用于本地真实游戏验收，不是签名发布版本。
