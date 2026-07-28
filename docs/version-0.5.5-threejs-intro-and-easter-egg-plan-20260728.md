# 0.5.5 Three.js 开屏与版本彩蛋完整交接方案

## 1. 任务目标与边界

本文供下一位“实际执行 AI”直接实施。当前仓库是 Windows / Tauri 2 + Vue 3 + TypeScript + Vite，应用版本已经是 `0.5.5`，本次必须继续保持 `0.5.5`，不得升级任何版本号、插件 marker 或发布清单版本。

目标分为两条独立功能链：

1. 左侧导航品牌区删除“CS2 助手”，把单独的 `0.5.5` 放大并变为可点击彩蛋入口。每次点击浮动提示“别点我”；按当前版本的 minor 段决定解锁次数，`0.5.5` 为 5 次，将来的 `0.6.x` 自动变为 6 次。连续点击判定应宽松。达到次数后打开可游玩、可关闭的古风科技感 Three.js 小游戏。
2. 每次程序冷启动显示一个可跳过的 Three.js 开屏：先用 3D 场景展示官网首页“鸣谢长廊”的公开赞助信息，再展示上游 `ed0ard/CS2-Bot-Improver` 的 GitHub 信息，然后进入主程序。

非目标：

- 不改官网后台和 `E:\cs2as` 的 D1 表结构；已有公开接口足够。
- 不把彩蛋做成独立窗口、网页或 iframe，不使用浏览器自动操作实现功能。
- 不接排行榜、账号、支付、遥测或持久化分数。
- 不因开屏网络失败、WebGL 失败或资源失败阻止用户使用主程序。
- 不改现有主界面整体视觉系统；沉浸式古风科技视觉只存在于开屏和彩蛋全屏层。

## 2. 已确认的当前事实

### 2.1 应用结构

- `src/components/AppShell.vue` 的 `.sidebar-brand` 当前渲染图标、`<strong>CS2 助手</strong>` 和 `<small>{{ appConfig.appVersion }}</small>`，这就是本次导航修改点。
- `src/config/app.ts` 已通过 `__APP_VERSION__` 提供 `appConfig.appVersion`，不能再硬编码 `0.5.5`。
- `src/App.vue` 是全局层的正确挂载点，当前已有 `GlobalToast` 和更新退出弹窗。
- `src/components/AppShell.vue` 在自身 `onMounted` 中执行 CS2 目录扫描和 Panel 初始化。开屏应作为覆盖层，不能等待动画完成后才挂载 `AppShell`，否则白白延迟初始化。
- `src/components/GlobalToast.vue` 目前只监听 store/panel，尚无通用事件入口。版本彩蛋不能为 Toast 复制一套浮层，应扩展现有组件。
- `src/styles/main.css` 已有 `--modal-scrim`、motion token、焦点样式、亮暗主题及 `prefers-reduced-motion` 的既有基础，可追加独立的 intro/easter-egg 样式区。
- `package.json` 尚未依赖 `three`；当前 Vue 是 `3.5.31`，Vite 是 `8.0.3`。
- Tauri 主窗口是 `960x700`，最小 `720x620`，无原生装饰；全屏覆盖层的顶部必须避开/保留 44px 自定义标题栏，不能遮住窗口关闭按钮。

### 2.2 官网鸣谢数据

官网工作区位于 `E:\cs2as`。首页使用公开接口：

```text
GET https://cs2as.600318.xyz/api/supporters
```

响应契约由官网 `src/lib/api.ts` 和 `src/worker.ts` 证明：

```ts
interface SupporterAcknowledgement {
  id: string
  nickname: string | null
  message: string | null
  amountCents: number
  sortOrder: number
  isVisible: boolean
  createdAt: string
  updatedAt: string
}

interface SupportersResponse {
  supporters: SupporterAcknowledgement[]
}
```

2026-07-28 实测接口成功返回 4 条公开记录。桌面端只消费服务端已经过滤的公开记录，不调用 `/api/admin/supporters`，不携带 Cookie/Token。空昵称与空留言继续沿用官网默认文案：

```ts
const DEFAULT_SUPPORTER_NAME = '青锋无名客'
const DEFAULT_SUPPORTER_MESSAGE = '长夜执剑，幸与诸君同路。'
```

### 2.3 上游 GitHub 数据

上游固定为：

```text
https://github.com/ed0ard/CS2-Bot-Improver
https://api.github.com/repos/ed0ard/CS2-Bot-Improver
```

2026-07-28 实测公开 GitHub API 返回 `full_name`、`description`、`stargazers_count`、`forks_count`、`open_issues_count`、`default_branch`、`pushed_at` 和 `license.spdx_id`。动态数字只能作为“本次启动获取的快照”，不能当成产品承诺。

## 3. GitHub 候选调查与最终选型

### 3.1 彩蛋游戏：以 Slash Saber 为机制/资产候选

- 仓库：[honzaap/SlashSaber](https://github.com/honzaap/SlashSaber)
- 在线演示：[slashsaber.com](https://slashsaber.com)
- 技术：Vue 3 + Three.js，完整的鼠标挥剑、方向判定、障碍生成、生命/分数/重开闭环。
- 许可：仓库标注 `CC-BY-4.0`，需要署名、许可证链接、修改声明；README 还提出使用 3D 资产时给仓库 Star 的请求，实际执行前应尊重该请求。
- 视觉资产：刀剑、竹障碍、木人桩、古典走廊组件，和“古风、帅气、可游玩”的方向高度吻合。
- 风险：原项目包含 `cannon-es`、`postprocessing`、`three-nebula`、`three-csg-ts`、Vuetify、DRACO 等较重依赖，整包迁入会显著扩大安装体积和冲突面；部分设计容易让用户联想到特定动漫 IP，因此不能照搬 logo、名称、十套明显衍生造型或原项目品牌文案。

最终策略：**参考其交互闭环，按需复用少量确认受 CC BY 4.0 覆盖的通用场景资产；游戏逻辑在本仓库内用 Three.js 原生 Raycaster/平面交点重新实现，不整包复制源码，不引入 Vuetify、物理引擎和粒子框架。** 主题改为原创“青冥试剑”：数据剑光斩断迎面而来的竹简/能量靶，古代剑阁与青色 HUD 共存。

### 3.2 开屏：Three.js 官方能力 + 原创场景

- 基础库：[mrdoob/three.js](https://github.com/mrdoob/three.js)，MIT；2026-07-28 npm 当前版为 `0.185.1`。
- 可参考其官方 examples 中 renderer、GLTF loader、Raycaster、instancing、bloom 的写法，但不要复制整份 demo HTML。
- 可参考 [Steve245270533/character-simulator](https://github.com/Steve245270533/character-simulator) 的 Vue/Three 场景生命周期、加载进度、资源管理思路。该项目为 GPL-3.0-only，虽然本项目 AGPL-3.0-or-later 可容纳兼容代码，仍不建议搬入其大地图、角色和 NES 资源；本次不需要第三人称控制器。

开屏最终不依赖外部大模型：使用 Three.js 程序化几何构建“云海剑阵/数据碑廊”即可。低多边形山影、牌坊轮廓、悬浮金属剑、数据玉简、粒子星屑都能由 `BufferGeometry`、`InstancedMesh`、简单 shader/材质完成，视觉原创且包体更小。

### 3.3 许可证落地

若实际复用 Slash Saber 的任何源码或资产：

1. 把原始 LICENSE 保存到 `third_party/slash-saber/LICENSE`。
2. 在 `NOTICE.md` 加作者、仓库 URL、CC BY 4.0 URL、复用文件列表及“已修改为 CS2AS05 青冥试剑场景”的声明。
3. 在彩蛋“来源”按钮或游戏结束页提供可见署名，不要只埋在源码。
4. 把选中的原始资产与派生资产分开放在 `third_party/slash-saber/original/` 和 `src/assets/easter-egg/derived/`，记录 SHA256，便于复查。
5. 若无法证明某个资产受仓库许可证覆盖，立即弃用并以程序化几何替代。

Three.js 自身 MIT 许可也加入 `NOTICE.md`。不要直接引入许可证为 `NONE/NOASSERTION` 的 GitHub 项目。

## 4. 产品与视觉方案

UI/UX Pro Max 调研把此类页面归为 Immersive/Interactive Experience：深色聚焦、可跳过、加载反馈、移动/低性能降级；关键风险是性能与可访问性。实际配色不能直接套用其偏紫模板，应与现有应用的蓝/青技术色相容，并加入铜金作为古风层次，避免单一蓝紫色。

建议仅在 3D 层定义局部 token：

```css
--cinema-ink: #071014;
--cinema-jade: #43b8d7;
--cinema-cyan: #78e4ef;
--cinema-bronze: #c59a4a;
--cinema-paper: #e7edf0;
--cinema-danger: #d95a62;
```

### 4.1 导航版本入口

- 保留现有 `PackageCheck` 图标，删除可见文字“CS2 助手”。
- `0.5.5` 改为语义正确的 `<button type="button">`，而不是给 `small/div` 绑定点击。
- 字号建议桌面 `20px/1`、字重 800、tabular numbers；按钮实际命中区至少 44x44px。不要用随 viewport 缩放的字体。
- 默认仍像品牌版本号，不增加“点击五次”提示；hover/press 只出现轻微青色描边/辉光，不移动布局。
- `title` 和 `aria-label` 使用“版本 0.5.5”，不在无障碍名称中泄露彩蛋。

### 4.2 开屏叙事（建议总长 5.2 至 7 秒，始终可跳过）

1. `0.0-0.8s`：墨黑背景中剑锋/牌坊轮廓点亮，中央出现 `CS2AS / 0.5.5`。
2. `0.8-3.6s`：镜头穿过“鸣谢碑廊”，每位公开赞助者是一枚悬浮玉简；正面显示昵称，次级文本显示留言与金额。数量多时最多实例化/展示 12 条，按接口顺序；更多记录用“以及 N 位同路人”收束。
3. `3.6-5.6s`：玉简聚拢成上游项目数据碑，显示 `ed0ard/CS2-Bot-Improver`、简介、license、stars/forks、最后更新时间；动态字段缺失时使用内置静态信息，不展示假数字。
4. `5.6-6.4s`：剑光划开场景，交叉淡出到已经初始化中的主界面。

右上角始终提供 `跳过` 文字按钮和加载/阶段进度；`Esc` 跳过。动画没有声音，避免意外音频。`prefers-reduced-motion: reduce` 时不进行镜头飞行和粒子运动，改成两张静态信息画面交叉淡入，总长最多 1.2 秒，仍可立即跳过。

### 4.3 彩蛋游戏“青冥试剑”

- 全屏层位于标题栏下方，视觉是夜色剑阁、青色数据网格、铜金灯火和雾化远山。
- 玩家移动鼠标/触摸拖动形成剑痕；使用最近两个有效指针点构成挥砍线段，与屏幕投影后的靶标包围圆/方向门做命中判定。
- 靶标沿 Z 轴驶来，分普通竹简、方向符印、不可斩铜印三类。初版不做物理碰撞。
- 状态闭环：3 秒倒计时 -> 60 秒游戏 -> 结算（分数、最高连击）-> 重试/关闭。
- `Esc` 和右上角 `X` 均可关闭；关闭后彻底释放 renderer、纹理、geometry、material、RAF 和监听器，再恢复触发按钮焦点。
- 游戏分数只保存在组件本次打开生命周期，不写 localStorage，不上传。
- 不要求 pointer lock，不抢系统鼠标，不阻止 Tauri 标题栏操作；键盘不是唯一输入方式。

## 5. 点击次数与“宽松连续”精确定义

阈值必须从版本字符串解析，不能写死 5：

```ts
export function easterEggThreshold(version: string): number {
  const core = version.split('-')[0] ?? ''
  const minor = Number.parseInt(core.split('.')[1] ?? '', 10)
  return Number.isFinite(minor) && minor > 0 ? minor : 5
}
```

建议状态机：

- 每次只接受版本按钮的主键点击/键盘激活，其他界面点击不打断计数。
- 有效点击后立刻通过全局 Toast 显示标题“提示”、正文“别点我”，持续约 1.4 秒；连续点击只重置同一个 Toast 计时器，不堆叠多个 Toast。
- 从第一次有效点击开始建立一个 12 秒 rolling window；只要下一次点击距离上次不超过 4 秒就继续累计。超过 4 秒则把本次当作新序列的第 1 次，而不是归零后吞掉这次点击。
- 切换导航、鼠标离开、窗口失焦均不清零；窗口隐藏超过 12 秒或完成解锁才清零。
- 达到阈值时先清零，再打开彩蛋，防止第 6 次点击造成二次打开。
- 使用 `event.detail` 或 double-click 事件不是必要条件，不要依赖双击速度。

把纯逻辑放在 `src/features/easter-egg/version-trigger.ts`，用 fake timers 单测边界。版本 `0.6.0` 必须得到 6，`0.10.2` 得到 10，预发布 `0.5.5-beta.1` 得到 5，坏格式回退 5。

## 6. 数据与缓存设计

新增 `src/features/intro/types.ts` 与 `src/services/intro-data.ts`：

```ts
interface IntroData {
  supporters: SupporterAcknowledgement[]
  upstream: UpstreamProjectSummary
  fetchedAt: string | null
  sources: { supporters: 'network' | 'cache' | 'fallback'; upstream: 'network' | 'cache' | 'fallback' }
}
```

加载规则：

1. `Promise.allSettled` 并发请求官网和 GitHub，单请求 `AbortController` 超时 1800ms，总等待预算最多 2200ms。
2. 校验 HTTP 状态、JSON 对象形状、字符串长度、数字是否为有限非负整数；赞助最多接收 50 条，渲染最多 12 条。所有文本只通过 Vue 文本插值/Canvas texture 安全绘制，不用 `v-html`。
3. 成功响应可写入 localStorage：`cs2as:intro:supporters:v1`（TTL 24h）与 `cs2as:intro:upstream:v1`（TTL 6h）。缓存只用于启动视觉，不是权威业务状态。
4. 官网失败：优先用未超过 7 天的旧缓存；再失败则显示“鸣谢长廊暂未连接 / 感谢每一位同路人”，不能把历史姓名硬编码进应用。
5. GitHub 失败：使用内置静态上游名、URL、项目用途与 `AGPL-3.0`，stars/forks 显示 `--`，不得用构建时过期数字冒充实时值。
6. 开屏动画和数据加载并行启动。数据晚到时只替换尚未播放的阶段；不要让镜头停在空白处等待网络。

浏览器开发模式可能受 CORS 或 GitHub rate limit 影响，这必须被视为正常降级路径。不要申请 GitHub token，更不能把 token 打进客户端。若 WebView2 对 GitHub API 的公开 CORS 实测不稳定，可在 `E:\cs2as` 增加一个只读聚合代理作为后续项，但本轮优先不改官网。

金额是个人公开鸣谢数据，按官网现有产品行为展示。建议开屏以“¥20.00”格式显示，绝不汇总为可用于财务判断的总额。

## 7. 组件和文件拆分

建议新增：

```text
src/
  components/
    intro/StartupIntro.vue
    easter-egg/EasterEggGame.vue
    three/ThreeStage.vue
  features/
    intro/types.ts
    intro/scene.ts
    easter-egg/version-trigger.ts
    easter-egg/game-scene.ts
  services/intro-data.ts
  assets/easter-egg/derived/        # 仅在确认复用资产时建立
tests/
  version-easter-egg-trigger.spec.ts
  intro-data.spec.ts
  intro-lifecycle.spec.ts
third_party/slash-saber/            # 仅在实际复用时建立
```

职责规则：

- `ThreeStage.vue` 只管理 canvas 容器、ResizeObserver、DPR、WebGL context lost/restored、RAF 的开始/暂停/销毁；通过工厂函数接收 intro/game scene，避免两个组件各写一套易泄漏生命周期。
- `intro/scene.ts` 只处理 Three.js 场景和阶段切换，不直接 fetch。
- `game-scene.ts` 只处理靶标池、指针映射、命中与分数事件，不依赖 Pinia。
- `StartupIntro.vue` 管数据、skip、焦点和阶段文案。
- `EasterEggGame.vue` 管游戏 UI、关闭/重试和来源署名。
- `App.vue` 管 `introOpen` / `easterEggOpen` 两个顶层 overlay；不可同时打开。
- `AppShell.vue` 仅负责版本按钮触发，向上 emit 或派发有类型的 CustomEvent；优先 emit，避免新的全局事件耦合。

依赖只新增：

```powershell
npm install three
npm install -D @types/three
```

先检查当前 Three.js 包是否已自带完整类型；如果自带且 TypeScript 可通过，则不装多余 `@types/three`。不要引入 TresJS、React Three Fiber、GSAP、Vuetify、Cannon 或 postprocessing，初版使用原生 Three.js 即可。

## 8. 全局 Toast 的最小扩展

建议在 `src/components/GlobalToast.vue` 增加一个 `cs2as:toast` window event 监听，payload 沿用 `ToastMessage`，并在 `onBeforeUnmount` 清理；同时封装：

```ts
export function dispatchToast(message: ToastMessage) {
  window.dispatchEvent(new CustomEvent<ToastMessage>('cs2as:toast', { detail: message }))
}
```

事件接收处必须做运行时 shape 校验，不能无条件信任 `detail`。为彩蛋支持可选 `durationMs`，默认仍是现有 4000ms，范围夹在 800 至 10000ms。不要改变 store 和 panel 错误 Toast 的现有行为。

## 9. Three.js 生命周期与性能硬约束

1. `StartupIntro` 和游戏都必须动态 import `three` 及场景模块。主界面首包不应包含完整 Three.js；只有开屏会立即加载 intro chunk，游戏 chunk 必须等第 5 次点击才加载。
2. renderer DPR：`Math.min(devicePixelRatio, 1.5)`；平均帧时间持续超过 24ms 时降到 1.0，并关闭雾粒子/阴影。
3. 初始场景 draw calls 目标 `< 80`，活动靶标 `<= 24`，粒子使用 `Points` 或 `InstancedMesh`，禁止每帧创建临时 Vector/Material。
4. canvas CSS 尺寸稳定为 overlay 可用区，不因加载文字/分数变化而重排。
5. `document.hidden` 时暂停 RAF；overlay 关闭时 cancel RAF、disconnect ResizeObserver、remove listeners，并逐一 dispose geometry/material/texture/renderer。
6. 捕获 `webglcontextlost`，显示静态 fallback 和“进入助手/关闭游戏”，不能反复创建 renderer。
7. WebGL 初始化失败、动态 import 失败、模型失败均要被捕获；开屏自动进入静态信息卡，游戏显示“当前设备无法启动彩蛋”并可关闭。
8. 开屏资源总压缩体积建议 `< 1.5MB`，游戏额外资源 `< 4MB`。若 Slash Saber 资产超预算，只选 1 把通用剑、2 类竹靶和 1 套走廊模块，使用 glTF Transform/Draco 前先权衡 decoder 体积；小模型优先 Meshopt 或直接未压缩 GLB。

## 10. 无障碍、焦点和输入

- overlay 使用 `role="dialog" aria-modal="true"`，进入时焦点放到“跳过”或“关闭”，关闭后回到版本按钮。
- 开屏 3D canvas 标记 `aria-hidden="true"`，赞助/上游信息另有屏幕阅读器可读的 DOM 文本；游戏提供简短状态 `aria-live="polite"`，不要逐帧播报分数。
- `Escape` 始终是退出路径；Tab 不得进入 overlay 背后的主界面。实现轻量 focus trap，不能禁用系统快捷键。
- 触摸/鼠标共用 Pointer Events，并设置合理 `touch-action`；不依赖 hover。
- `prefers-reduced-motion` 和 renderer 性能降级两条路径都必须测试。
- 所有按钮保持至少 44x44px，关闭按钮使用现有 Lucide `X`，带 tooltip/title/aria-label。

## 11. 实施顺序

### 阶段 A：建立可测试的基础

1. 保留工作区现有改动，先运行 `git status --short` 并记录；不要回退其他 0.5.5 文件。
2. 安装 Three.js 最小依赖，确认 lockfile 没有版本字段漂移。
3. 先实现 `version-trigger.ts`、`intro-data.ts` 及单测。
4. 扩展 GlobalToast 通用入口，保证旧 Toast 测试通过。

### 阶段 B：导航入口和彩蛋闭环

1. 修改 `AppShell.vue` 品牌区和 emit。
2. 在 `App.vue` 接住触发并动态挂载 `EasterEggGame.vue`。
3. 先用程序化方块靶实现 60 秒完整游戏，再做古风美术；不得先堆视觉导致玩法不可验收。
4. 完成资源 dispose、Esc/关闭/重试、WebGL fallback 后再接入选定 GLB。
5. 更新 NOTICE 和 third_party 证据。

### 阶段 C：开屏

1. 在 `App.vue` 上层挂载 `StartupIntro`，但 `AppShell` 继续同步挂载和初始化。
2. 先完成静态 fallback、跳过和数据加载，再接 Three.js scene。
3. 实现赞助碑廊 -> 上游数据碑 -> 主程序的时间线，所有阶段可被 skip 中断。
4. 最后补 reduced-motion、低性能降级和缓存。

### 阶段 D：收口

1. 扫描版本：`package.json`、lockfile、Cargo、tauri config 仍为 `0.5.5`。
2. 更新 `tests/app-brand.spec.ts`，旧断言当前要求存在“CS2 助手”，必须改为验证其删除、版本按钮存在且版本来自 `appConfig`。
3. 运行简化但覆盖关键面的验证，真实 3D 游戏手感和 Windows WebView2 表现交由用户最终体验。

## 12. 测试与验收矩阵

自动化最低要求：

```powershell
npm run typecheck
npm run lint
npm run test
npm run build:web
```

针对性用例：

- 版本解析：0.5.5=5、0.6.x=6、0.10.x=10、预发布、坏格式。
- 点击：4 次不打开；第 5 次打开；超过 4 秒后的本次成为新序列第 1 次；导航切换/blur 不清零；打开后计数清零。
- 每次点击只存在一个“别点我” Toast，快速点击不会创建 5 个 DOM Toast。
- supporters：正常、空数组、字段坏损、超时、缓存命中、超 TTL、超量截断、文本超长。
- GitHub：正常、403 rate limit、超时、无 license、无 description、fallback 无假数字。
- intro：skip 会立即停止 timeline 和 RAF；动画自然结束只触发一次 close；数据晚到不把已经结束的 overlay 重新打开。
- lifecycle：连续打开/关闭游戏 3 次，RAF/resize/pointer/keydown 监听器数量不增长，renderer dispose 被调用。
- WebGL unavailable/context lost：能看到静态 fallback 并关闭。
- reduced motion：没有连续镜头/粒子 RAF，最多 1.2 秒或用户立即跳过。

用户手工验收（Windows 桌面候选包）：

1. 冷启动，标题栏关闭/最小化始终可用；开屏期间后台目录扫描继续，跳过后主界面状态已开始加载。
2. 联网看到与官网当前一致的鸣谢；断网启动不白屏、不阻塞，并出现缓存或通用感谢文案。
3. 上游信息明确写 `ed0ard/CS2-Bot-Improver`，动态统计不冒充实时保证。
4. 左栏没有“CS2 助手”，`0.5.5` 更醒目但不破坏窄窗口导航。
5. 以宽松节奏点 5 次成功进入；停顿超过 4 秒后重新点仍把该次算作第 1 次。
6. 鼠标挥斩可命中、计分、失败/结算、重试，随时可关闭；不会影响主界面当前导航和数据。
7. 连开/关 3 次后任务管理器 GPU/内存不持续明显上涨。
8. Windows 缩放 100%/150%、窗口 `720x620` 与 `960x700` 均无文字遮挡。

## 13. 完成标准与交付记录

只有以下全部成立才可报告完成：

- 两条流程都能从干净冷启动/全新页面状态稳定复现。
- 任何网络或 WebGL 故障都不会阻断主程序。
- 版本仍为 `0.5.5`，已有启动、安装、更新、Panel 功能测试无回归。
- 所有复用资源有明确许可证、署名、修改声明和来源路径；不能以“GitHub 上公开”替代许可证证据。
- 最终交付记录包括：实现 commit（若用户要求提交）、新增依赖及版本、资源路径/体积/SHA256、NOTICE 变更、自动化结果、Windows 用户手工验收结果和仍存限制。

本轮方案制定只新增本文档，不修改应用代码、不安装 Three.js、不构建和发布。实际执行 AI 应以实时运行行为为准，若实现期间发现官网接口或 GitHub 候选发生变化，先更新证据再调整实现。
