# 皮肤工坊安全门槛、图片加载与来源归并修复实施交接（2026-08-12）

> 文档角色：本文件由【方案制定 AI】新增，交给完全不共享对话上下文的【实际执行 AI】直接实施。
>
> 工作目录：`E:\CS2AS05`
>
> 本轮边界：只修本文列出的三组问题；不升级版本号，不新开分支，不回退已有改动，不提交、不推送、不部署、不发布安装包。完成代码和自动化验证后，真实 Windows 安装版与 CS2 内效果由用户执行。

## 1. 最终目标与不可协商的产品语义

### 1.1 进入工坊必须完成安全确认

- 玩家每次从其他页面进入“皮肤工坊”，先显示阻塞式安全确认模态框；没有确认前不能浏览或修改装备，也不能部署或写入配置。
- “每次进入”以 `SkinForgeView` 的一次挂载周期为准，不使用永久跳过标记。玩家离开工坊再回来，必须重新确认。
- 文案必须直接说明：饰品修改仅用于本地/离线 `-insecure` 环境；不得连接 VAC 保护服务器；第三方插件可能带来账号、兼容性和游戏更新后的失效风险；玩家应自行确认并承担使用风险。
- 主按钮建议命名“我已了解风险，进入工坊”，次按钮“返回概览”。确认按钮不能伪装成普通“知道了”。
- 关闭、按 Escape、点击遮罩均视为未确认，返回“概览”，不能落入未解锁的工坊页面。

### 1.2 插件未就绪时，永远提醒并禁止装备选择/配置写入

- 唯一就绪条件是当前 CS2 根目录的 `skin_forge_check_plugin` 返回 `allPresent === true` 且 `hashMismatches.length === 0`。不要只看 `deployedVersion`，不要只看目录存在。
- 未就绪、检查失败、目录切换后尚未复检，全部按“未部署”处理（fail closed）。
- 安全确认完成后，如果插件未就绪：
  - “部署 / 更新插件”按钮持续高亮，不因看过、点击失败、切换分类或重进页面而消失。
  - 按钮旁显示锚定提示气泡：“先部署 PlayerSkinMod，工坊才会开放装备选择并写入配置。”
  - 气泡没有永久关闭入口；部署成功并复检通过后自动消失。
  - 所有工坊编辑操作必须不可用：模式切换、阵营切换、装备选择、武器编辑、刀/手套/角色/音乐盒选择、磨损/模板、贴纸/挂件/命名/StatTrak、重置和“应用装备”。为减少误解，建议连分类内容区一起加禁用遮罩，但保留目录和部署状态可读。
- “应用装备”必须至少三层防护：按钮禁用、Pinia store 的 `save()` 前置检查、Rust `skin_forge_save_loadout` 再次校验插件。不能只做视觉禁用。
- `skin_forge_reset_loadout` 同样会写配置，必须执行相同 Rust 就绪门槛；不要漏掉重置路径。
- 部署成功后必须重新调用 `skin_forge_check_plugin`；只有复检为完整且哈希无差异，才解除高亮/气泡和装备锁。
- 切换 CS2 根目录时清空旧 `plugin` 状态并重新加载，禁止沿用旧目录的已部署状态。

### 1.3 “快速上手”只高亮一次，不显示气泡

- 快速上手按钮仍位于“应用装备”右侧，使用 `BookOpen` 图标和现有 tooltip/accessible name。
- 只做一次视觉高亮，不显示气泡，不自动弹出教程。
- 持久键固定为 `cs2as:skin-forge:quick-start-highlighted:v1`。值可写为字符串 `1`。
- 展示条件：本机从未记录该键，安全确认已经完成，按钮已真实渲染。建议至少保持 3 秒或直到玩家离开页面/点击按钮；随后立即写入持久键。
- 玩家点击按钮时立即写入持久键并移除高亮；之后进入工坊永远不再提醒。
- `localStorage` 不可用或写入失败时不能阻塞工坊；本次展示后在内存中标记已消费，避免当前会话反复闪烁。
- `prefers-reduced-motion: reduce` 下禁止脉冲动画，只保留稳定的描边、背景或焦点环。

### 1.4 取消工坊“关于与来源”，统一归入安装与诊断

- 删除皮肤工坊页中“快速上手”右侧的“关于与来源”图标按钮。
- `ForgeHelpDialog` 的页面类型从 `tutorial | safety | about` 收窄为 `tutorial | safety`，删除皮肤来源分支和无用的 `Info`/`ExternalLink` 导入。
- 不删除安装与诊断页现有的全局 `AboutSourcesModal` 入口；来源信息统一在那里展示。
- “参考项目与许可”页按用途分组展示：
  - Demo 参考：`https://github.com/unicbm/demotracer`
  - Demo 参考：`https://github.com/LaihoE/demoparser`
  - Demo 参考：`https://github.com/akiver/cs-demo-manager`
  - 皮肤参考：`https://github.com/kaecho/CS2-Skin-Forge`
- 保留既有 `ed0ard/CS2-Bot-Improver` 来源，不能因新增四项而删掉既有归属与许可说明。
- 每个条目显示用户向中文说明、仓库名和“打开项目”动作。不要捏造未核实的许可证；许可与固定提交以根 `NOTICE.md` 及 `third_party/*` 的 provenance 为准。
- 外链必须走 Rust 白名单命令，不能使用 WebView 默认 `<a target="_blank">`，也不能接收任意 URL。

## 2. 已调查现状与根因

### 2.1 当前实现位置

| 范围 | 当前文件 | 现状 |
| --- | --- | --- |
| 页面与顶栏动作 | `src/views/SkinForgeView.vue` | “应用装备”只检查 busy、CS2 进程和目录；快速上手与关于按钮都在此；安全说明只是可选链接 |
| 装备交互 | `src/features/skin-forge/components/ForgeWorkbench.vue` | 所有选择器直接修改 store loadout，没有插件门槛 |
| 帮助模态框 | `src/features/skin-forge/components/ForgeHelpDialog.vue` | 同时承载 tutorial、safety、about；安全页不是进入门槛 |
| 图片显示 | `src/features/skin-forge/components/ForgeImage.vue` | Tauri 中先调用 Rust 缓存，再用 `convertFileSrc(path)` 给 `<img>` |
| 图片下载缓存 | `src-tauri/src/commands/skin_forge.rs` | HTTPS/域名/MIME/2 MiB/256 MiB/重定向边界已有；最多 6 个活动请求，超限立即 `[IMAGE_BUSY]` |
| Tauri 安全配置 | `src-tauri/tauri.conf.json` | `app.security` 只有 `csp: null`，没有启用 `assetProtocol`，也没有 scope |
| 配置写入 | `src/stores/skinForge.ts`、`skin_forge_save_loadout`、`skin_forge_reset_loadout` | 前后端都没有验证 PlayerSkinMod 已完整部署 |
| 安装页关于入口 | `src/views/InstallView.vue` + `SupportActions.vue` + `AboutSourcesModal.vue` | 现有模态框只有一个硬编码上游项目动作 |
| 外链命令 | `src/services/tauri/support.ts`、`src-tauri/src/commands/support.rs`、`src-tauri/src/services/support.rs` | 只有 `open_upstream_project`，固定打开 Bot Improver |

### 2.2 图片全灭的 P0 根因

`@tauri-apps/api/core.d.ts` 对 `convertFileSrc` 的本地文件显示要求很明确：必须在 `app.security.assetProtocol` 中设置 `enable: true`，并配置允许访问的 scope。当前 `src-tauri/tauri.conf.json` 两项均缺失，所以 Rust 即使把图片下载到 `%LOCALAPPDATA%\com.aipc.cs2botimprover\skin-forge\cache\images`，WebView 仍没有权限读取，最终触发 `<img @error>`，表现为所有图片占位符。

2026-08-12 已用项目目录中的真实 `community.akamai.steamstatic.com` 图片 URL 做 GET：HTTP 200、`image/png`、84671 bytes，说明至少该样本上游可用，不能把全灭归因于 Steam CDN。

P0 修复是在 `src-tauri/tauri.conf.json` 增加最窄 scope：

```json
"security": {
  "csp": null,
  "assetProtocol": {
    "enable": true,
    "scope": ["$APPLOCALDATA/skin-forge/cache/images/**"]
  }
}
```

不要放宽为 `$APPLOCALDATA/**`、`$LOCALDATA/**` 或全盘路径。实际执行 AI 必须用当前 Tauri 2 schema 校验配置，并在打包后的主窗口实测 `asset://`/`http://asset.localhost` 响应成功。

### 2.3 图片加载的 P1 并发缺陷

首屏会同时挂载大量 `ForgeImage`。每个组件独立调用 `skin_forge_cache_image`，Rust 的全局计数超过 6 个后立即返回 `[IMAGE_BUSY]`，而 `ForgeImage.vue` catch 后直接永久 fallback，没有排队或重试。这会导致 P0 修完后仍可能只有前几张成功。

执行时采用前端共享加载器，不要移除 Rust 安全上限：

- 新增 `src/features/skin-forge/image-cache.ts`，模块级维护 `Map<string, Promise<string>>` 去重和 FIFO 队列。
- 前端同时 invoke 数不超过 Rust 上限，建议 4；相同 URL 共享同一 Promise。
- `[IMAGE_BUSY]` 只允许有界退避重试 2 次（例如 120 ms、300 ms），组件卸载或 src 改变后旧结果不得覆盖新结果。
- 其他网络、HTTP、MIME、大小、域名错误不盲重试，显示 fallback。
- `ForgeImage.vue` 增加 `loading`/`failed` 的可观测状态，失败 fallback 保留无障碍名称；禁止无限 spinner。
- 延迟加载应跟随可见卡片；若当前 `<img loading="lazy">` 无法阻止 invoke 提前发生，可用轻量 `IntersectionObserver`，不要一次预取几千张贴纸。
- 缓存命中仍走同一去重入口。测试后确认离开对话框不会遗留无限队列。

## 3. 推荐状态机与组件契约

```text
进入 SkinForgeView
  -> safetyPending（阻塞模态框，工坊不可操作）
  -> 玩家取消：导航 overview，结束
  -> 玩家确认：checkingPlugin
      -> ready：workbenchEnabled，隐藏部署提醒
      -> missing / mismatch / checkError：deploymentRequired
          -> 部署按钮持续高亮 + 锚定气泡
          -> workbench/apply/reset 全部禁用
          -> deploy 成功后重新 check
              -> ready：解除门槛
              -> 仍异常：保持 deploymentRequired 并展示可操作错误
```

建议在 `SkinForgeView.vue` 定义：

```ts
const safetyConfirmed = ref(false)
const pluginReady = computed(() =>
  forge.plugin?.allPresent === true && forge.plugin.hashMismatches.length === 0
)
const workshopUnlocked = computed(() => safetyConfirmed.value && pluginReady.value)
```

组件契约：

- `ForgeWorkbench` 新增必填 prop `disabled: boolean`，在事件处理函数入口也执行 guard，不能只靠 CSS `pointer-events`。
- `CatalogGrid`、`WeaponEditorDialog` 等现有组件若没有 disabled 契约，逐层加语义化 `disabled`/`aria-disabled`，不要把整个页面变成不可聚焦的透明层。
- 部署按钮容器作为气泡 anchor，气泡使用 `role="status"` 或静态说明；不要用 `role="alert"` 循环打断屏幕阅读器。
- 插件状态变化使用 `aria-live="polite"`；部署失败使用现有 error/toast。
- 安全模态框必须 Teleport 到 body、锁定初始焦点、Esc/关闭行为一致，并把焦点恢复或导航到概览。

## 4. UI 与动效规格（UI/UX Pro Max + Emil Design Engineering）

| Before | After | Why |
| --- | --- | --- |
| 安全说明是可忽略的小链接 | 每次进入先显示聚焦明确的风险确认模态框 | 风险确认是流程门槛，不是帮助内容 |
| 未部署仅显示普通次级按钮 | 使用 warning 语义描边、柔和外环和锚定气泡持续提示 | 让唯一下一步可扫描，不改变布局尺寸 |
| 未部署仍可搭配并点击应用 | 工作台保持可读但交互禁用，明确显示“完成部署后开放” | 防止玩家投入配置后才发现无法生效 |
| 快速上手没有首次引导 | 首次使用仅给按钮 3 秒高亮，不弹气泡 | 低打扰地建立入口认知 |
| 工坊和安装页各有来源入口 | 工坊删除来源按钮，安装页集中列出完整来源 | 降低重复入口和来源信息漂移 |
| 图片请求无队列，失败即终止 | URL 去重、有界 FIFO、可见时加载、有限重试 | 避免并发上限造成批量假失败 |

样式约束：

- 沿用现有 jade、bronze、warning、surface 语义 token，不新增紫蓝渐变、装饰光球或大面积单色。
- 部署高亮只动画 `opacity`/`box-shadow` 或伪元素的 `transform`，周期 1.6-2.0 秒，不移动按钮、不改变边框宽度导致布局抖动。
- 气泡 160-200 ms 强 `ease-out` 进入，起点 `opacity: 0; transform: translateY(-4px) scale(.97)`，transform origin 指向部署按钮；退出更快。
- 快速上手高亮不应比部署门槛更抢眼。若两个条件同时存在，部署提示优先，快速上手高亮延后到插件 ready 后再消费，避免双焦点。
- `:active` 可沿现有按钮反馈使用 `scale(.97)`，不得 `transition: all`。
- 1100x700 最小窗口不能遮住状态区和按钮；窄布局气泡改为按钮下方全宽说明，不溢出视口。
- 所有按钮保持至少 44px 命中区；图标使用现有 `lucide-vue-next`。
- 文案优先用户语言，不暴露 schema、invoke、asset protocol 等实现术语。

## 5. 逐文件实施清单

### P0：门槛与图片可见

1. `src/views/SkinForgeView.vue`
   - 新增安全确认状态和模态框/独立组件接线。
   - 删除 about 按钮与 `help = 'about'`。
   - 建立 `pluginReady`、`workshopUnlocked`；部署按钮高亮/气泡；应用与重置门槛。
   - 将 disabled 传给 `ForgeWorkbench`。
   - 加入快速上手一次性高亮读取/消费逻辑。
2. `src/features/skin-forge/components/ForgeSafetyDialog.vue`（推荐新增）
   - 承载每次进入风险确认，不复用普通帮助页的“知道了”。
   - emit `confirm` 与 `cancel`；完整焦点与 Escape 行为。
3. `src/features/skin-forge/components/ForgeWorkbench.vue` 及其子组件
   - 补 disabled 契约和每个 mutation handler 的 guard。
4. `src/stores/skinForge.ts`
   - `save()`、`reset()` 在调用 Tauri 前要求 fresh `pluginReady`；检查失败视为未就绪。
   - 根目录变化时清空 plugin 状态；不要让旧根结果竞态覆盖新根。
5. `src-tauri/src/commands/skin_forge.rs`
   - 提取 `require_plugin_ready(root_path)`，内部复用 `check_plugin`，校验 `all_present` 和空 `hash_mismatches`。
   - `skin_forge_save_loadout` 与 `skin_forge_reset_loadout` 在任何文件写入前调用。
   - 错误码建议 `[PLAYER_SKIN_MOD_REQUIRED]`，中文说明指向“部署 / 更新插件”。
   - 注意当前 `save` 先写 app draft 再写游戏配置；门槛必须在两个写入动作之前，确保未部署时“程序才不会写入配置”，连 draft 也不写。
6. `src-tauri/tauri.conf.json`
   - 启用 asset protocol，scope 仅为 `$APPLOCALDATA/skin-forge/cache/images/**`。
7. `src/features/skin-forge/image-cache.ts` + `ForgeImage.vue`
   - 实现 URL 去重、有界队列、有限 busy 重试、竞态取消/忽略。

### P1：来源归并

8. `src/features/skin-forge/components/ForgeHelpDialog.vue`
   - 删除 about 类型、分支和无用 import。
9. `src/components/AboutSourcesModal.vue`
   - 保留两层“关于 -> 参考项目与许可”结构。
   - 将单一 source-list 改成“核心上游 / Demo 参考 / 皮肤参考”三个清晰分组或等价层级。
   - 五个仓库分别有固定 ID、中文说明与打开动作；单项失败只在该项附近或统一错误区说明。
10. `src/services/tauri/support.ts`
    - 新增 `openReferenceProject(project: ReferenceProjectId)`，不要接受 URL。
11. `src-tauri/src/commands/support.rs`、`src-tauri/src/services/support.rs`
    - 定义 Rust enum 或严格 match：`bot-improver`、`demotracer`、`demoparser`、`cs-demo-manager`、`skin-forge`。
    - ID 映射到本文五个固定 HTTPS GitHub URL；未知 ID 返回错误。
    - 可保留旧命令兼容其他调用，但 `AboutSourcesModal` 迁移到新命令。
12. `NOTICE.md`
    - 当前已有 demoparser、cs-demo-manager、CS2-Skin-Forge 记录；先核对再改，禁止重复段落。
    - demotracer 若只是“参考”而未复制代码/资产，不要擅自加入版权或许可证声明；可在 UI 标为参考项目。只有实际复用后才按事实更新 NOTICE/provenance。

### P2：样式和测试

13. `src/styles/main.css`
    - 新增风险模态框、部署 anchor/气泡、disabled 工作台、一次性 quick-start 高亮。
    - 添加 reduced-motion 静态回退；复用现有断点和 token。
14. 测试文件见下一节。不要修改与本问题无关的 Demo、Rating、BOT 或发布逻辑。

## 6. 必须新增或更新的自动化测试

### 6.1 前端组件/状态测试

新增 `tests/skin-forge-safety-gate.spec.ts`：

- 首次挂载先出现安全模态框；未确认时工作台、部署和应用不可操作。
- 确认 + plugin missing：部署按钮有 persistent cue，气泡存在，模式/阵营/应用/选择/重置全部禁用。
- 部署失败后提示仍存在；部署成功但复检不完整仍不解锁；复检 ready 才解锁。
- 取消、Escape、遮罩关闭均导航到 overview。
- 离开再进入重新要求安全确认。
- 切换 CS2 根目录后旧 ready 状态不能沿用。

新增 `tests/skin-forge-quick-start-cue.spec.ts`：

- storage 无记录且安全确认 + plugin ready 后显示一次高亮，无气泡。
- 3 秒结束或点击后写入 `cs2as:skin-forge:quick-start-highlighted:v1`。
- 重挂载不再显示；localStorage 抛错不阻塞页面且本会话不反复出现。
- plugin missing 时部署 cue 优先，quick-start 尚不消费；ready 后再出现。
- reduced-motion 类/媒体条件下没有脉冲动画（可用现有设计契约静态断言）。

新增 `tests/skin-forge-image-loader.spec.ts`：

- 20 个不同 URL 同时请求时，最大并发不超过 4，最终都会启动而非 busy 永久失败。
- 相同 URL 多组件只 invoke 一次。
- `[IMAGE_BUSY]` 最多重试 2 次；成功后所有订阅者拿到同一地址；永久失败停止重试并 fallback。
- src 快速变更时旧 Promise 结果不覆盖新图。
- `convertFileSrc` 只接收 Rust 返回的缓存路径。

更新 `tests/about-sources-modal.spec.ts`：

- 仍需第二层才显示来源列表，重开回到第一页。
- 断言五个项目与四个新增 URL 对应的项目 ID 全部出现。
- 每个按钮调用 `openReferenceProject` 的固定 ID，不传任意 URL。
- 工坊页测试断言“关于与来源”按钮不存在、快速上手仍存在。

### 6.2 Rust 单元测试

- 完整插件目录允许 save/reset；缺文件、哈希错误、目录不存在全部在写入前返回 `[PLAYER_SKIN_MOD_REQUIRED]`。
- 失败后目标 `player_loadout.json` 与 app draft 都不存在或保持原字节不变。
- reference project ID 白名单五项逐一映射正确；未知 ID 被拒绝。
- 保留现有图片 URL、MIME、缓存大小、路径边界测试。

### 6.3 配置契约测试

新增或更新 installer/contract 测试，解析 `src-tauri/tauri.conf.json` 并断言：

- `assetProtocol.enable === true`。
- scope 精确包含 `$APPLOCALDATA/skin-forge/cache/images/**`。
- scope 不包含 `$HOME/**`、`$LOCALDATA/**`、`**` 等宽泛模式。

## 7. 验证顺序与证据门槛

执行 AI 必须快速但完整地按以下顺序验证；任何失败先修本次引入问题，不得用跳过测试收尾。

```powershell
Set-Location -LiteralPath 'E:\CS2AS05'
npm run typecheck
npm run lint
npm test -- --run tests/skin-forge-safety-gate.spec.ts tests/skin-forge-quick-start-cue.spec.ts tests/skin-forge-image-loader.spec.ts tests/about-sources-modal.spec.ts tests/skin-forge-v1.8.2-catalog.spec.ts tests/skin-forge-loadout-adapter.spec.ts
npm run build:web
Set-Location -LiteralPath 'E:\CS2AS05\src-tauri'
cargo fmt --check
cargo test --lib commands::skin_forge
cargo test --lib services::support
Set-Location -LiteralPath 'E:\CS2AS05'
git diff --check
```

若定向测试名与 Rust 模块过滤不匹配，先用 `cargo test --lib -- --list` 找真实名称；不要宣称未实际执行的结果。改动触及共享 support command 或配置后，最终至少再运行一次 `npm test -- --run` 全量前端测试和 `cargo test --lib`。

### 7.1 实际执行 AI 可完成的桌面验收

- 用 `npm run dev:desktop` 或 `npm run build:desktop` 启动真实 Tauri 主窗口，不能只验浏览器 Vite 页面，因为 asset protocol 只在 Tauri 生效。
- 用一个隔离的假 CS2 目录/现有 Rust fixture 验证未部署门槛，不得操作用户真实 CS2 配置。
- 清空本应用图片缓存前先解析并确认绝对路径严格位于 app local data 的 `skin-forge\cache\images`；不做宽目录递归删除。更安全的做法是用全新 app data 测试实例或只记录新下载文件。
- 在武器、刀具、手套、角色、音乐盒各抽查至少 3 张；打开武器编辑器再抽查皮肤、贴纸、挂件。要求图片可见，不是只看到成功 HTTP 或缓存文件。
- 查看 Tauri/WebView 日志：无 asset scope 拒绝、无持续 `[IMAGE_BUSY]` 风暴、无无限重试。
- 首次载入与缓存后二次载入各记录一次；二次应从缓存显示。
- 以 1440x900 和项目最小窗口 1100x700 检查：气泡不溢出、按钮不换行压坏标题、模态框内容可滚动、工作台禁用态仍可读。
- 开启系统 reduced motion 后确认部署提示和快速上手没有循环脉冲。

### 7.2 用户负责的最终真实环境验收

实际执行 AI 完成代码后，把以下最短步骤交给用户，不替用户操作真实游戏：

1. 安装新候选，进入皮肤工坊，确认每次进入都有风险确认。
2. 保持 PlayerSkinMod 未部署，确认部署按钮和气泡持续出现，任何装备选择和“应用装备”都不可写入。
3. 退出 CS2，点击部署；部署成功后确认提醒消失、装备选择开放。
4. 依次查看武器、刀具、手套、角色、音乐盒以及编辑器内皮肤/贴纸/挂件图片。
5. 完成一套 CT/T 装备并应用，在本地/离线 `-insecure` BOT 对局重生查看；不要连接 VAC 保护服务器。
6. 关闭再打开工坊：风险确认再次出现；快速上手高亮不再出现。

真实游戏内没有截图/录像、配置 JSON 回读和 PlayerSkinMod 日志前，只能称为“代码与桌面候选通过”，不能称为“游戏内修复完成”。

## 8. 验收矩阵

| 场景 | 预期结果 |
| --- | --- |
| 每次进入工坊 | 风险模态框阻塞，确认前无工坊写操作 |
| 取消风险确认 | 返回概览，不进入半解锁页面 |
| 无 CS2 目录 | 部署提醒存在，按钮按目录条件禁用并给出明确原因，工作台锁定 |
| 插件缺失/哈希错误/检查失败 | 部署按钮持续高亮 + 气泡；所有配置 mutation 和写入被阻止 |
| 部署成功并复检 ready | 提醒自动消失，工作台开放 |
| 直接 invoke save/reset 绕过 UI | Rust 返回 `[PLAYER_SKIN_MOD_REQUIRED]`，目标和 draft 均不变 |
| 首次 quick-start 条件满足 | 只高亮一次，无气泡、不自动开教程 |
| 再次进入 | 风险仍确认；quick-start 永不再高亮 |
| 20 张图片并发 | 不超过前端并发上限，最终可显示，无批量 `[IMAGE_BUSY]` 永久 fallback |
| 缓存图片 | Tauri asset protocol 可读，scope 仅限图片缓存目录 |
| 关于与来源 | 仅安装与诊断入口；五个既有/新增参考项目均可通过白名单打开 |
| reduced motion | 无循环脉冲，静态状态仍清晰 |

## 9. 停止条件与禁止事项

- 如果必须把 asset scope 放宽到整个用户目录或磁盘才能显示图片，立即停止并调查路径解析；不能为了显示成功牺牲边界。
- 如果新增写入门槛会破坏“部署过程保留已有 `player_loadout.json`”的事务语义，先区分 deploy 内部文件保留和用户主动 save/reset；禁止粗暴让部署自身调用被门槛锁死。
- 如果执行过程中发现用户正在修改同一文件，重新回读并合并，不覆盖、不 reset、不 checkout。
- 不删除或重建现有图片数据目录，不从上游全量覆盖本地生成数据。
- 不改 Demo 解析、比分、Rating、BOT、观察者、Viewer、Watcher、更新器或发布流程。
- 不升级 `0.5.7`/`1.8.2`，不新开分支，不提交、不推送、不部署、不发布安装包，除非用户在后续明确授权实际执行 AI。
- 不把“GitHub 页面可访问”写成许可证确认；2026-08-12 通过 Codex 内置浏览器确认四个新增仓库 URL 均可访问且仓库身份匹配，仅作为链接有效性证据。

## 10. 实施完成后的交付格式

实际执行 AI 最终回复必须分开写：

1. 已实现：具体文件、状态机、asset scope、图片队列、来源白名单。
2. 自动化证据：逐条命令与真实通过数量；失败或未跑必须直说。
3. 桌面 Tauri 证据：真实窗口下图片分类抽查、首次/缓存加载、控制台错误、两种窗口尺寸、reduced motion。
4. 未验证：真实 CS2 `-insecure` 游戏内结果、安装签名/打包（若未获授权）。
5. 用户最短验收步骤：使用 7.2 的六步，不附冗长日志。

完成标准不是“按钮看起来亮了”，而是：未部署时任何路径都不能写配置；部署提醒不会被永久忽略；首次帮助只提醒一次；全部饰品类别在真实 Tauri WebView 中能稳定显示；来源集中且每个外链经过固定白名单。
