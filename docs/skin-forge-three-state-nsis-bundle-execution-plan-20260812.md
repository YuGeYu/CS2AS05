# 皮肤工坊五类装备三态显示与 0.5.7 NSIS 安装器交接方案（2026-08-12）

> 文档角色：本文件由【方案制定 AI】新增，交给不共享上下文的【实际执行 AI】执行。
>
> 工作目录：`E:\CS2AS05`
>
> 本轮目标：调查并在上游契约允许的范围内，让武器、手套、角色、音乐盒、刀具都正确表达“每次重生随机 / 自定义且有具体值 / 自定义但未选择”三态；随后使用当前版本号执行完整 NSIS bundle。
>
> 本轮不做：不修改版本号（必须保持 `0.5.7`）、不修改 Demo/比分/Rating/BOT/Watcher/Viewer/更新器业务、不复制旧安装器冒充新候选、不提交/推送/部署。

## 1. 执行前基线与证据

执行 AI 必须先读取当前工作树和以下文件，不回退用户已有改动：

- `src/types/skin-forge.ts`
- `src/features/skin-forge/components/ForgeWorkbench.vue`
- `src/features/skin-forge/player-skin-mod-adapter.ts`
- `src/features/skin-forge/data/catalog.ts`
- `src/stores/skinForge.ts`
- `third_party/CS2-Skin-Forge/upstream/Panel/src/utils/types.ts`
- `third_party/CS2-Skin-Forge/upstream/Panel/src/App.tsx`
- `third_party/CS2-Skin-Forge/upstream/Panel/src/components/KnifePanel.tsx`
- `third_party/CS2-Skin-Forge/upstream/Panel/src/components/GlovePanel.tsx`
- `third_party/CS2-Skin-Forge/upstream/Panel/src/components/AgentPanel.tsx`
- `third_party/CS2-Skin-Forge/upstream/Panel/src/components/MusicKitPanel.tsx`（若路径不同，用 `rg -n "musicKit|useRandom"` 定位）
- `third_party/CS2-Skin-Forge/upstream/addons/counterstrikesharp/plugins/PlayerSkinMod/Services/LoadoutService.cs`
- `package.json`
- `src-tauri/tauri.conf.json`

当前项目版本应为：`package.json.version=0.5.7`、`src-tauri/tauri.conf.json.version=0.5.7`。若发现不一致，先停止并报告，不擅自改版本。

已知当前模型：`Loadout.mode` 是全局 `custom|random`；`musicKit` 是全局具体值；刀具、手套、角色和武器具体配置存于 CT/T 或武器记录中。当前 `ForgeWorkbench.vue` 已有音乐盒三态摘要，但武器摘要仍是“n 把已配置”，刀具/手套/角色摘要只按具体值查找，未完整区分全局 random 与 custom 未选择。

## 2. 上游能力闸门：先判断能否实现

上游 Panel/PlayerSkinMod 当前已确认 `useRandom:boolean` 是全局随机开关，`musicKit:number` 与刀具、手套、角色、武器字段分开。上游初始值包含 `useRandom=true`、`musicKit=-1`、刀具索引 `-1`、手套索引 `-1`、角色模型 `-1`。上游选择具体项目时将 `useRandom:false`；取消按钮在部分面板会把字段清空并按上游语义恢复随机。

执行 AI 必须把能力分为两类：

1. **上游明确支持的三态**：可以实现 UI 三态，但序列化必须保持原字段契约。
2. **上游只有全局随机、没有分类独立随机的字段**：不得为每个类别新增伪造的 `weaponMode/gloveMode/agentMode/knifeMode` 写入 PlayerSkinMod。此时“每次重生随机”只表示插件全局 `useRandom=true`，各类别只能显示同一个全局随机状态；custom 下才显示具体值或未选择。

如果上游实际只提供二态（随机/具体），而“custom + 未选择”会被插件解释成随机或默认装备，必须保留上游真实语义，显示“未选择”只作为本地草稿状态，并在写入/回读时标注无法区分。不能声称游戏内存在第三种独立状态。

## 3. 三态统一展示契约

定义只读展示函数，避免五处模板各自判断：

```text
global mode=random
  -> 每次重生随机
global mode=custom && concrete value exists
  -> 具体值名称
global mode=custom && no concrete value
  -> 未选择
```

具体类别规则：

- **刀具**：当前 team 的 `knife.index >= 0` 且能映射目录时显示刀具名称；否则 custom 显示“未选择刀具”。若 `mode=random`，显示“每次重生随机”。刀面只作为名称补充，不能把没有刀具的 `paintKit=-1` 当成具体刀具。
- **手套**：`gloves.index >= 0` 且 `defindex` 有目录项时显示手套名称；否则 custom 显示“未选择手套”；random 优先显示随机。
- **角色**：`agent !== null` 且路径/目录项有效时显示角色名称；否则 custom 显示“未选择角色”；random 优先显示随机。
- **音乐盒**：`musicKit !== null` 且有目录项时显示音乐盒名称；否则 custom 显示“未选择音乐盒”；random 优先显示随机。
- **武器**：不能只用“已配置数量”。对每一把武器显示：random 时“每次重生随机”；custom 且 weapon record 存在且有具体 `paintKit>=0` 时显示皮肤名称；custom 且该武器没有配置或 `paintKit=-1` 时显示“未选择该武器皮肤”。目录标题可保留“已配置数量”，但必须另外提供三态摘要/筛选状态。

如果上游武器随机并非由 `useRandom` 对每把武器独立控制，必须在文档和 UI 文案中写明这是全局随机，不得暗示“仅这把武器随机”。

## 4. UI 与可访问性实现边界

建议新增纯函数模块 `src/features/skin-forge/loadout-display.ts`（或沿用仓库已有 formatter 文件），输入 `Loadout`、team、类别和目录，输出 `{ state: 'random'|'custom-value'|'custom-empty', label, valueId? }`。函数必须纯、可单测，不得触发 store 写入。

`ForgeWorkbench.vue` 右侧摘要、武器卡片、刀具/手套/角色/音乐盒 picker 的选中状态都使用该 formatter。保留现有安全 gate、disabled 和 `chooseCustom()`：浏览可以看，未部署时不能通过显示层改动状态。

如上游允许 custom-empty，五类 picker 均提供明确的“取消选择”入口，并在取消后立即显示“未选择”。如上游不允许独立 custom-empty，则取消动作只能恢复上游定义的随机/默认 sentinel，并显示上游真实状态，不得伪造第三种写入状态。

三态不是额外的第三个 `mode` 值；不得把 `mode` 改成 `random|custom|empty`，因为 PlayerSkinMod 只接受现有 `useRandom` 契约。三态是“全局模式 + 类别具体值是否存在”的派生状态。

## 5. adapter、上游和回读测试

更新或新增 `tests/skin-forge-three-state-display.spec.ts`，至少覆盖：

- 五类在 `mode=random` 时均显示“每次重生随机”。
- 五类在 `mode=custom` 且具体值存在时显示正确目录名称。
- 五类在 `mode=custom` 且无具体值时显示对应“未选择”文案。
- 武器 `paintKit=-1`、未创建 weapon record、具体 paintKit 三种 custom 输入。
- CT/T 独立刀具、手套、角色值不会串队伍；音乐盒全局值不会被 CT/T 切换破坏。
- `toPlayerSkinModLoadout` / `fromPlayerSkinModLoadout` 的 `useRandom`、`musicKit=-1`、刀具/手套/角色 sentinel round-trip。
- 若上游无法保持 custom-empty 与 random 的差异，测试必须明确断言“回读后降级为上游状态”，并在报告中列为限制。
- disabled/safety gate 下 formatter 可以读，但任何取消/选择不改变 store。

同时运行现有：

```powershell
npm run typecheck
npm run lint
npm test -- --run
cargo fmt --check
cargo test --manifest-path src-tauri/Cargo.toml --lib
git diff --check
```

## 6. 版本不变的完整 NSIS bundle

业务和测试完成后，重新读取版本值并记录：

```powershell
node -p "require('./package.json').version"
Select-String -Path src-tauri\tauri.conf.json -Pattern '"version"'
```

两个结果必须都是 `0.5.7`。确认 `src-tauri/tauri.conf.json` 的 `bundle.active=true`、`targets="nsis"`、资源列表和 updater 配置未被本轮意外改动。

完整构建命令：

```powershell
npm run bundle:desktop
```

该脚本等价于 `tauri build bundle`，会先执行 `beforeBuildCommand` 的 `npm run build:web`，再生成 NSIS 安装器。不要使用 `npm run build:desktop`，它是 `--no-bundle` 裸 EXE，不能作为本轮交付安装程序。

构建后定位新产物（不要复用旧文件）：

```powershell
Get-ChildItem src-tauri\target\release\bundle\nsis -Filter *.exe |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 5 FullName,Length,LastWriteTime
```

产物必须来自本次构建时间窗口，并单独记录绝对路径、文件大小、SHA-256：

```powershell
Get-FileHash <新生成的NSIS安装器绝对路径> -Algorithm SHA256
```

版本不变不代表可以覆盖旧候选。将本次验证记录放入新的报告文件，例如 `docs/skin-forge-three-state-nsis-bundle-execution-report-20260812.md`；不要修改旧报告来伪造时间线。

## 7. 安装器验收和证据边界

至少做一次隔离目录安装事务：安装到临时路径，读取已安装资源版本/哈希，启动安装后的程序并确认窗口可创建，退出后确认没有残留主进程或 WebView 子进程，再卸载。安装器可能未签名或 updater 私钥缺失时，报告必须明确写出 `Authenticode` 和签名状态，不能称为正式发布版。

桌面自动化/安装验收不能证明真实 CS2 生效。真实玩家验证仍需用户在匹配版本 PlayerSkinMod + CS2 `-insecure` 中执行：部署插件，选择五类三态，回读 `player_loadout.json` 与插件日志，并观察 CT/T 重生结果。若上游不支持 custom-empty 的持久区分，必须把该项标成“上游限制”，不能用 UI 文案替代游戏证据。

## 8. 停止条件与最终交付格式

遇到以下任一情况停止并报告：发现版本不是 `0.5.7`；上游字段无法证明三态；需要修改 PlayerSkinMod C# 才能伪造契约；三态显示与回读不一致；NSIS 构建失败且原因涉及环境/签名/资源缺失；安装事务不能清理进程。不要为了通过测试修改无关模块。

最终报告必须包含：

1. 上游能力结论：哪些类别真实支持三态，哪些只是全局随机派生，哪些因上游限制未实现。
2. 修改文件和三态 formatter/adapter 契约。
3. 完整测试命令与通过/失败/忽略计数。
4. NSIS 安装器绝对路径、构建时间、大小、SHA-256、签名状态和隔离安装结果。
5. 未完成的真实 CS2 证据及由用户执行的步骤。

