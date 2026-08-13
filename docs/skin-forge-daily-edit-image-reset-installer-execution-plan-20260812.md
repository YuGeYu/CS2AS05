# 皮肤工坊每日安全确认、编辑提交、图片切换、重置计数与安装器交接方案（2026-08-12）

> 文档角色：本文件由【方案制定 AI】新增，交给不共享上下文的【实际执行 AI】实施。
>
> 工作目录：`E:\CS2AS05`
>
> 本轮目标：修复用户实测的四个皮肤工坊问题，并产出可交给用户模拟操作的全新 Windows NSIS 安装程序。
>
> 本轮不做：不修改 Demo 比分、Rating、BOT、Watcher、Viewer、更新器或无关安装逻辑；不回退现有脏工作树；不复制旧安装器冒充新候选；不把未签名候选写成正式发布；不在没有证据时声称真实 CS2 效果通过。

## 1. 当前事实与调查证据

上一份 `docs/skin-forge-safety-image-sources-fix-execution-plan-20260812.md` 已由实际执行 AI 部分落实，当前工作树已经包含：

- `ForgeSafetyDialog.vue`、`skin-forge-safety-gate.spec.ts`：目前每次挂载都显示安全确认。
- `image-cache.ts`、`ForgeImage.vue`、`skin-forge-image-loader.spec.ts`：有 4 并发 FIFO 和 `[IMAGE_BUSY]` 两次重试。
- `src-tauri/tauri.conf.json`：asset protocol 已启用，scope 为 `$APPLOCALDATA/skin-forge/cache/images/**`。
- `ForgeWorkbench.vue` 与 `WeaponEditorDialog.vue`：武器编辑器通过 `emit('save', structuredClone(draft))` -> 父层 `saveWeapon()` -> `forge.updateWeapon()` 提交。
- `DEFAULT_LOADOUT` 仍在 `src/types/skin-forge.ts` 中给 CT/T 各放入两个 legacy starter weapon：`ak47` 和 `m4a1s`，所以重置后重新载入默认值时计数为 `2 / 35`。
- 当前执行报告明确写着只生成了 `tauri build --no-bundle` 裸 EXE，没有 NSIS 安装器；用户这次要求的是实际安装程序，必须重新 bundle。

### 1.1 安全确认现状

`src/views/SkinForgeView.vue` 的 `safetyConfirmed` 是组件内 `ref(false)`，没有日期键、没有 `localStorage` 读写。因此当前行为是“每次挂载一次”，不是“每天一次”。

### 1.2 “完成编辑”现状与必须调查的事件链

当前按钮本身是：

```vue
<button class="primary-button" type="button" @click="save">
  <Check :size="17" />完成编辑
</button>
```

`save()` 只发出 `save` 事件，父层 `saveWeapon()` 只有在 `props.disabled === false` 时才会更新 store 并关闭模态框。用户说“任何枪械都无效”，不能假定是单一枪械数据问题；执行 AI 必须先加一条失败测试，分别证明：

1. 点击按钮会触发子组件 `save` 事件，并携带与草稿一致的 `defindex`/字段。
2. 父层收到事件后确实调用 `updateWeapon`，`editingWeapon` 变为 `null`，模态框消失。
3. 若当前工坊锁定，按钮必须不可点/不可提交，而不是看似可点但父层静默 `return`。
4. 图片加载、贴纸目录异步加载、按钮层级或遮罩没有遮挡 footer；用真实挂载和 `elementFromPoint`/pointer 事件证据验证，而不是只读源码。

若第 1、2 项都通过但用户仍复现“无效”，优先排查 CSS 层级、WebView 点击命中和 `disabled` 状态；若第 1 项失败，再修模板事件绑定；若第 2 项失败，修父子事件契约。禁止直接删除 `if (props.disabled) return`，因为那会绕过插件安全门槛。

### 1.3 图片快速切换现状

`image-cache.ts` 的 `inFlight` 只保存“当前正在排队/请求”的 Promise，完成后立即删除；它没有成功结果缓存。更重要的是：

- `ForgeImage.vue` 组件卸载时只增加 `requestVersion` 并断开 `IntersectionObserver`，不会从共享 FIFO 中移除未启动任务。
- 快速切换分类或搜索结果时，旧组件的 queued task 仍会启动并占用 4 个槽位；新页面只能等待旧任务。
- Rust 调用没有前端超时/取消；上游慢、队列积压或 WebView 重新导航时，组件可长期显示 spinner。
- 失败重试没有区分“旧页面任务”和“当前页面任务”。

这解释了“快速切换时全都转圈”和“大量皮肤中部分失败”并存：不是单纯 CDN 失败，而是生命周期、队列优先级、成功缓存和超时缺失的组合问题。

### 1.4 重置计数现状

`DEFAULT_LOADOUT.weapons.ct/t` 各自包含：

```ts
{ ak47: legacyDefaultWeapon(7), m4a1s: legacyDefaultWeapon(16) }
```

`ForgeWorkbench.vue` 使用 `Object.keys(teamWeapons).length` 直接显示数量，所以 reset 后显示 `2 / 35` 是当前代码的必然结果，但不符合“重置为默认配置、没有用户自定义装备”的用户认知。这个数字不应继续保留为正常 UX。

## 2. 目标行为

### 2.1 安全确认：同一自然日只出现一次

- 每个自然日第一次进入皮肤工坊显示风险确认；当天再次进入不显示。
- 日期按本机本地日历计算，格式固定 `YYYY-MM-DD`，不要用 UTC 日期造成北京时间凌晨错位。
- 建议持久键：`cs2as:skin-forge:safety-confirmed-date:v1`，值为 `2026-08-12` 这类日期字符串。
- 只有玩家点击“我已了解风险，进入工坊”后才写入今天日期；取消、Escape、遮罩关闭不写入。
- 日期变化、日期值格式非法、storage 读取异常均视为今天未确认（fail closed），但 storage 不可用时不能阻塞本次确认成功后的工坊使用。
- 安全确认不能与快速上手共用 key。快速上手仍按原“一次性”语义；本轮只新增安全确认的“每日”语义。
- 测试环境提供纯函数或可注入 `today`，不要在测试里依赖真实系统日期。

建议新增 `src/features/skin-forge/safety-confirmation.ts`：

```ts
export const SAFETY_CONFIRMATION_KEY = 'cs2as:skin-forge:safety-confirmed-date:v1'
export function localDateKey(date = new Date()): string { /* local YYYY-MM-DD */ }
export function hasConfirmedSafetyToday(storage = window.localStorage, date = new Date()): boolean
export function confirmSafetyToday(storage = window.localStorage, date = new Date()): void
```

`SkinForgeView.vue` 在挂载时用 `hasConfirmedSafetyToday()` 初始化；确认事件先调用 `confirmSafetyToday()`，再设置 `safetyConfirmed = true`、调用 `forge.load()`。不能在组件卸载时清除当天确认。

### 2.2 “完成编辑”：必须真实提交并关闭

正常路径：

```text
武器卡片点击
 -> editingWeapon = item
 -> selectedWeapon = 已配置 weapon 或 createWeapon(defindex)
 -> WeaponEditorDialog draft = structuredClone(props.weapon)
 -> 完成编辑
 -> emit save(draft)
 -> ForgeWorkbench.saveWeapon
 -> forge.updateWeapon(activeTeam, weapon)
 -> dirty = true
 -> editingWeapon = null
 -> 模态框关闭，卡片数量/预览/选中涂装刷新
```

实施要求：

- 为 `WeaponEditorDialog.vue` 增加明确的 `data-testid="weapon-editor-save"`，按钮保留中文“完成编辑”；测试通过语义或 test id 点击，不依赖坐标。
- `save()` 必须只在当前编辑器仍挂载、草稿 `defindex` 与传入武器一致时发出一次事件；避免重复点击产生双提交。可用 `saving`/`submitted` 状态，提交后 150-250ms 内按钮进入 busy/disabled，但父层关闭应立即发生，不要让用户等待图片或贴纸目录。
- 父层 `saveWeapon()` 保留 disabled guard，但锁定态应让按钮不可达：`ForgeWorkbench` 给 `WeaponEditorDialog` 传 `disabled`，编辑器所有 tab、选择器、输入和 footer 按钮遵循原生 disabled/`aria-disabled`。
- 如果模态框在插件状态变化后变成锁定，保存按钮应显示“完成部署后可编辑”或直接 disabled，不能点击后无反馈。
- `selectedWeapon` 不得因异步图片/目录加载变为 null；编辑器草稿应是稳定快照。
- 保存后必须检查 store 的对象引用/键名：最终 key 统一为 `weapon_${defindex}`。不能继续生成 `ak47`/`m4a1s` 这种 legacy key 写回新配置。
- 点击“完成编辑”后，至少更新：`Object.keys(teamWeapons).length`、卡片 `aria-selected`、卡片图片选择、当前预览和 dirty 状态。
- 如果用户没有改任何字段，也应关闭模态框并完成一次草稿提交，不应表现为按钮失效。

建议新增 `tests/skin-forge-weapon-editor-save.spec.ts`：

- 对 AK、M4A1-S、任意一个非默认枪械各挂载真实 `WeaponEditorDialog`，点击 `data-testid`，断言 `save` 恰好一次、payload `defindex` 正确。
- 挂载 `ForgeWorkbench` + mock store，点击卡片再点完成编辑，断言模态框关闭、store 更新和 dirty=true。
- 在 `disabled=true` 时断言保存按钮 disabled/不可提交，且 store 不改变；同时有可读的锁定原因。
- 先让 `loadStickerCatalog()` pending，再点击“完成编辑”，断言仍能提交并关闭，不被贴纸目录加载阻塞。
- 连续双击完成编辑，断言只提交一次。
- 增加一次真实 Tauri/桌面手工证据：打开三种枪械，分别完成编辑，记录模态框关闭与卡片计数变化。

### 2.3 图片：可取消、可复用、可超时、有优先级

保留 Rust 的 HTTPS、域名、MIME、大小、asset scope 边界；重写前端 loader 的生命周期。

#### 设计契约

- 成功结果缓存：`resolvedByUrl: Map<string, string>`，同 URL 后续直接返回 `convertFileSrc` 结果，不重新 invoke。
- 进行中去重：`inFlight: Map<string, Promise<string>>`。
- 队列任务带 `AbortSignal`/取消状态和 `priority`；当前可见/编辑器图片优先于旧页面的预取。
- 每个 `ForgeImage` 创建 `AbortController`，卸载、src 改变或分类切换时 abort 自己的等待；共享请求只有在没有订阅者时才取消底层任务。
- 旧任务不能阻塞新页面。至少实现：任务启动前检查 subscriber count/aborted；队列选择跳过已取消任务；新任务按“eager/当前视图 > lazy/预取”优先。
- 每次 `skinForgeCacheImage` 设置 15 秒硬超时；超时后只对当前任务显示失败/允许用户重新进入页面触发重试，不能永远 spinner。
- `[IMAGE_BUSY]` 最多重试 2 次，其他错误不重试；超时不无限重试。
- 成功缓存不能因为一个组件卸载而删除；缓存只在内存会话内保留，磁盘缓存仍由 Rust 管理。
- `resetForgeImageQueueForTests()` 必须只用于测试，运行时不能靠清空 Map 解决页面切换。

#### ForgeImage 视觉状态

- loading：显示稳定尺寸的 spinner，占位区域不能抖动。
- success：显示图片。
- failed：显示图片不可用图标和简短可读状态，不持续旋转。
- 切换 src 时先保留旧图或显示轻量过渡，不能让整个列表全部回到 spinner；建议对已有成功图先保留，只有新图自身进入 loading。
- `aria-busy` 只加在当前图片 host，不要把整个目录标成 busy。
- 1440x900、1100x700 下卡片尺寸稳定；reduced motion 禁止无限旋转，改为静态进度点/“正在载入”文字。

#### 图片测试

扩展 `tests/skin-forge-image-loader.spec.ts`：

- 同 URL 先成功，卸载后再次请求不 invoke 第二次。
- 旧页面创建 20 个 lazy 任务后全部取消，立即创建新页面 8 个 eager 任务；新任务必须在旧任务前启动。
- 底层 Promise 永不 resolve 时 15 秒超时并 reject，spinner 进入 failed。
- 旧 src Promise 延迟返回时不能覆盖新 src。
- `[IMAGE_BUSY]` 仍最多 3 次 invoke（首次 + 两次重试）。
- 切换枪械皮肤筛选时，成功缓存图片不重新闪烁。
- 统计最大活动 invoke 不超过 4，且所有仍被订阅的图片最终 settle（success 或 failed）。

### 2.4 重置后计数改为 0 / 35

产品语义定为：重置 = 清除 slot 0 的玩家自定义装备，恢复随机模式；因此“已配置”应为 `0 / 35`，而不是 legacy starter 的 `2 / 35`。

实施要求：

- 将 `DEFAULT_LOADOUT.weapons.ct` 和 `.t` 改为空对象，不再把 AK47/M4A1S 当作已配置条目。
- `createWeapon()` 仍负责打开任意未配置枪械的编辑器草稿，不要把默认条目塞回 loadout。
- `migrateLoadout()` 对缺失 `weapons` 或空 slot 返回空对象；旧配置文件中真实存在的 AK/M4A1S 仍应被读取并显示为已配置，不能抹掉用户已有配置。
- `fromPlayerSkinModFile(null)`、Rust reset 后重新 load、首次进入无配置三种情况都应得到 0 条 weapon keys。
- `reset()` 成功后不只把内存替换为 default，还应重新从目标文件 readback 或调用 `load()`，确认磁盘与 UI 一致；清除 `lastSave`，避免底部仍显示旧保存路径/哈希。
- reset 成功后 `dirty=false`、mode=`random`、CT/T 的武器计数都是 0；刀具、手套、角色和音乐盒回到默认随机语义。
- 如果产品必须保留上游旧字段兼容，保留在 `passthrough` 或 adapter 中，不要把它们计入新的 UI `已配置` 数量。

建议新增/更新测试：

- `tests/skin-forge-loadout-adapter.spec.ts`：空 slot -> CT/T 均 0 keys；旧文件含 AK -> 仍 1 key。
- `tests/skin-forge-reset.spec.ts`：mock `skinForgeReset` 后断言 `loadout.weapons.ct/t` 为空、mode random、dirty false、lastSave null。
- `tests/skin-forge-count.spec.ts`：默认、reset、添加一把、添加 CT/T 两把分别断言 `0/35`、`1/35`、`2/35`。
- 现有断言若把 legacy starter 当默认产品状态，改为明确测试“旧数据兼容”而非“新默认状态”。

## 3. 安装程序产出方案

用户明确要求安装程序用于模拟玩家操作；实际执行 AI 必须在代码、自动化和桌面候选通过后执行完整 NSIS bundle。当前版本已经是 `0.5.7`，本轮不再无意义升版本号。

### 3.1 构建前闸门

在 `E:\CS2AS05` 执行并保存输出到本轮证据目录：

```powershell
Set-Location -LiteralPath 'E:\CS2AS05'
npm run workspace:check
npm run typecheck
npm run lint
npm test -- --run
npm run build:web
Set-Location -LiteralPath 'E:\CS2AS05\src-tauri'
cargo fmt --check
cargo test --lib
cargo clippy --all-targets -- -D warnings
Set-Location -LiteralPath 'E:\CS2AS05'
git diff --check
```

若 clippy 只报告 vendored `third_party/demoparser` 既有 warning，要把完整 warning 记录下来并确认本项目 crate 没有新增 warning；不能静默忽略。

### 3.2 生成 NSIS

只在上述闸门通过后执行：

```powershell
Set-Location -LiteralPath 'E:\CS2AS05'
npm run bundle:desktop
```

原始产物必须是：

```text
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.7_x64-setup.exe
```

不要把 `src-tauri\target\release\CS2BotImproverAssistant.exe` 裸 EXE 交给用户代替安装程序。不要模糊选择旧 `0.5.6` 或历史同名文件。

### 3.3 安装器静态与隔离安装验收

执行 AI 必须创建独立证据目录，例如：

```text
E:\CS2AS05\workspace\player-acceptance\skin-forge-0.5.7-20260812\
  CS2人机增强助手_0.5.7_x64-setup.exe
  CS2人机增强助手_0.5.7_x64-setup.exe.sha256
  TESTING.md
  evidence\
```

记录：绝对路径、字节数、SHA-256、生成时间、文件版本、`Get-AuthenticodeSignature` 状态、是否有同名 updater `.sig`。没有签名时明确标记 `Authenticode NotSigned / local test candidate`；不能发布到生产更新源。

在隔离安装目录执行一次静默或交互安装，确认：

- 安装目录存在主 EXE、卸载器和 `resources/skin-forge` 固定资源。
- 安装版启动进程的绝对路径来自安装目录，不是源码或 `target/release`。
- 安装版能打开皮肤工坊、读取图片缓存、调用部署检查；不出现 `[PLUGIN_RESOURCE_MISSING]`。
- 关闭主程序后 WebView2 子进程退出；卸载后安装目录和卸载登记符合预期。
- 不删除用户 CS2、Demo、CounterStrikeSharp 或已部署 PlayerSkinMod，除非产品有独立明确的卸载动作。

### 3.4 交给用户的最短模拟步骤

交付时只给用户：

1. 安装并启动指定绝对路径的 `0.5.7` NSIS。
2. 进入皮肤工坊：当天第一次应显示安全确认；当天第二次不显示，跨自然日再显示。
3. 在已部署插件的前提下，打开 AK47、M4A1-S 和一把非默认枪械，分别修改皮肤或不修改直接点击“完成编辑”；每次都应关闭模态框并刷新卡片。
4. 快速切换武器分类、搜索词和皮肤分页；观察图片最终显示或明确失败，不应整页永久转圈。
5. 点击重置，确认 CT/T 武器均显示 `0 / 35 已配置`，底部没有旧保存哈希。
6. 只在本地/离线 `-insecure` 环境应用和验证；不要连接 VAC 保护服务器。

用户不应执行 PowerShell、不应复制 DLL、不应手工编辑 JSON。执行 AI 负责后台记录截图、进程路径、安装目录和日志；真实 CS2 画面由用户执行并反馈。

## 4. 必须新增或更新的文件清单

### P0

- `src/features/skin-forge/safety-confirmation.ts`：每日确认纯函数和 storage 容错。
- `src/views/SkinForgeView.vue`：按日期显示安全确认，确认后写今天日期。
- `src/features/skin-forge/components/WeaponEditorDialog.vue`：完成编辑 test id、单次提交、disabled 语义。
- `src/features/skin-forge/components/ForgeWorkbench.vue`：修复父子事件链、传 disabled、更新稳定快照。
- `tests/skin-forge-weapon-editor-save.spec.ts`：新增编辑提交回归测试。
- `src/features/skin-forge/image-cache.ts`：成功缓存、订阅者取消、任务优先级、超时。
- `src/features/skin-forge/components/ForgeImage.vue`：AbortController、切 src 保留成功图、超时失败态。
- `tests/skin-forge-image-loader.spec.ts`：补快速切换、取消、超时、缓存和优先级测试。
- `src/types/skin-forge.ts`：默认 loadout 改为空 weapon maps。
- `src/stores/skinForge.ts`：reset 后 readback、清理 `lastSave`、计数状态一致。
- `tests/skin-forge-reset.spec.ts`、`tests/skin-forge-count.spec.ts`：新增 reset/计数回归。

### P1

- `src/styles/main.css`：编辑提交 busy/disabled、图片 loading 稳定尺寸和 reduced-motion 静态状态。
- `docs/skin-forge-safety-image-sources-fix-execution-report-20260812.md`：执行完成后追加本轮实际证据，不提前伪造结果。
- `workspace/player-acceptance/skin-forge-0.5.7-20260812/TESTING.md`：安装器用户说明和风险边界。

## 5. 完成判定、停止条件与交付报告

实际执行 AI 最终必须分开报告：

1. 每日安全确认：首次/同日再次/跨日三种证据。
2. 编辑提交：至少三把枪的模态框关闭、store 更新、计数/预览刷新证据。
3. 图片：快速切换、缓存命中、超时失败、无无限 spinner 的证据。
4. 重置：CT/T 均 `0 / 35`、mode random、dirty/lastSave 状态证据。
5. 自动化：实际命令、通过数量、失败数量；未跑必须直说。
6. NSIS：绝对路径、大小、SHA-256、生成时间、Authenticode、`.sig` 状态。
7. 隔离安装：安装版进程路径、资源回读、启动/关闭/卸载结果。
8. 未验证项：真实 CS2 `-insecure` 游戏内结果必须单列。

完成标准：每日确认语义正确；任意枪械编辑可提交并关闭；快速切换图片最终有成功或失败终态且不被旧任务拖死；重置显示 0/35；全量自动化与隔离安装通过；用户拿到的是本轮全新 NSIS 安装器，而不是裸 EXE 或旧候选。
