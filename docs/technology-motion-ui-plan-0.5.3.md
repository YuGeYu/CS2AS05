# 0.5.3 科技动效与界面美化实施方案

本文件是给下一位实际执行 AI 的完整交接依据。执行 AI 不需要本轮聊天、截图或其他 AI 的上下文；以本文记录的基线、文件、交互契约、测试和验收条件为准。

- 状态：待实际执行 AI 实施
- 方案日期：2026-07-26
- 工作区：`E:\CS2AS05`
- 当前分支：`main`，通常不新开分支
- 当前基线：`ba51d0b8cc13cad8e8a7eecf0b3e28ecbc186eb1`
- 当前版本与目标版本：`0.5.3`，本次继续打 `0.5.3`，不得升为 `0.5.4`
- 当前工作树：制定本方案前为干净状态
- 技术栈：Vue 3.5、Pinia 3、Tauri 2、TypeScript、Vitest、Lucide Vue、单一全局样式 `src/styles/main.css`

## 1. 目标与名词校正

本次只改现有桌面程序的视觉与交互反馈，不改变 Panel 配置、CS2 启动、安装、目录扫描、命令内容或持久化语义。完成以下内容：

1. 主导航切换时出现克制、连续、具有科技仪表感的活动游标和页面进出动效。
2. **概览页**（用户原话中的“预览页面”指当前 `OverviewView.vue`）的启动模式与 BOT 难度分段控件增加滑动指示器和状态切换反馈。
3. 人机预设页的 Aim 与 **Nades** 分段控件增加同一种动效。用户原话“Nodes”是笔误，当前产品、类型和磁盘配置均使用 `Nades`，不得改成 Nodes。
4. Bot 物品某项从关闭变为开启且后端写入成功后，显示一次短促成功动效。
5. 刀具从未选中变为选中且后端写入成功后，显示一次短促选中动效。
6. 命令页仅在 Tauri 剪贴板写入成功后显示科技感复制反馈；失败必须显示错误，不能播放成功动效。
7. 全局表面、边线、标题和焦点态做一轮克制的科技感统一，不改变现有信息密度和布局层级。

“科技感”在本项目中的定义是：精确移动的指示器、短边线扫描、清晰状态色、层次明确的技术面板和约 140-320ms 的微交互。它不等于赛博朋克堆叠。

## 2. 明确不做

- 不改任何 Rust/Tauri command、Panel 文件读写、启动参数、安装包资源、版本门禁、CS2 目录扫描或默认配置。
- 不改导航信息架构、页面文案、选项值、刀具 ID/图片来源、命令数据或主题持久化。
- 不采用浏览器 View Transitions API、原生 Clipboard API、`alert/prompt/confirm` 或仅浏览器环境可用的功能；命令复制继续使用 `@tauri-apps/plugin-clipboard-manager`。
- 不加入 WebGL、canvas、粒子、音效、外部字体、视频背景、渐变光球、紫色主色、持续闪烁或持续空闲动画。
- 不把所有页面区块改成悬浮卡片，不新增卡片套卡片；卡片圆角保持不超过 8px。
- 不为了动画引入第三方动画库。Vue `<Transition>`、CSS transition/keyframes 和现有响应式状态足够。
- 不改现有启动 CS2 全屏状态特效 `LaunchExperience.vue` 的退出条件和业务逻辑；只保证新全局样式不破坏它。
- 本轮方案制定阶段不实现业务代码、不构建安装包、不提交、不推送。以下步骤由实际执行 AI 完成。

## 3. 已核实的当前实现

### 3.1 导航和页面容器

`src/components/AppShell.vue`：

- `current` 是本地 `ref<ViewKey>('overview')`，六个导航按钮直接执行 `current = item.key`。
- 选中态只依赖 `button[aria-current='page']` 的边框、背景与文字色。
- 页面使用 `<component :is="view" />` 直接替换，没有 Vue 过渡，也没有共享活动游标。
- `max-width: 860px` 后侧栏变成横向图标导航；`max-width: 700px` 后品牌区只保留图标。

### 3.2 分段控件

`src/components/ui/SegmentedControl.vue`：

- 通过 `modelValue`、`options`、`disabled`、`label` 和 `update:modelValue` 复用。
- 当前选中态只由 `button[aria-pressed='true']` 改背景和阴影。
- 调用位置包括：
  - `src/views/OverviewView.vue`：启动模式、BOT 难度。
  - `src/views/PresetsView.vue`：Aim、Nades、队伍阵营 CT/T。
- Pinia 的 `src/stores/panel.ts::mutate` 在 Tauri operation 成功返回 `PanelSnapshot` 后才替换 `snapshot`，失败会恢复旧快照并写入 `lastError`。因此移动指示器必须跟随确认后的 `modelValue`，不能在点击时乐观移动。

### 3.3 Bot 物品

`src/components/ui/ToggleSwitch.vue` 当前只有轨道变色和圆点平移。`src/views/BotItemsView.vue` 调用 `panel.setBotItem`，但用 `.catch(() => undefined)` 吞掉 Promise 结果；真实错误仍由 store 写入 `panel.lastError`。

开启反馈必须满足：初次挂载时即使值为 `true` 也不播放；只有值由 `false` 变为经后端确认的 `true` 才播放；关闭动效更安静；卸载时清理计时器。

### 3.4 刀具

`src/views/KnivesView.vue`：

- 20 张刀具卡已有本地图片、`aria-pressed`、Check 图标、多选、全选、清空和按键绑定。
- `toggle(id)` 当前立即计算新数组并 `void save(...)`，未等待后端成功。
- `save` 调用 `panel.setDropKnives(...).catch(() => undefined)`。
- 选中态当前只有边框、背景、左侧 inset line 和 Check 图标，无瞬时反馈状态。

### 3.5 命令复制

`src/views/CommandsView.vue` 当前动态导入并调用：

```ts
const { writeText } = await import('@tauri-apps/plugin-clipboard-manager')
await writeText(value)
```

成功后只更新 `.command-result` 文本。按钮没有临时状态，失败没有本页可见的捕获分支。复制反馈必须以 `writeText` resolve 为成功边界。

### 3.6 当前样式和无障碍基础

`src/styles/main.css` 已有 light/dark 两套语义色、明确 focus ring、响应式布局和全局 `prefers-reduced-motion: reduce` 处理。现有启动特效使用 `launch-spin`/`launch-spin-reverse`。新实现必须扩展这些基础，不能覆盖或删除现有减弱动画规则。

窗口基准来自 `src-tauri/tauri.conf.json`：默认 `960x700`，最小 `720x620`。另需覆盖 `1280x800` 宽屏。

## 4. 统一视觉与动效语言

先在 `src/styles/main.css` 的 light/dark token 区补充语义 token，具体颜色可在实际截图中微调，但命名和用途保持稳定：

```css
--tech-accent: #0b8fbf;          /* 克制青色，仅用于扫描边线/短反馈 */
--tech-accent-soft: rgba(...);   /* 浅底或暗底的低对比填充 */
--tech-line: rgba(...);          /* 技术分隔线/网格，低对比 */
--motion-fast: 140ms;
--motion-standard: 220ms;
--motion-emphasis: 320ms;
--ease-out-tech: cubic-bezier(.2, .8, .2, 1);
--ease-in-tech: cubic-bezier(.4, 0, 1, 1);
```

暗色主题必须单独给出可读的 `--tech-accent-soft` 和 `--tech-line`，不能只复用浅色透明度。原有蓝色 `--primary` 仍是主操作和选中状态；青色只做短暂边线/扫描；绿色 `--success` 只表示成功；琥珀色 `--warning` 和红色 `--danger` 语义不变。

统一规则：

- 进入或确认采用 ease-out，退出采用 ease-in。
- 一次交互最多突出 1-2 个元素，不让整页同时动。
- 主要使用 `transform` 与 `opacity`；不要动画化布局宽高、padding、网格轨道或造成滚动位置跳动的属性。
- 所有固定格式控件保留稳定高度：导航 44px、分段控件 44px、toggle 46x26px、刀卡最小 138px、命令行最小 40px。
- hover 只能辅助鼠标用户；键盘 focus、`aria-current`、`aria-pressed`、checkbox 语义和禁用态必须独立成立。

## 5. 文件级实施步骤

### 5.1 `src/components/AppShell.vue`：导航游标与页面切换

1. 将导航点击集中到 `selectView(key)`；相同 key 再次点击不重启动效。
2. 计算 `activeNavIndex`，在 `<nav>` 内加入一个 `aria-hidden="true"` 的 `.nav-cursor`，以 CSS 变量 `--nav-index` 定位。
3. 不使用 DOM 测量或 View Transitions API。导航目前固定六项，执行 AI应在 CSS 中固定每项 44px 和已知 gap：
   - 桌面侧栏游标沿 Y 轴 `translate3d(0, calc(var(--nav-index) * 47px), 0)`；宽度占 nav 内容区。
   - `max-width: 860px` 后所有图标按钮固定 44px 宽，游标沿 X 轴移动。
   - 游标自身 `pointer-events: none`，导航按钮保持在它上层；不能遮挡点击与焦点框。
4. 游标视觉使用 1px 边框、左侧/底部短青色刻度和很浅的背景，不做发光大阴影。移动时长约 220ms。
5. 将页面组件改为带 key 的 Vue 过渡：

```vue
<Transition name="view-swap" mode="out-in">
  <component :is="view" :key="current" />
</Transition>
```

6. `.view-swap` 仅做 180-220ms 的 `opacity` 加 6-10px 单向位移或短 clip reveal。`mode="out-in"` 防止两个完整页面叠在一起。不要给 `.view-container` 本身 transform，以免影响 fixed overlay；不得破坏 `LaunchExperience`。
7. 保留原 `aria-current="page"`、title、aria-label 和键盘原生 button 行为。

### 5.2 `src/components/ui/SegmentedControl.vue`：共享滑动指示器

1. 增加可选 `pending?: boolean` prop。根节点在 pending 时设置 `aria-busy="true"`；各 button 的 disabled 为 `disabled || pending`，阻止重复提交。
2. 计算当前 option index；当 `modelValue === null` 或不在 options 中时隐藏指示器，不默认指向第一个选项。
3. 在按钮前加入 `.segmented__indicator[aria-hidden=true]`，根元素输出：

```ts
{ '--segment-count': options.length, '--segment-index': activeIndex }
```

4. 指示器绝对定位在 3px 内边距中，宽度为可用宽度除以 count，以 `translate3d(calc(var(--segment-index) * 100%), 0, 0)` 移动，约 220ms ease-out。按钮设相对层级并保持透明背景，让文字位于指示器之上。
5. `aria-pressed` 仍是唯一权威选中语义。不得仅通过颜色或动画表达选择。
6. 根节点可在 pending 时显示一条很短的低对比扫描线，但成功移动只在父级 `modelValue` 因 store 成功返回而变化后发生。失败时指示器留在旧值。

调用方调整：

- `src/views/OverviewView.vue`
  - 启动模式传 `:pending="panel.mutationKey === 'mode'"`。
  - BOT 难度传 `:pending="panel.mutationKey === 'difficulty'"`。
  - `changeMode`、`changeDifficulty` 保持返回 store Promise，不引入本地乐观值。
- `src/views/PresetsView.vue`
  - Aim 传 `:pending="panel.mutationKey === 'aim'"`。
  - Nades 传 `:pending="panel.mutationKey === 'nades'"`。
  - CT/T 本地分段控件也会自然获得共享滑动样式，但不需要 pending，也不得改变复制队伍命令的逻辑。

### 5.3 `src/components/ui/ToggleSwitch.vue` 与 `src/views/BotItemsView.vue`

推荐由共享组件观察经后端回写的 prop，以免父页复制动画逻辑：

1. `ToggleSwitch.vue` 增加 `ref(false)` 的 `justEnabled`，在组件 mount 完成后 watch `modelValue`。
2. 仅当 `previous === false && current === true` 时把 `justEnabled` 置 true，约 420ms 后复位。`immediate` 必须为 false，所以首次渲染的 true 不动。
3. 再次触发先清旧 timer；`onBeforeUnmount` 清 timer。
4. 根 label 添加 `is-just-enabled` class。CSS 只做：轨道一圈短促青/绿边线、圆点轻微 settle（不超过 1.06 倍）、标题或右侧一次淡入刻度；不要整行闪白。
5. true -> false 只保留现有圆点回移和颜色淡出，不播放成功扫描。
6. `BotItemsView.vue::update` 继续由 `panel.setBotItem` 控制快照；建议改为 `async/await + try/catch`，但不得清除 store 的 `lastError`。页面应像概览一样提供 `panel.lastError` 的 `role="alert"` 可见错误，否则用户只看到开关回弹。
7. mutation 期间现有 `!!panel.mutationKey` 会禁用列表。失败时 prop 不变或恢复，绝不能出现 `is-just-enabled`。

### 5.4 `src/views/KnivesView.vue`：单刀确认动效与批量反馈

1. 新增 `recentlySelectedId: Ref<number | null>`、`selectionNotice` 和两个可清理 timer；在 unmount 时与键盘监听一并清理。
2. 把 `toggle(id)` 改为 async：先记录本次是 adding 还是 removing，再 `await panel.setDropKnives(...)`。
3. 只有 adding 且 Promise resolve 后设置 `recentlySelectedId = id`，约 450ms 后清除；失败仅保留 store 错误和旧选中状态。
4. 卡片绑定 `:class="{ 'is-just-selected': recentlySelectedId === knife.id }"`。该 class 执行一次：
   - 1px 青色边线从一侧扫过；
   - 图片 `scale(1) -> scale(1.03) -> scale(1)`；
   - Check 图标以 opacity/scale 进入。
5. deselect 不播放成功扫线，只用现有选中态自然退出。
6. **全选不能让 20 张卡同时播放动画。** 将全选/清空抽成 async 批量 handler：成功后只在 `.selection-actions` 显示一条约 1 秒的 `aria-live="polite"` 摘要（如“已选择全部 20 款刀具”），并给操作栏一次短边线反馈。清空使用普通状态反馈，不使用绿色成功爆发。
7. 初次 mount 时即使 20 把刀全选也不能播放任何 `.is-just-selected` 动效。
8. 在刀具页底部或现有错误区域显示 `panel.lastError`，保留 `role="alert"`。

### 5.5 `src/views/CommandsView.vue`：复制成功与失败

1. 将 `copy` 签名改为 `copy(id: number, value: string)`，按钮调用传入 `entry.id`。
2. 新增：
   - `copiedId: Ref<number | null>`；
   - `copyStatus` 文本；
   - `copyTone: Ref<'success' | 'error' | null>`；
   - 一个 1000-1400ms 的 timer，重复复制时先清除并重新计时，unmount 时清除。
3. 只有以下 Promise 成功后才设置 `copiedId` 和 success：

```ts
const { writeText } = await import('@tauri-apps/plugin-clipboard-manager')
await writeText(value)
```

4. 复制成功的按钮设置 `data-copied="true"`，显示 Lucide `Check` 替换 `Copy`，行内出现一次短青色边线追踪与浅绿色确认色。按钮尺寸、命令文本换行和列表滚动位置必须稳定。
5. import 或 writeText reject 时：清除 copiedId，显示简短错误（例如“复制失败，请重试。”），`copyTone='error'`，不得显示 Check 或播放成功动画。
6. `.command-result` 保留 `aria-live="polite"`；错误可增加 `role="alert"` 或使用独立 assertive 区域。尾随空格成功文案必须继续区分，不能 trim 要复制的 value。
7. 继续使用 Tauri clipboard 插件，不改成 `navigator.clipboard`。

### 5.6 `src/styles/main.css`：全局科技感收口

在不改页面结构的前提下统一以下细节：

- `workspace-shell`/`.workspace-main`：用极低对比 `--tech-line` 做一层静态细网格或水平刻度纹理，透明度必须在 light/dark 下都不妨碍文字。纹理静止，不做无限移动。
- `.sidebar-brand`、`.view-heading`、`.control-band`、`.launch-band`：补一处短角标或边线细节，不要每个元素都有四角装饰。
- `.control-group`：保留 8px 圆角、白/暗色实底与当前信息密度；hover 仅轻微改边框，不浮起整张卡。
- 按钮/输入/选择器：统一 140ms hover、pressed 和 focus-visible 反馈；focus ring 对比度不低于现状。
- `.overline`：可增加静态短线或小刻度，但 letter-spacing 继续为 0，不能用负字距。
- `success/warning/danger` 仍各自独立，不把全部状态染成蓝/青。
- 不对标题字号做 viewport 缩放；不改变 `720x620` 最小窗口下的文本容纳与滚动策略。

## 6. 减弱动画、无障碍与性能

1. 保留并扩展现有 `@media (prefers-reduced-motion: reduce)`：
   - nav cursor 和 segmented indicator 立即定位，不滑动；
   - 页面只做立即显隐或完全无过渡；
   - toggle、刀卡、命令复制不执行 keyframes/扫描线，仍保留静态选中、Check 和文字反馈；
   - 现有 launch rings 保持不旋转。
2. 不能用动画替代文字或 ARIA。复制结果、批量刀具结果和失败都要有可读文本。
3. 键盘 Tab 顺序、Enter/Space 激活、focus-visible、disabled、`aria-current`、`aria-pressed` 和 checkbox 标签关系不得退化。
4. 动效只作用于当前交互元素；不创建全局 requestAnimationFrame 循环，不在页面隐藏后继续计时。所有 setTimeout 在 unmount 清理。
5. 优先 transform/opacity，并仅在短暂动画 class 上使用 `will-change`；不要长期给整个列表或所有卡片开启 GPU layer。

## 7. 预计修改文件

必须检查并按需修改：

```text
src/components/AppShell.vue
src/components/ui/SegmentedControl.vue
src/components/ui/ToggleSwitch.vue
src/views/OverviewView.vue
src/views/PresetsView.vue
src/views/BotItemsView.vue
src/views/KnivesView.vue
src/views/CommandsView.vue
src/styles/main.css
```

建议新增测试：

```text
tests/navigation-motion.spec.ts
tests/segmented-motion.spec.ts
tests/toggle-motion.spec.ts
tests/knife-selection-motion.spec.ts
tests/command-copy-feedback.spec.ts
```

如能保持测试清晰，可以合并成较少文件；不能把行为测试替换为单纯 `readFileSync(...).toContain(...)` 的 CSS 字符串断言。不要修改 Rust、ZIP、刀图、版本号或安装配置，除非实际执行中发现与本方案直接相关且能证明必须修改；此时先在执行报告中解释。

## 8. 自动化测试契约

使用 Vitest fake timers、Vue Test Utils 与现有 Pinia/mock 方式覆盖：

1. **导航**
   - 初始 overview 的 `aria-current=page` 与 `--nav-index=0`。
   - 点击不同导航后只有一个 aria-current，游标 index 正确，动态页面带新的 key。
   - 重复点击当前项不制造额外状态。
2. **SegmentedControl**
   - null 时无活动 indicator；2/3/4 个 options 的 count/index 正确。
   - pending 时 `aria-busy=true` 且按钮禁用。
   - emit 后父级 prop 未更新前 indicator 不移动；成功更新 prop 后才移动；失败保持旧 index。
3. **ToggleSwitch**
   - 初始 true 不带 `is-just-enabled`。
   - false -> true 后短暂带 class，timer 到期清除。
   - true -> false 不带成功 class；unmount 后无遗留 timer/warning。
4. **刀具**
   - mock `setPanelDropKnives` resolve 后，新增的单张卡短暂获得 class。
   - reject 时卡片不获得 class，旧 `aria-pressed` 不变，并显示错误。
   - deselect 不触发成功 class。
   - 全选成功不使 20 卡同时加 class，只显示单个 aria-live 摘要。
5. **命令复制**
   - mock Tauri `writeText` resolve 后仅对应行 `data-copied=true`、Copy 换 Check、aria-live 文案正确。
   - reject 后显示错误，无 copied data 与 Check。
   - 尾随空格原样传给 writeText；重复复制重置 timer；unmount 清 timer。
6. **回归**
   - 现有 `tests/cs2-process-polling.spec.ts`、`tests/knife-assets.spec.ts`、`tests/panel-data.spec.ts`、`tests/launch-experience.spec.ts` 必须继续通过。

建议执行顺序：

```powershell
cd E:\CS2AS05
npm run typecheck
npm run lint
npm test
npm run build:web
npm run verify
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
git status --short
```

`npm run verify` 已包含 workspace check、typecheck、lint、test 和 web build，但仍先跑窄测试便于定位。此任务没有 Rust 预期改动，`cargo test` 是发布前回归，不应为了节省时间跳过。

## 9. 真实桌面验收矩阵

Web 单测不能证明 Tauri 剪贴板、暗色渲染和真实窗口动画观感。实际执行 AI 在自动化通过后运行：

```powershell
cd E:\CS2AS05
npm run dev:desktop
```

让用户或具备桌面观察能力的执行环境在以下尺寸逐一检查，记录截图/短录屏和结果：

| 尺寸 | 主题 | 必查内容 |
| --- | --- | --- |
| `720x620` | light/dark | 横向图标导航不挤压，2 列刀卡，控件和文字不重叠，页面可滚动 |
| `960x700` | light/dark | 默认窗口完整流程、导航游标、分段滑块、toggle、刀卡、复制反馈 |
| `1280x800` | light/dark | 内容不过度拉伸，技术纹理不过强，留白与信息密度合理 |
| 上述至少默认尺寸 | reduced motion | 无滑动/扫描/旋转，静态状态和文字反馈仍完整 |

逐项手工验收：

1. 六个导航连续切换，游标方向在桌面/横向导航下正确，页面不叠层、不闪白、不改变滚动容器尺寸。
2. 概览页四类实际切换：模式、难度；人机预设页 Aim、Nades。成功后指示器落到磁盘回读值；制造/模拟失败时留在旧值并显示错误。
3. Bot 物品先关后开：只有成功开启有一次反馈；页面初始全开不集体闪动。
4. 单选一把未选刀，只有该卡动；取消选择不播放成功效果；全选不触发 20 张并发动画。
5. 命令页复制普通命令和带尾随空格命令，实际粘贴核对内容；再模拟 clipboard 失败验证错误分支。
6. 检查现有启动 CS2 特效、目录推荐弹窗、安装与诊断页、主题切换、标题栏拖动/窗口按钮均未被全局样式破坏。
7. 开发者检查动画期间无明显 layout shift、横向滚动条或长任务；快速重复点击不会留下永久 class、Check 或 busy 状态。

真实 CS2 不需要因纯 UI 动效重复启动才能判断大部分验收，但 Overview 的后端确认链、Bot 配置回写和刀具回写应至少在一套可用测试目录中验证；若需要真实游戏内确认，由用户执行并回报，执行 AI 不得把未执行项写成已通过。

## 10. 构建、打包与发布边界

所有自动化与桌面验收通过后，仍保持版本 `0.5.3`。用户要求实际执行 AI 一并发布时才运行：

```powershell
npm run bundle:desktop
npm run release:manifest
```

打包后报告 NSIS 安装程序绝对路径、字节数、SHA256、文件版本和生成时间，并实际验证安装/启动/卸载或明确列出未验证项。一次执行最多尝试 5 次安装包构建。除非用户明确要求，不自动新开分支；推送 GitHub 最多尝试 1 次。不得因本次 UI 改动重新生成 `CS2BotImprover.zip`。

## 11. 回退方案

1. 开始实施前记录 `git status --short` 和当前 HEAD，不清理、不 reset、不覆盖用户已有改动。
2. 所有新增状态仅在 Vue 组件内，所有视觉规则集中在 `main.css`；若单个效果有问题，优先按功能块回退对应 class/token，不回退已经发布的 0.5.3 Panel、刀图、插件门禁或启动特效。
3. 若共享 `SegmentedControl` 动效导致某个选项布局异常，先回退 indicator/pending 增量，保留原 `aria-pressed` 实现；不要复制五套分段组件。
4. 若页面 Transition 影响 fixed overlay 或滚动，单独撤销 `<Transition>`，导航游标可以保留。
5. 若安装包验证失败，保留构建日志和此前可用安装包；不要删除用户安装目录或游戏文件。
6. 最终只 stage 本任务实际修改文件；不使用 `git reset --hard`、`git checkout --` 或清理无关 untracked 文件。

## 12. 完成定义与交接报告模板

只有同时满足以下条件才算完成：

- 七项用户需求均有可见、克制且一致的实现；成功动画均由权威成功状态触发，失败没有假成功。
- light/dark、三种窗口尺寸和 reduced motion 均无重叠、截断、错位或不可操作控件。
- 键盘与 ARIA 语义不退化，所有 transient timer 可清理。
- 全量 `npm run verify`、Rust 回归和 `git diff --check` 通过。
- 桌面验收有记录；未做真实 CS2 或安装验证时明确写“未验证”，不能推断为通过。
- 版本仍是 `0.5.3`，安装与游戏业务逻辑没有被改写。

实际执行 AI 最终报告至少包含：

```text
基线 HEAD / 最终 HEAD：
实际修改文件：
导航与页面过渡实现：
Segmented/Toggle/Knife/Copy 的成功与失败边界：
reduced-motion 处理：
新增测试与结果：
npm run verify：
cargo test：
桌面尺寸/主题验收：
真实 CS2 验收（已执行或未执行）：
安装包路径/大小/SHA256（如已打包）：
git status --short：
未完成项与风险：
```
