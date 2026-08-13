# 皮肤工坊编辑提交、图片重试、音乐盒状态与取消选择交接方案（2026-08-12）

> 文档角色：本文件由【方案制定 AI】新增，交给不共享上下文的【实际执行 AI】实施。
>
> 工作目录：`E:\CS2AS05`
>
> 本轮只处理皮肤工坊四项问题：编辑完成永久“正在提交”、失败图片按需重试、音乐盒随机状态展示、具体选择自动切换自定义及取消选择。不要修改 Demo 比分、Rating、BOT、Watcher、Viewer、更新器或无关页面。

## 1. 交接规则与当前基线

执行前必须重新读取当前工作树，保留用户和其他 AI 的既有改动，不执行 `git reset`、`git checkout`、清理或回退。当前项目是 Vue 3 + Tauri 2 + Pinia + Rust，相关目录如下：

- `src/features/skin-forge/components/WeaponEditorDialog.vue`
- `src/features/skin-forge/components/ForgeWorkbench.vue`
- `src/features/skin-forge/components/ForgeImage.vue`
- `src/features/skin-forge/image-cache.ts`
- `src/features/skin-forge/player-skin-mod-adapter.ts`
- `src/stores/skinForge.ts`
- `src/types/skin-forge.ts`
- `third_party/CS2-Skin-Forge/upstream/Panel/src/utils/types.ts`
- `third_party/CS2-Skin-Forge/upstream/addons/counterstrikesharp/plugins/PlayerSkinMod/Services/LoadoutService.cs`

已有 PlayerSkinMod readiness gate、Tauri 本地图片协议和 HTTPS/域名/MIME/大小限制必须继续保留。插件未部署时 UI、Pinia、Rust 三层仍然 fail-closed；不得为了修提交而绕过 `[PLAYER_SKIN_MOD_REQUIRED]`。

现有上游契约已经确认：Panel 的 `musicKit:number` 与 `useRandom:boolean` 分离；C# `LoadoutService` 读取 `useRandom`，`musicKit=-1` 为无具体音乐盒回退值。取消选择的具体 sentinel 仍必须通过当前 adapter 和上游测试确认，禁止凭经验写入未知字段。

## 2. P0：修复“完成编辑”永久正在提交

### 2.1 根因

`WeaponEditorDialog.vue` 当前 `save()` 先将本地 `submitted` 设为 `true`，只发出 `save` 事件；父层 `ForgeWorkbench.vue` 没有成功/失败 ack，也没有异常恢复路径。于是任何事件链中断、父层静默 return、异常或组件状态变化都会留下永久 spinner。完成编辑是本地 draft 提交，不应等待图片目录、网络请求、Rust 写盘或贴纸加载。

### 2.2 建议状态机

将子组件事件改为带一次性回调：

```ts
const emit = defineEmits<{
  close: []
  save: [
    weapon: WeaponLoadout,
    done: (result: { ok: true } | { ok: false; message: string }) => void
  ]
}>()
```

实现要求：

1. 点击前检查 `props.disabled`、已有提交状态和 draft `defindex`；锁定状态必须按钮不可提交。
2. 点击后只允许一次提交，设置 `submitted=true`、`aria-busy=true`，保留 draft 的完整深拷贝。
3. `done` 必须幂等，父层第一次回调后忽略后续回调。
4. 父层成功执行 `forge.updateWeapon(activeTeam, weapon)` 后调用 `done({ ok: true })`，再关闭模态框。
5. 父层失败或捕获异常时调用 `done({ ok:false, message })`；子组件恢复按钮、保留草稿并显示可读错误。
6. 增加 10 秒 watchdog。10 秒未收到 ack 时恢复按钮并显示“提交确认超时，请重试”，同时记录可诊断错误；不能继续无限 spinner。
7. 即使没有选择任何皮肤、贴纸或其他字段，也必须成功保存并关闭；完成编辑不得依赖 catalog pending 状态。
8. 按钮保留 `data-testid="weapon-editor-save"`，设置 `aria-busy` 和清晰的 title；加载图标在 reduced-motion 下不得无限旋转。

父层若 `updateWeapon` 当前是同步函数，应明确捕获同步异常；若未来变为 Promise，必须 `await` 后再 ack。不要用固定延时伪造成功。

### 2.3 P0 测试

新增 `tests/skin-forge-weapon-editor-submit.spec.ts`，至少覆盖：

- AK-47、M4A1-S 及一个非默认枪械打开模态框，点击完成编辑只触发一次 save。
- payload 的 `defindex` 与当前枪械一致，空选择 payload 也合法。
- 父层成功 ack 后模态框消失，Pinia 武器更新、`dirty=true`、模式为 `custom`。
- 父层失败或抛错后按钮恢复，错误可见，草稿仍在。
- 不调用 ack 时 10 秒 watchdog 恢复按钮。
- 贴纸 catalog Promise 永不结束时，完成编辑仍立即完成。
- `disabled=true` 时按钮不可点且 save 不触发。
- 连续双击不产生第二个保存。

验收命令：`npm run test -- tests/skin-forge-weapon-editor-submit.spec.ts`；再运行项目完整前端测试与类型检查。失败时保留最小复现输出，不扩大修改范围。

## 3. P1：失败图片在玩家触达时单项重试

### 3.1 目标行为

约 200 张贴纸中少量加载失败可以接受一次网络失败，但玩家滚动到失败项时必须能只重试该 URL。失败项不得永久黑名单，不得刷新整个目录，不得让失败项无限 spinner。

### 3.2 image-cache.ts 契约

在现有最大并发 4、单请求 15 秒超时、`[IMAGE_BUSY]` 最多两次自动重试基础上：

- 增加成功结果 `resolvedByUrl: Map<string,string>`，同 URL 成功结果立即复用。
- 增加 `retryForgeImage(url)`，仅清除该 URL 的失败状态并重新入队。
- 同一 URL 的双击重试必须共享 in-flight Promise，不能产生重复下载。
- 失败只记录 attempts/lastFailedAt 用于节流；请求完成（成功或失败）都要释放 `inFlight` 和 active 槽位。
- 快速切换目录时，旧页面任务不得长期占满并发。优先使用 AbortController、订阅者计数或可见项优先级；旧任务结果必须做 stale-result 拒绝。
- 保留 HTTPS、允许域名、重定向、MIME、2 MiB 单图和本地 asset scope 限制，不接受 WebView 任意 URL。

### 3.3 ForgeImage.vue 交互

失败状态显示“图片加载失败”和至少 44px 可点击的“重试加载”按钮，按钮只调用当前 URL 的 retry。重试期间显示有限时加载状态；再次失败仍回到可重试状态。成功、失败、重试中三种状态要有 `aria-live` 或可读状态，reduced-motion 时使用静态文字/点状状态。

### 3.4 图片测试

更新 `tests/skin-forge-image-loader.spec.ts`：

- 首次失败显示 fallback 和重试按钮。
- 点击重试只重新请求目标 URL，不影响其他目录项。
- 同一 URL 并发点击只发起一次请求。
- 第二次成功写入成功缓存，后续切换不再请求。
- 第二次失败释放 active/inFlight，界面不永久转圈。
- 快速切换后旧请求结果不能覆盖当前可见项。
- 超时、非法 MIME、越过大小限制仍按安全错误处理，不因 retry 绕过限制。

## 4. P1：音乐盒随机/自定义/未选择状态

### 4.1 UI 展示规则

`ForgeWorkbench.vue` 右侧“当前装备”音乐盒摘要不能只用 `musicKit` 查名称。统一按以下优先级：

```text
mode === 'random'                         -> 每次重生随机
mode === 'custom' && musicKit !== null    -> 具体音乐盒名称
mode === 'custom' && musicKit === null    -> 未选择音乐盒
```

“默认音乐盒”不得再作为随机模式的显示文本。只有确实存在上游默认值且尚未选择时才可使用默认文案，并应优先改成“未选择音乐盒”以免混淆。

### 4.2 adapter 与上游回读

保持当前映射：

- random：`useRandom=true`、写入 `musicKit=-1` 或上游要求的随机 sentinel。
- custom + 具体音乐盒：`useRandom=false`、写入真实 `musicKit`。
- custom + 未选择：`useRandom=false`、`musicKit=-1`，但必须用现有 adapter/C# 测试确认该组合不会被误解释为 random。

新增/更新 adapter 测试，证明 random/custom/not-selected 三种组合可以序列化和反序列化，不因 `musicKit=-1` 丢失 `mode`。同时检查真实 `player_loadout.json` 字段与插件日志；没有 JSON、日志或游戏内证据时，只能称为自动化契约通过，不能声称 CS2 已生效。

## 5. P1：具体选择自动切换自定义，支持取消

### 5.1 统一选择入口

在 `ForgeWorkbench.vue` 增加统一的 `chooseCustom()`：安全门关闭时返回 false；否则将 `forge.loadout.mode` 设为 `custom`，再执行选择并 `markDirty()`。以下所有具体选择及属性修改首行调用它：

- 刀具、刀具涂装；
- 手套、手套涂装；
- 角色；
- CT/T 音乐盒；
- 武器皮肤、贴纸、挂件、命名、StatTrak、磨损、种子；
- 武器编辑完成（`updateWeapon` 已切 custom，但要补测试）。

仅切换 CT/T 不改变当前模式。随机模式选择具体对象后必须变为 custom；custom 切回 random 时可以保留具体草稿值，但摘要和写入以 random 为准，再切回 custom 恢复具体值。

### 5.2 取消选择的上游验证

每个 picker 提供明确的“取消选择”按钮或菜单项，不能只放陌生图标。建议候选 sentinel 如下，但执行 AI 必须先阅读 adapter 和 `LoadoutService.cs` 并用测试确认：

| 项目 | 候选内部值 | 候选上游字段 |
| --- | --- | --- |
| 刀具 | `index=-1`、`defindex=42`、`paintKit=-1` | `knifeIndex*=-1`、`knifePaint*=-1` |
| 手套 | `index=-1`、`defindex=0`、`paintKit=-1` | `gloveIndex*=-1`、`gloveDefIndex*=0`、`glovePaint*=-1` |
| 角色 | `agent=null`、`agentPath=''` | `agentModel*=-1`、路径为空 |
| 音乐盒 | `musicKit=null` | `musicKit=-1`，同时保留 `useRandom=false` |
| 武器皮肤 | `paintKit=-1` 或删除 weapon key | 必须以 adapter/上游实现为准，禁止猜测 |

取消后隐藏对应涂装子 picker，右侧显示“未选择刀具/手套/角色/音乐盒”；操作调用 `chooseCustom()`、更新 sentinel、`markDirty()`。disabled 时选择和取消都无效。

### 5.3 选择状态测试

新增 `tests/skin-forge-selection-mode.spec.ts`，覆盖 random -> 具体选择 -> custom、CT/T 切换不改 mode、每类取消后的 sentinel 和摘要、random/custom round-trip、adapter round-trip，以及 disabled 状态不变更。任何 sentinel 与上游冲突时暂停实现并记录证据，不用替代值掩盖问题。

## 6. 验证分层与执行顺序

实际执行 AI 按此顺序工作：

1. 先写 P0 提交失败测试，再实现 ack/watchdog，再跑单测。
2. 实现图片缓存和单项重试，补 loader/component 测试。
3. 修音乐盒摘要与 adapter round-trip，再补选择模式和取消 sentinel 测试。
4. 运行 `npm run test`、`npm run typecheck`（以仓库实际脚本名为准），Rust 使用 `cargo test --manifest-path src-tauri/Cargo.toml`。记录通过/失败/忽略数量。
5. 运行 `cargo fmt --check`、`git diff --check`，检查没有越界修改。
6. 启动 Tauri 开发或构建的桌面程序，人工检查：提交空枪械、失败图片重试、音乐盒三态、取消各类装备、disabled 安全门和 reduced-motion。

自动化通过不等于真实 CS2 通过。真实验收由用户在隔离的 CS2 `-insecure` 环境执行：部署匹配版本 PlayerSkinMod，选择随机/具体/取消，检查 `player_loadout.json`、插件日志和游戏内 CT/T 重生结果。执行 AI 必须把用户未执行的部分标为“待用户验证”。

## 7. 交付格式与停止条件

交付报告必须列出：修改文件、状态机/缓存/字段契约、测试命令与结果、桌面验收结果、未完成的真实 CS2 证据、已知限制和回滚点。不得把“构建成功”写成“游戏内生效”。

遇到以下情况立即停止并报告，不扩大范围：上游 sentinel 无法证明；Rust 写入协议与 UI 状态冲突；安全 gate 被绕过；图片重试导致任意 URL 或无限并发；提交 watchdog 仍可能永久 spinner；测试需要改动 Demo/安装器以外模块。未获得本轮明确授权时，不提交、不推送、不部署、不发布安装程序。
