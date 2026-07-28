# 0.5.5 Three.js 冻结与赞助鸣谢展示修复交接方案

## 1. 交接目标

本文供下一位“实际执行 AI”直接实施。两个工作区都在 Windows：

- 桌面程序：`E:\CS2AS05`
- 官网/Cloudflare Worker：`E:\cs2as`

应用必须继续保持 `0.5.5`，不得再升级版本号。本次是在已经实现的开屏和彩蛋基础上修复，不回退现有 0.5.5 改动。

用户提供了两张 1202x877 截图：

- `C:\Users\GOPtZ\Pictures\Screenshots\屏幕截图 2026-07-28 124543.png`
- `C:\Users\GOPtZ\Pictures\Screenshots\屏幕截图 2026-07-28 124617.png`

按画面内容可确认一个画面是“青冥试剑”游戏：HUD 的时间和分数在变化，但 WebGL 画面停留在随机初始帧；另一个画面是开屏：显示“鸣谢长廊暂未连接”，背景只有程序化剑、网格、矩形碑块。文件先后顺序不影响以下结论。

本次必须完成两件事：

1. 让开屏可靠取得官网公开赞助数据，并用真正运动的 3D 古风科技场景逐位展示赞助者，而不只是把数据放在静态 DOM 卡片里。
2. 修复彩蛋游戏只渲染初始帧的问题，证明靶标、场景和剑痕在连续帧中运动；修正倒计时/结束状态仍在后台模拟的问题。

## 2. 当前实现和关键文件

桌面程序当前新增代码位于：

```text
src/components/three/ThreeStage.vue
src/components/three/stage.ts
src/components/intro/StartupIntro.vue
src/components/easter-egg/EasterEggGame.vue
src/features/intro/scene.ts
src/features/easter-egg/game-scene.ts
src/services/intro-data.ts
```

已安装 `three 0.185.1` 与 `@types/three 0.185.0`。场景、几何和玩法是程序化原创，不依赖第三方游戏资产。

官网当前公开接口：

```text
GET https://cs2as.600318.xyz/api/supporters
```

2026-07-28 12:52 实测得到 HTTP 200 和 4 条赞助记录，说明接口和 D1 数据都已部署。问题不是“接口不存在”，而是跨域响应头缺失。

## 3. 已确认的根因

### 3.1 开屏没有赞助信息：官网 supporters 路由没有 CORS

桌面端 `src/services/intro-data.ts` 直接在 WebView 中执行：

```ts
fetch('https://cs2as.600318.xyz/api/supporters')
```

官网 `E:\cs2as\src\worker.ts` 已定义 `publicCorsHeaders`，但只把它用于软件更新和站点公告等路由。当前代码：

```ts
if (route === 'GET /api/supporters') return publicSupporters(env)
```

`publicSupporters()` 最终调用没有 CORS headers 的 `json(...)`。`OPTIONS /api/supporters` 也落入普通 204 分支，没有 CORS headers。

带 `Origin: http://tauri.localhost` 的现场请求响应中没有 `Access-Control-Allow-Origin`。因此浏览器/WebView 会拦截响应，`loadIntroData()` 捕获失败并回落为空数组，画面出现“鸣谢长廊暂未连接”。**只部署当前官网源码仍然不会修复，必须先改路由。**

GitHub 公共 API 当前可访问；它不是赞助信息缺失的根因。

### 3.2 游戏停止在随机初始帧：共享 RAF 驱动不是自恢复循环

`ThreeStage.vue` 的工厂先执行一次 `renderer.render(...)`，因此每次打开都能看到由 `Math.random()` 生成的不同初始场景。随后运动完全依赖：

```ts
function render(time: number) {
  if (disposed || !controller || document.hidden || props.animate === false) return
  controller.frame(...)
  frameId = requestAnimationFrame(render)
}
```

这里有两个致命终止路径：

1. 首帧或任意一帧遇到 `document.hidden === true` 时直接 `return`，不再预约下一帧；依赖一次后续 `visibilitychange` 才可能恢复。WebView 初始化/覆盖层切换中一次瞬态隐藏就可永久停帧。
2. `controller.frame()` 或 `renderer.render()` 任意一次抛错时，下一次 RAF 尚未预约，循环永久停止；组件没有捕获、错误 UI、日志或恢复策略。

截图中游戏 HUD 的 `seconds` 已从 60 走到 50、分数为 110，而 canvas 保留初始视觉。这说明 Vue 的 `setInterval` 和指针事件仍工作，只有 WebGL 连续渲染停止，与上述机制一致。

### 3.3 游戏模拟状态与 UI 状态脱节

当前 `EasterEggGame.vue` 在倒计时、playing、finished 全阶段都挂载并持续动画 `ThreeStage`，`game-scene.ts` 不知道游戏状态：

- 倒计时 3 秒内靶标已经出生、前进并可能扣生命。
- 结算画面出现后靶标仍在背后运动和计分。
- `animate` prop 没有绑定到状态；即使绑定，`ThreeStage.vue` 也没有 watch prop，首次为 false 后不能可靠启动。
- 当前“挥动鼠标”实际上只对每个 pointer event 发一条射线，没有可见剑痕，也没有用前后指针组成斩击线段，视觉和文案不一致。

### 3.4 当前测试没有证明动画运行

现有新增测试只覆盖赞助 JSON 解析和版本点击次数。没有测试：

- RAF 连续调度超过 1 帧。
- transient hidden 后恢复。
- controller.frame 抛错后的错误上报/循环清理。
- countdown 前不模拟、finished 后停止模拟。
- retry/close 后没有遗留 RAF 和事件监听器。

## 4. 修复原则

1. 先修“每帧确实执行”的基础设施，再改美术；不能以增加模型、粒子掩盖停止的循环。
2. 动画时钟必须只有一个权威来源。游戏倒计时可以由 UI clock 驱动，但物理/靶标更新必须明确受 `playing` 控制。
3. `document.hidden` 代表暂停，不代表销毁。暂停后必须有可靠恢复入口。
4. 一次帧异常必须被捕获、记录并转成可关闭的失败状态，不能静默留下一张假装运行的静态画面。
5. 赞助数据是动态内容；3D 场景应能在网络响应到达后更新，而不是工厂创建时把空数组永久固化。
6. 官网接口不可用时仍允许进入主程序，但“完成”必须在真实部署响应带 CORS 且桌面包能读到赞助数据后确认。

## 5. 第一阶段：修复并证明共享 ThreeStage 循环

### 5.1 重写 RAF 生命周期

在 `src/components/three/ThreeStage.vue` 中引入明确状态：

```ts
let running = false
let pausedByVisibility = false
let frameId: number | null = null
let lastTime: number | null = null
```

提供以下内部函数：

```ts
function shouldRun() {
  return !disposed && controller !== null && props.animate !== false && !document.hidden
}

function ensureLoop() {
  if (running || !shouldRun()) return
  running = true
  lastTime = null
  frameId = requestAnimationFrame(onFrame)
}

function stopLoop() {
  running = false
  if (frameId !== null) cancelAnimationFrame(frameId)
  frameId = null
  lastTime = null
}
```

`onFrame` 必须使用 `try/catch/finally`。建议在进入帧时把当前 `frameId` 清空，在 `finally` 中根据 `shouldRun()` 续帧；如果 frame 抛错则停止并只 emit 一次 `error`：

```ts
function onFrame(now: number) {
  frameId = null
  if (!shouldRun()) { running = false; return }
  try {
    const delta = lastTime === null ? 0 : Math.min((now - lastTime) / 1000, 0.05)
    lastTime = now
    controller!.frame(now / 1000, delta)
  } catch (error) {
    running = false
    emit('error', normalizeFrameError(error))
    return
  }
  frameId = requestAnimationFrame(onFrame)
}
```

不能在同一时间存在两个 RAF。`visibilitychange`、`window focus` 和 animate prop watch 都只能调用幂等 `ensureLoop/stopLoop`。

### 5.2 监听 animate prop

新增：

```ts
watch(() => props.animate, (animate) => {
  if (animate === false) stopLoop()
  else ensureLoop()
})
```

可见性处理：

- `document.hidden` 时 `stopLoop()`。
- 重新可见时 `ensureLoop()`。
- 额外监听 `window.focus` 调用 `ensureLoop()`，作为 WebView2 未可靠派发 visibilitychange 时的恢复保险。
- 不要用轮询强行每 100ms 拉起 RAF。

### 5.3 帧异常与诊断

- `error` payload 至少包含阶段 `factory` 或 `frame`，内部 console/log 记录原始错误；用户 UI 显示简洁中文。
- 开发构建可在 controller 上暴露只读诊断：`frameCount`、`lastFrameAt`，生产 UI 不显示。
- `frameCount` 在 1 秒后仍小于 2 时报告 `THREE_STAGE_STALLED`，但只作为开发/测试诊断，不能在低刷新或窗口隐藏时误报。
- WebGL context lost 继续停止动画；不要自动无限创建 renderer。

### 5.4 ResizeObserver 首帧

工厂 resolve 后主动调用一次 `controller.resize(currentSize())`，不要假设 observer 必然在正确时序触发。零尺寸时不创建/更新投影，等 host 至少 `2x2` 再 ensureLoop。

## 6. 第二阶段：把游戏状态接入模拟

### 6.1 active/paused 契约

扩展 `ThreeStageController`：

```ts
setActive?(active: boolean): void
```

`ThreeStage.vue` 增加可选 prop `active?: boolean`，watch 后调用 `controller.setActive?.(props.active !== false)`；它与 `animate` 的含义不同：

- `animate` 控制是否需要渲染帧。
- `active` 控制游戏世界是否推进。

游戏在 countdown/finished 仍可低速渲染环境呼吸效果，但靶标、计分和生命只在 active 时更新。`EasterEggGame.vue` 传：

```vue
<ThreeStage :active="status === 'playing'" ... />
```

### 6.2 权威开始与停止

- 工厂创建后只生成静态场景，不生成可碰撞靶标。
- 倒计时从 3 到 0 后设置 `status='playing'`，controller 收到 active=true 后清空累计时间、生成第一个靶标并开始 60 秒。
- finish 后 active=false，冻结靶标与分数；环境允许以不超过正常 25% 速度继续轻微运动。
- 指针处理必须同时检查 `status === 'playing'` 和 controller active。
- retry 继续使用新 session/key 完整销毁旧场景，或新增明确 reset；初版保持重建更可靠。

### 6.3 使用 delta 的确定性更新

- 将 delta clamp 从 0.1 收紧到 0.05，窗口恢复后不允许靶标瞬移。
- 靶标速度用常量，例如 `TARGET_SPEED = 6.5` world units/s；当前 `10.5` 在 60fps 下太快且难观察。
- spawn 使用 accumulator/while，不能依赖绝对 `time`；active=false 时 accumulator 不增长。
- 场景初始 z、摄像机 near/far、通过摄像机时间应有单测可计算：普通靶从出生到 miss 至少 3.0 秒，给玩家反应时间。

### 6.4 真正的挥斩交互

当前单点 Raycaster 改为“屏幕线段扫掠”：

1. pointerdown 开始 stroke 并保存上一个归一化点。
2. pointermove 达到最小距离阈值后，把前后点投射到固定 gameplay plane。
3. 对每个可斩靶使用投影包围圆或 capsule/segment distance 判定；一次 stroke 对一个靶只计一次。
4. pointerup 结束 stroke。

加可见剑痕：最多 24 个 trail segment 的对象池，青白色，150-260ms 内淡出；命中产生 8-16 个 instanced sparks，禁止每次命中新建大量 material。

不可斩铜印命中时清连击并产生铜色反弹光，不扣成负分到看不懂的状态。HUD 提示保持简洁。

## 7. 第三阶段：让游戏画面真正“古风 + 科技 + 动起来”

当前截图只是巨型 wireframe Box、GridHelper 和方块靶，不能满足视觉目标。继续保持程序化原创，按以下顺序升级：

### 7.1 古风空间骨架

- 用柱、额枋、斗拱简化体、飞檐轮廓组成 3 组中式牌楼/剑阁门，不使用日式鸟居造型。
- 远景用 3-5 层低多边形山脊 silhouette，颜色从墨青到灰蓝形成空气透视。
- 两侧悬挂铜金灯笼/能量灯，点光源数量控制在 2 个，其余用 emissive mesh 假光。
- 避免当前一个巨型 wireframe box 覆盖全屏。线框只用于牌楼内的数据扫描门，opacity 更低。

### 7.2 可感知的持续运动

至少同时存在三种容易看出的运动，用于用户肉眼确认不再冻结：

1. 数据扫描门沿 Z 轴向玩家移动并循环回收。
2. 地面光纹向后流动；可用一组复用横线位移，不必引入 shader 框架。
3. 远处雾粒/纸屑缓慢漂移，低性能模式关闭。

另外：灯笼轻微摆动、靶标旋转/浮动、命中剑痕与粒子都由 delta 驱动。不要让所有对象同频摆动。

### 7.3 靶标造型

- 普通靶：青玉竹简/符印，正面有简单程序化篆纹数据线。
- 方向靶：竹简上有清晰单向光槽，只有对应斩击方向得分。
- 禁斩靶：铜制封印，轮廓与色彩同时区别，不能只靠颜色。

活动靶 `<= 16`，draw call 目标 `< 90`，DPR 上限 1.5。保持现有 Three.js MIT/原创 NOTICE，无需再引入 Slash Saber 资产。

## 8. 第四阶段：官网 CORS 修复与一次部署

在 `E:\cs2as\src\worker.ts` 中把 supporters 明确列为公开 CORS 路由。最小修改方向：

```ts
const isPublicSupporters = url.pathname === '/api/supporters'

if (request.method === 'OPTIONS'
    && (softwareUpdateMatch || updaterFeedMatch || updaterDownloadMatch || isPublicSiteNotice || isPublicSupporters)) {
  return new Response(null, { status: 204, headers: new Headers(publicCorsHeaders) })
}

if (route === 'GET /api/supporters') {
  return publicSupporters(env, new Headers(publicCorsHeaders))
}
```

或者让 `publicSupporters` 固定返回 CORS headers。不要给 admin/auth 路由加 `*`，不要把带 Cookie 的管理接口跨域开放。

补官网测试，至少断言：

- `OPTIONS /api/supporters` -> 204。
- `Access-Control-Allow-Origin: *`。
- methods 只含 `GET, HEAD, OPTIONS`。
- `GET /api/supporters` -> 200 且同样带 `*`。
- `POST /api/admin/supporters` 不出现 wildcard CORS。

运行官网 `typecheck/test/build` 后只部署一次：

```powershell
cd E:\cs2as
npm run typecheck
npm run test
npm run build
npm run deploy
```

部署后必须用带 Origin 的真实请求验证，不以 Wrangler “成功”作为完成：

```powershell
curl.exe -i -H "Origin: http://tauri.localhost" https://cs2as.600318.xyz/api/supporters
curl.exe -i -X OPTIONS -H "Origin: http://tauri.localhost" -H "Access-Control-Request-Method: GET" https://cs2as.600318.xyz/api/supporters
```

两者必须出现 `Access-Control-Allow-Origin: *`。若第一次部署失败，只排查并最多再试 2 次；总部署不超过 3 次。

## 9. 第五阶段：真实赞助信息驱动 3D 碑廊

### 9.1 保持异步，不阻塞开屏

`StartupIntro` 立即启动程序化场景，同时并发 `loadIntroData()`。数据到达后通过 scene getter 或显式 controller API更新，不重建 renderer。

推荐让工厂闭包传入只读 getter：

```ts
createIntroScene(canvas, size, () => ({
  phase: phase.value,
  supporters: supporters.value,
  upstream: data.value.upstream,
}))
```

scene 每帧只比较一个稳定 signature（支持者 id/updatedAt），只有变化时重建 CanvasTexture；不能每帧重新画文字纹理。

### 9.2 赞助者 3D 展示方式

不要继续用底部 3 列静态卡片作为主展示。改成真正的“鸣谢碑廊”：

- 每位赞助者对应一块青玉悬浮碑/竹简。
- 用 CanvasTexture 写昵称、金额和最多一行留言；中文字体使用系统 `Microsoft YaHei UI`/`Microsoft YaHei`，不联网加载字体。
- 碑沿左右两侧弧形轨道向镜头移动，当前赞助者到主展示位时减速、微旋转正对镜头，并出现铜金描边。
- DOM 层只显示当前一位的完整可访问文本，2.0-2.6 秒轮换一次；scene 中其他碑只显示昵称/光纹，避免小字不可读。
- 4 位当前真实赞助者应能在延长后的开屏内至少各出现一次；若记录更多，显示前 8 位并以“以及 N 位同路人”收束。
- 无数据时保留通用感谢碑，但它与“网络加载中”状态必须区分。前 1.8 秒显示“正在连接鸣谢长廊”，请求确认失败后才显示“暂未连接”。

### 9.3 动画时间线调整

当前总时长 6.4 秒对 4 位赞助者过短。建议：

- `0-0.7s` 场景点亮。
- `0.7-5.7s` 鸣谢碑廊，至少展示当前 4 位；根据数量动态延长，但总开屏不超过 9.5 秒。
- `5.7-7.4s` 上游项目数据碑。
- `7.4-8.0s` 剑光收束进入主界面。

始终保留“跳过”。若用户启用 reduced motion，展示静态当前赞助名单和上游信息，最长 1.5 秒，不执行镜头飞行。

### 9.4 缓存行为

- CORS 修复后，首次成功数据写入现有缓存。
- 后续断网允许使用 7 天旧缓存，并清楚标记 `source='cache'`，但用户界面无需显示技术词。
- 不把这 4 位当前赞助者硬编码进客户端，否则未来删除/隐藏后仍可能展示。
- `loadIntroData()` 需要新增“network empty array 也是成功”测试，不能错误回退已删除的旧名单。

## 10. 自动化验证方案

### 10.1 ThreeStage 单元测试

新增 `tests/three-stage-loop.spec.ts`，使用 Vue Test Utils 和可控 RAF 队列，不依赖真实 WebGL：

1. factory resolve 后预约 1 个 RAF。
2. 手动执行 3 个 callback，controller.frame 恰好执行 3 次且每次仅预约一个后继。
3. `document.hidden=true` 后停止；模拟 visibilitychange/focus 并恢复可见后重新预约。
4. animate 从 false 改 true 会启动，从 true 改 false 会取消。
5. frame 第 2 次抛错时 emit 一次 error、不再预约、dispose 仍在 unmount 调用。
6. unmount 后所有 RAF、ResizeObserver、visibility/focus/contextlost 监听被清理。

不要只对源码字符串做断言。

### 10.2 游戏场景确定性测试

把随机源和 clock 注入 `createGameScene` 或抽出纯状态 `GameSimulation`，新增：

- active=false 连续调用 frame，靶标数量、z、生命、分数不变。
- active=true 后固定 delta 推进，z 按速度变化且至少 2 个不同时间点不同。
- 3 秒内普通靶不会从出生直接 miss。
- finish/active=false 后不再扣生命。
- pointer line segment 穿过青玉靶计分一次，重复 move 不重复计分。
- 铜印不增加分数并清连击。
- retry/reset 清空 targets、score、combo、lives 恢复 3。

### 10.3 Intro 数据与场景测试

- supporters 网络成功后 source=network、缓存写入、scene getter signature 变化。
- GET 被 CORS/网络拒绝时 fallback 不阻塞开屏。
- 真实 4 条数据生成 4 块碑，昵称/金额映射正确。
- 文本超长被截断且 Canvas 不溢出；空昵称/留言使用官网同款默认文案。
- 动画两个不同时刻的 camera/tablet transform 不相同。

### 10.4 官网测试

官网测试和线上 header 回读缺一不可。生产回读应保存到新的 `E:\cs2as\release-evidence\supporters-cors-<timestamp>\`，包含 GET/OPTIONS headers 和脱敏后的响应 shape；赞助昵称是公开展示数据，但证据不需要重复保存完整留言。

## 11. 构建和实际验收

桌面程序执行简化验证：

```powershell
cd E:\CS2AS05
npm run typecheck
npm run lint
npm run test
npm run build:web
```

最终真实 Windows 验收交由用户，但实际执行 AI 应给出明确步骤：

### 开屏

1. 清除 `cs2as:intro:*` localStorage 或使用干净安装启动。
2. 在线启动，必须看到官网当前赞助者逐位进入 3D 主展示位，不再显示“暂未连接”。
3. 观察至少 5 秒：碑、镜头、粒子或扫描门持续运动，不能只依靠 DOM 进度条证明动画。
4. 断网再启动，应展示缓存并能跳过；清空缓存断网则显示通用感谢，不白屏。
5. 上游阶段正常展示，标题栏关闭/最小化始终可用。

### 彩蛋游戏

1. 点击版本 5 次进入，倒计时期间没有靶标穿过玩家或扣生命。
2. 倒计时结束后不用操作，连续录屏/观察 10 秒：至少两个靶标从远处明显靠近，地面光纹与扫描门持续移动，HUD 时间同步减少。
3. 鼠标划过青色靶，必须出现剑痕和命中特效并计分；点一下而没有形成有效 stroke 不应误判为挥斩。
4. 结算后观察 3 秒，分数和生命不再变化；重试后重新开始。
5. Alt-Tab 5 秒再回来，动画继续且靶标不瞬移、不一次扣多条生命。
6. 连续打开/关闭 3 次，画面每次都运动，内存/GPU 不持续上涨。

建议用户提供 10-15 秒录屏，而不是两张截图，因为本问题的完成标准是连续运动。

## 12. 实施顺序与停止条件

严格按以下顺序：

1. 记录两个工作区 `git status`，保留现有改动。
2. 重写 ThreeStage RAF 生命周期并补循环测试。
3. 接入 game active 状态，先用当前方块证明连续运动、暂停和恢复。
4. 修官网 supporters CORS，跑测试并部署/回读。
5. 让 intro scene 接收真实动态数据，完成 3D 碑廊。
6. 最后升级古风游戏美术、剑痕和命中特效。
7. 跑桌面验证并让用户做真实 WebView2 录屏验收。

停止条件：

- 若 RAF 测试通过但用户机器仍冻结，必须先取得 WebView2 console 中的 frame error、`document.visibilityState`、frameCount 和 context lost 信息，不继续盲改美术。
- 若线上接口 200 但没有 CORS header，官网修复尚未完成；不能以本机 PowerShell 能读取 JSON 作为桌面成功证据。
- 若 CORS 正确而桌面仍为空，检查实际 Tauri origin、fetch error 和缓存解析，不硬编码赞助者绕过。
- 若任何许可证不明的外部资产被引入，停止构建并移除；本方案不需要外部资产。

## 13. 完成标准

只有以下全部成立才可报告完成：

- 生产 `GET` 和 `OPTIONS /api/supporters` 对桌面 origin 返回正确 wildcard CORS，管理接口未被跨域开放。
- 干净在线桌面启动能看到真实官网赞助数据驱动的 3D 运动展示。
- 游戏连续运行至少 60 秒或完整一局，无静态初始帧；Alt-Tab 后能恢复。
- countdown、playing、finished 三个状态与模拟严格一致。
- 有自动化测试证明至少 3 个连续 RAF、隐藏/恢复、帧异常、active pause 和确定性靶标位移。
- 关闭/重试没有重复 RAF、监听器、renderer 或资源泄漏。
- 应用、Cargo、Tauri、插件 marker 仍全部是 `0.5.5`，已有 0.5.5 功能无回归。
- 用户用真实 Windows WebView2 完成连续运动录屏验收；仅静态截图不能证明本次冻结问题已修复。

本轮仅制定方案并新增本文档，不修改运行代码、不部署官网、不构建安装包。
