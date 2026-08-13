# CS2-Skin-Forge v1.8.2 完整移植执行报告

## 交付结论

- 安装器：`E:\CS2AS05\workspace\player-acceptance\skin-forge-0.5.7-v1.8.2-20260811\CS2人机增强助手_0.5.7_x64-setup.exe`
- 版本：助手 `0.5.7`；内置 PlayerSkinMod `1.8.2`；内置 Bot ZIP marker `0.5.7`
- 大小：`113,349,712` bytes
- SHA-256：`40C75AA17C6702F70023F8661F91FBBF1C7C540A739A0C8862944AFC07D48009`
- Authenticode：`NotSigned`
- updater 签名：未生成。`tauri build` 已成功生成 NSIS，随后因本机只有 updater 公钥、缺少 `TAURI_SIGNING_PRIVATE_KEY` 而以非零状态结束；未推送更新源，也未声称已签名。
- 当前状态：**安装候选完成，真实 CS2 游戏内待用户验收**。本轮没有启动真实 CS2、没有部署到玩家目录、没有声明游戏内效果完成。

## 来源与数据 Gate

已固定公开迁移仓库 `https://github.com/kaecho/CS2-Skin-Forge` 的 `v1.8.2`，peeled commit 为 `75f52fbd5fd0616dbbdd09a65c3a1981593400d1`；旧基线 `b2edea17db9128609dd41f726f179cd965206433` 是祖先。原 `emptysuns` 匿名地址 404，已在来源说明中保留这一事实。README 声明 GPL-3.0，但固定提交没有独立 LICENSE，未伪造许可证。

上游快照、来源、构建说明和 vendor SHA-256 清单位于 `third_party/CS2-Skin-Forge/`。本地目录对账为：35 武器、1456 武器涂装记录、1318 唯一武器 paint kit、最大 paint kit 1477、20 刀型、576 刀面、8 手套类型、94 手套涂装、63 角色、94 音乐盒、10461 贴纸、78 挂件；Cologne 2026 冠军签名贴纸 ID `11174–11193` 共 20 项，重复 ID 均为 0。

数据清单和来源哈希记录在 `src/features/skin-forge/data/generated/catalog-manifest.json`。贴纸目录只在首次打开贴纸页时动态加载，每次最多渲染 100 项；`tests/skin-forge-v1.8.2-catalog.spec.ts` 验证目录数量、1477、20 项新增贴纸和 DOM 上限。

## Loadout 与工作台

- schema 已升为 3，保留 schema 1/2、旧 shared maps、旧 knife defindex 写法和未知字段 passthrough。
- CT/T 武器 paint、wear、seed 独立；贴纸、挂件、命名、StatTrak 按插件契约共享。
- 支持图片卡片、名称搜索、分类、刀型/合法刀面、手套/合法涂装、CT/T 角色、音乐盒、随机/自定义模式、编辑弹窗、wear、seed、命名、StatTrak、最多 5 张贴纸及贴纸参数、挂件参数。
- 快速上手、安全说明、关于/来源、部署状态、随机模式和 `skin_menu` / `skin_random` / `skin_reset` 已有玩家可见入口。
- `ForgeImage.vue` 在桌面端通过 Rust 受控缓存，限制 HTTPS、Steamstatic 白名单、MIME、2 MiB 单图、256 MiB 缓存、6 路并发、重定向白名单和固定 fallback；开发页面保留直链回退。

`tests/skin-forge-loadout-adapter.spec.ts` 与 `tests/skin-forge-v1.8.2-catalog.spec.ts` 共 9 项通过，包含 v1.8.2 golden JSON 逐字段往返和 CT/T 防串队。

视觉验收记录：`1440x900`、`1100x700`、`760x800` 无横向溢出；编辑器边界为 `140,40,1160x820`；贴纸首次动态加载约 346 ms；`NiKo Champion Cologne 2026` 搜索返回 4 项；reduced-motion 媒体偏好实测生效。截图保存在 `workspace/skin-forge-v1.8.2-visuals/`。最终刷新本地页时内置浏览器 URL 策略阻止了再次导航，未绕过该限制；前述截图和最终类型/lint/build 链共同作为视觉回归证据。

## PlayerSkinMod 与安装资源

固定提交的上游 C# 文件仍有 1.8.1/1.8.2 版本不一致，已保留 vendor 原件并使用窄版本 patch；重新构建得到 net8.0 DLL，无本项目编译错误。安装资源哈希：

| 文件 | SHA-256 |
| --- | --- |
| `PlayerSkinMod.dll` | `0DAC2B47275EC6AD308F8F5F4797140D1AA8BA63DA93F2840DA90BA9A3838388` |
| `PlayerSkinMod.json` | `D9D686289899663C92795C04738B563E065DF8B7170DF02B8518F2C666A19706` |
| `skins_en.json` | `DFD0A2CB407065FC0B567E67891FC7E2DFA3AB1C794D1708FF1DDBCBAD29F98B` |
| `CS2BotImprover.zip` | `43EC171AA3D6530B68DEFB4B2EF1911C1AB13531F4907EAF5B24B54A12DA5B95` |

`generate-plugin-manifest.ps1` 已同步到 schema 2，保留 BotVision 条目和来源哈希；旧 ZIP fixture 已更新为新摘要。Rust 部署仍保留 canonicalize、路径边界、CS2 运行阻止、CounterStrikeSharp 不覆盖、原子写入、整目录备份和失败恢复。

## 自动化证据

- `npm run workspace:check`：通过，registry valid。
- `npm run typecheck`：通过。
- `npm run lint:oxlint`：0 warnings / 0 errors。
- `npm run lint:eslint`：通过。
- `npm test -- --run`：37 个文件、136 项通过。
- `npm run build:web`：通过；贴纸 chunk 约 2.96 MB，属于完整目录动态资源，构建仅报告体积提示。
- `cargo fmt --check`：通过。
- `cargo clippy --all-targets -- -D warnings`：通过；第三方 demoparser 输出 10 条既有普通 warning，但未转为本 crate 失败。
- `cargo test --lib`：此前完整结果 74 passed / 0 failed / 3 ignored；版本回调后 `services::cs2::tests::` 7 passed / 0 failed，覆盖最终 0.5.7 ZIP marker。3 个 ignored 测试需要真实 Demo/真实 CS2 环境。
- `git diff --check`：通过。

## 最终安装冒烟

最终安装目录：`E:\CS2AS05\workspace\player-acceptance\install-smoke-0.5.7-v1.8.2-final`。

静默安装退出码 0；安装目录包含主 EXE、卸载器、资源目录和 `CS2BotImprover.zip`。从安装目录启动时 PID `11844` 有真实窗口句柄，标题为“CS2人机增强助手”，进程响应正常并拉起 WebView2；关闭后主进程和子进程均退出。安装目录回读确认助手 `0.5.7`、PlayerSkinMod `1.8.2`、Bot marker `0.5.7` 以及上表全部哈希。调用该目录 `uninstall.exe /S` 退出码 0，目录和 HKCU 卸载登记均已消失。

## 用户只需执行的真实验收

1. 安装上方 NSIS，打开“皮肤工坊”，确认目录和图片可浏览；保持 PlayerSkinMod 未部署，进入本地 `-insecure` BOT 对局，确认原有 BOT/命令功能无回归。
2. 退出 CS2，在安装版助手选择 CS2 根目录并执行“部署 / 更新插件”。
3. 在图文工作台分别设置 CT/T 不同武器、刀、手套、角色、音乐盒，并配置贴纸、挂件、命名和 StatTrak。
4. 点击“应用装备”，重启本地对局并重生；必要时执行 `skin_menu`、`skin_random`、`skin_reset`。
5. 保存三类证据到 `workspace/player-acceptance/skin-forge-0.5.7-v1.8.2-20260811/`：JSON 回读、插件日志、游戏内可见截图/录像。

只允许离线/本地 `-insecure` 环境，不连接 VAC 保护服务器。缺少任一类证据，都只能称为“安装候选完成，游戏内待验收”。
