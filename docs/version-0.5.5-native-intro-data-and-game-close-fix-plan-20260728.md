# 0.5.5 原生开屏数据与游戏关闭按钮修复交接方案

## 1. 任务与边界

本文交给下一位“实际执行 AI”。项目在 Windows 开发，主工作区为 `E:\CS2AS05`，官网 Worker 工作区为 `E:\cs2as`。当前产品版本必须保持 `0.5.5`；不要回退现有开屏、Three.js、CORS 或游戏循环修复。

本轮只解决两个用户确认的问题：

1. 开屏截图 `C:\Users\GOPtZ\Pictures\Screenshots\屏幕截图 2026-07-28 133744.png` 仍显示默认的“青锋无名客”和“鸣谢长廊暂未连接”，没有真实赞助信息。必须稳定显示官网公开鸣谢数据。
2. 彩蛋游戏进入 `playing` 状态后，右上角 `X` 无法关闭，只有 `Escape` 有效。`X` 必须在倒计时、游戏中、结算、失败四种状态均可立即关闭。

不改赞助数据、不硬编码当前赞助者、不新增账号/遥测/排行、不把窗口关闭按钮或 `Escape` 行为改成其他逻辑。此处要求“修复”，不是再写视觉方案。

## 2. 现场证据

### 2.1 开屏实际状态

截图时间为 2026-07-28 13:37:44，画面已包含新版程序化牌楼、剑和中央碑位，因此并非旧版开屏；但文案为：

```text
鸣谢长廊
幸与诸君同路
鸣谢长廊暂未连接
感谢每一位同路人
```

这证明 `StartupIntro.vue` 已经走完 `loadIntroData()` 并收到了 `sources.supporters === 'fallback'`，不是单纯截图抓得过早的“正在连接”状态。

截至 2026-07-28 14:07 的线上回读：

```text
GET https://cs2as.600318.xyz/api/supporters -> 200
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, HEAD, OPTIONS

OPTIONS https://cs2as.600318.xyz/api/supporters -> 204
Access-Control-Allow-Origin: *
```

响应确有 4 条公开记录。`E:\cs2as\src\worker.ts` 的 CORS 修改仍是未提交工作树改动，但线上已经具有正确响应头。此前“接口没有部署/CORS 缺失”的问题已被修正，不能继续把它当成唯一根因。

### 2.2 桌面端当前薄弱点

`src/services/intro-data.ts` 仍直接在 Vue WebView 内调用：

```ts
fetch('https://cs2as.600318.xyz/api/supporters')
fetch('https://api.github.com/repos/ed0ard/CS2-Bot-Improver')
```

即使服务器允许 CORS，桌面客户端仍会受 WebView2 的代理、TLS、企业安全软件、初始化网络、DNS 或 CORS 实现差异影响。用户截图已经证明该链路在真实 Windows 程序中失败。按照本项目“不要依赖浏览器自带操作实现功能”的约束，生产桌面端不应把 WebView `fetch` 作为唯一官方数据链路。

当前 `loadIntroData()` 还把两端请求一次性 `Promise.allSettled`，然后无区分地将 supporters 失败降为 fallback；没有向 Tauri 日志输出机器可诊断的失败码，无法判定是 invoke/CORS/TLS/超时/数据格式哪一种。

### 2.3 关闭按钮根因

`src/components/easter-egg/EasterEggGame.vue` 当前把 Pointer Events 绑定在整个游戏 `<section>`：

```vue
<section
  @pointerdown="pointer($event, 'pointer-down')"
  @pointermove="pointer($event, 'pointer-move')"
  @pointerup="pointer($event, 'pointer-up')"
>
```

在 `playing` 时，`pointer()` 对 section 调用：

```ts
target.setPointerCapture(event.pointerId)
```

这里的 `event.currentTarget` 是整个 section，不是 canvas。Pointer capture 会把后续 pointerup/click 路径重定向到 section；右上角按钮的 `@click.stop="close"` 因此可能根本不收到 click。倒计时阶段 `pointer()` 提前 return，没有 capture，所以同一个 `X` 能关闭。这与用户描述完全一致。

按钮 CSS `z-index: 4` 只能改变视觉覆盖顺序，不能解除父元素已建立的 pointer capture；仅提高 z-index 或再加一个 click handler 不是可靠修复。

## 3. 总体方案

### 3.1 数据链路原则

生产 Tauri：固定白名单 HTTPS URL 的 Rust 原生请求 -> Tauri `invoke` -> Vue 验证/缓存/渲染。

浏览器开发模式：保留现有 `fetch` 作为开发预览 fallback，官网 CORS 继续保持。任何一端失败都不阻塞开屏，但错误必须可诊断。

### 3.2 输入层原则

游戏动作只在一个专用、无按钮的“输入层”接收；关闭按钮、HUD、结算按钮处于更高层级，永不在 pointer capture 的祖先路径中。关闭动作在 DOM pointerdown 和 click 两层都明确处理，保证鼠标与触摸行为。

## 4. 第一阶段：提交并保留官网 CORS 修复

`E:\cs2as\src\worker.ts` 当前未提交的正确修改必须作为官网变更的一部分保留：

- `GET /api/supporters` 返回 `Access-Control-Allow-Origin: *` 等公开 CORS headers。
- `OPTIONS /api/supporters` 返回 204 和同样 headers。
- admin/auth/supporter 管理接口不增加 `*`。

执行 AI 先确认这段现有工作树改动，补官网测试后将它一起提交；不要因为本轮改成 native fetch 而撤销它。理由：官网首页、浏览器开发模式、未来非 Tauri 客户端仍需要公开只读 endpoint 的 CORS。

官网回读命令：

```powershell
curl.exe -i -H "Origin: http://tauri.localhost" https://cs2as.600318.xyz/api/supporters
curl.exe -i -X OPTIONS -H "Origin: http://tauri.localhost" -H "Access-Control-Request-Method: GET" https://cs2as.600318.xyz/api/supporters
```

两者均必须有 `Access-Control-Allow-Origin: *`。如果线上已有正确回读，不要为了本项重复部署；只有 Worker 源码实际再修改时才按既有限制部署，累计不超过 3 次。

## 5. 第二阶段：桌面原生 intro 数据命令

### 5.1 Rust 依赖和安全边界

在 `src-tauri/Cargo.toml` 新增最小 `reqwest`，建议：

```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

保持 Rust `1.77.2` 兼容；加入前先核对该 reqwest 版本/MSRV，若不兼容选择最后一个兼容 `0.12.x` 版本。不要启用 native-tls，不让系统 OpenSSL 安装状态影响 Windows 包。

请求 URL 不由前端传入，Rust 常量固定为：

```rust
const SUPPORTERS_URL: &str = "https://cs2as.600318.xyz/api/supporters";
const UPSTREAM_URL: &str = "https://api.github.com/repos/ed0ard/CS2-Bot-Improver";
```

只允许 HTTPS、禁止重定向到任意 host（`redirect(Policy::none())` 或每次重定向重新做 host allowlist 校验）。不接受 Cookie、不存 Token、不接收任意 URL 参数。GitHub 请求必须发送稳定的 `User-Agent: CS2AS05/0.5.5` 和 `Accept: application/vnd.github+json`。

### 5.2 推荐文件划分

```text
src-tauri/src/
  commands/intro.rs       # #[tauri::command] async get_intro_public_data
  services/intro.rs       # 固定 endpoint、HTTP、反序列化、长度/数量校验
  commands/mod.rs         # pub mod intro
  services/mod.rs         # pub mod intro
  lib.rs                  # 只注册该命令
src/services/tauri/intro.ts # invoke 的前端薄封装
src/services/intro-data.ts  # 根据 isTauri 选择 native 或 browser-dev provider
```

不要把 HTTP 代码塞进已有 `support.rs`，该文件职责是白名单外部浏览器启动；intro 是读取公开数据的独立服务。

### 5.3 IPC 契约

Rust 和 TypeScript 采用与现有 `IntroData` 对齐的蛇形/驼峰明确契约，推荐 Rust 用 `#[serde(rename_all = "camelCase")]`：

```ts
type IntroPublicPayload = {
  supporters: Array<{
    id: string
    nickname: string | null
    message: string | null
    amountCents: number
    sortOrder: number
    isVisible: boolean
    createdAt: string
    updatedAt: string
  }>
  upstream: {
    fullName: string
    description: string
    url: string
    license: string
    stars: number | null
    forks: number | null
    pushedAt: string | null
  }
  sources: { supporters: 'network' | 'fallback'; upstream: 'network' | 'fallback' }
  diagnostics: {
    supporters: 'ok' | 'timeout' | 'network' | 'http' | 'invalid'
    upstream: 'ok' | 'timeout' | 'network' | 'http' | 'invalid'
  }
}
```

`diagnostics` 只供日志、测试和开发诊断，不在普通开屏对用户展示技术错误。对用户仍显示“鸣谢长廊暂未连接”。

每个 endpoint 独立处理：

- supporters 成功时返回最多 50 条通过校验的记录，按服务端顺序，不在客户端重排。
- supporters 失败时返回空数组和原因码；不要使整个 invoke reject，因为上游信息仍可能可用。
- upstream 同理，失败返回同项目已有的静态 fallback。
- 总 connect + request timeout 建议 2.2 秒，单请求 body 上限 128 KiB；解析前检查 `Content-Type` 和状态码。
- Rust 仍要校验长度、非空 id、非负 `amountCents`、最大 nickname/message 长度；前端也保留现有 `parseSupporters/parseUpstream`，形成不信任边界。

Rust command 应为 async，并用 `tauri::async_runtime`/async reqwest，不在 UI 线程同步阻塞。

### 5.4 命令注册与权限

1. `src-tauri/src/commands/mod.rs` 加 `pub mod intro;`。
2. `src-tauri/src/services/mod.rs` 加 `pub mod intro;`。
3. 在 `src-tauri/src/lib.rs` 的 `tauri::generate_handler![]` 注册 `commands::intro::get_intro_public_data`。
4. Tauri 2 command 不需要为自身 invoke 增加额外 capability；仍核对 `src-tauri/capabilities/default.json` 不开放 shell、任意 HTTP 或任意文件权限。
5. 为 Rust service 写本地 HTTP/解析单测。建议把 response parse 和 endpoint-client 注入，使用 mock server 或纯 fixture，不能让单测调用真实官网/GitHub。

### 5.5 前端 provider 选择

在 `src/services/tauri/intro.ts`：

```ts
export function getIntroPublicData() {
  return invoke<IntroPublicPayload>('get_intro_public_data')
}
```

`src/services/intro-data.ts` 改为：

```ts
if (isTauri()) {
  // native is primary: validate returned payload, write cache, retain diagnostics in log
} else {
  // browser dev/test: existing fetchJson path
}
```

必须使用 `isTauri()`，不是 `window.__TAURI__` 字符串猜测。浏览器模式下仍允许 CORS fetch，避免 `npm run dev:web` 失去预览数据。

native command 异常（例如开发环境没有注册 command）不应让应用崩溃：

- 在确实是 Tauri 的生产环境，记录 `INTRO_NATIVE_INVOKE_FAILED`，先读本地缓存，再给出通用 fallback；**不要立即改回 WebView fetch 作为生产主路径**。
- 在自动化/jsdom 或浏览器模式，走 browser provider。
- 开发构建可再允许 native invoke 失败后 browser fetch 便于调试，但生产 release 不将其作为静默依赖。

日志应包括不会泄露内容的 `source`, `reason code`, `HTTP status`, `elapsedMs`, `count`；不得把完整赞助留言和 GitHub body 打进日志。

### 5.6 开屏渲染时序

`StartupIntro.vue` 现有 8-9.5 秒时长和 data getter 能在数据到达后更新碑位。保留此能力，但做两项调整：

1. 原生 provider 返回 supporters 后，`data.value` 的替换必须立刻触发 `currentSupporter` 和 `createIntroScene` state getter；不要通过 `key` 重建 `ThreeStage`。
2. 在 `loaded=false` 时只显示“正在连接鸣谢长廊”；只有 native/browser provider 确认失败且无缓存时显示“暂未连接”。成功空数组显示“鸣谢长廊静候同路人”，三者不可混淆。

截图中的“青锋无名客”只能出现在一条真实记录 `nickname=null` 时，不能作为空数组或请求失败的假占位。当前官网确有一条匿名记录，但如果请求失败则 DOM 不应擅自展示它。

## 6. 第三阶段：拆分游戏输入层，修复 X

### 6.1 模板改造

从 `<section>` 删除所有 `@pointer*`。在 `ThreeStage` 后、HUD/按钮前创建专用输入层：

```vue
<ThreeStage ... />
<div
  ref="inputLayer"
  class="game-input-layer"
  aria-hidden="true"
  @pointerdown="pointer($event, 'pointer-down')"
  @pointermove="pointer($event, 'pointer-move')"
  @pointerup="pointer($event, 'pointer-up')"
  @pointercancel="pointer($event, 'pointer-up')"
/>

<header class="game-hud">...</header>
<button
  ref="closeButton"
  type="button"
  class="cinema-icon-button"
  title="关闭"
  aria-label="关闭青冥试剑"
  @pointerdown.stop="close"
  @pointerup.stop
  @pointercancel.stop
  @click.stop="close"
>
  <X :size="20" />
</button>
```

`@pointerdown.stop="close"` 是鼠标/触摸的立即路径，`@click.stop="close"` 是键盘/辅助技术的语义路径。`close()` 必须幂等，因为一次手势可能触发 pointerdown 后又触发 click；加：

```ts
let closing = false
function close() {
  if (closing) return
  closing = true
  clearClock()
  releaseActivePointerCapture()
  emit('close')
}
```

不在 close handler 上使用 `.prevent`，以免破坏按钮焦点/键盘激活语义。

### 6.2 CSS 分层

```css
.game-overlay { cursor: default; }
.three-stage { z-index: 0; }
.game-input-layer {
  position: absolute;
  z-index: 1;
  inset: 0;
  cursor: crosshair;
  touch-action: none;
}
.game-hud, .game-credit { z-index: 2; }
.game-center { z-index: 3; }
.cinema-icon-button { z-index: 4; }
```

`game-input-layer` 是唯一使用 `touch-action:none` 的区域；不要让整张 dialog 抑制所有控制按钮的默认输入。HUD 继续 `pointer-events:none`，结算容器 `pointer-events:auto`，关闭按钮始终可点击。

### 6.3 Pointer capture 生命周期

`pointer()` 只在 `status==='playing'` 且 `event.currentTarget === inputLayer` 时 capture。追踪：

```ts
let activePointerId: number | null = null
```

- pointerdown：capture 并记录 id。
- pointerup/pointercancel：只释放同一个 id。
- `close()`、`finish()`、`onBeforeUnmount()`：若 input layer 仍持有 capture，安全释放并清空 id。
- 多指输入：初版只接受第一个 active pointer，其余 pointer 忽略；不可让第二根手指覆盖第一根 stroke。

`close()` 调用后必须先停止时钟、解除 capture，再 emit。这样 Vue 卸载不受指针状态影响。

### 6.4 不可退化的行为

- 倒计时：X 可关闭；点击/移动输入层不计分。
- playing：X 可关闭；在输入层挥剑仍有效；点击 X 不会误挥剑或减少生命。
- finished/failed：X 可关闭；“再试一局”和“关闭”可正常工作。
- 键盘 Escape、Tab focus trap、关闭后焦点回到版本按钮保持现状。

## 7. 测试清单

### 7.1 桌面数据测试

新增/扩展 `tests/intro-data.spec.ts` 和 Rust 单测：

- Tauri provider 成功返回 4 条 fixture 时，`loadIntroData()` source=network，缓存写入，`currentSupporter` 不是 fallback。
- Tauri supporters failure + 有 7 天内缓存：显示缓存；无缓存：空数组/`fallback`。
- 成功空数组覆盖旧缓存，不展示旧赞助者。
- provider 返回 malformed/超长/负金额数据时过滤，不崩溃。
- browser mode 仍走 mocked `fetch`，Tauri mode mock `invoke`，两条路径均测试。
- native command 支持 supporters 成功/upstream 失败以及反向情况；每个 reason code 正确。
- source 边界：前端接到 native payload 后仍调用 parser，不能直接信任 invoke。

增加 `StartupIntro` 组件测试：以 deferred promise 控制 native payload，在 mount 后 resolve 4 条记录，flush 后断言 DOM 出现例如 fixture 昵称和金额、不再有“暂未连接”；不需要真实 WebGL，stub `ThreeStage`。

### 7.2 游戏关闭组件测试

新增 `tests/easter-egg-close.spec.ts`，stub `ThreeStage`，使用 fake timers：

1. mount 后 countdown 状态，对 close 按钮 `pointerdown`，只 emit 一次 `close`。
2. 计时推进到 playing 后，在 input layer 触发 pointerdown 以建立 capture，再对 close 按钮触发 pointerdown/click；仍只 emit 一次 `close`。
3. close 前 interval 被清理，后续 timer 不改变 seconds。
4. close 后 pointer capture 被释放，unmount 无 throw。
5. input layer pointerdown 不会冒泡给 close，close button pointerdown 不会调用 `stage.interact`。
6. Escape 在 playing 仍只 emit 一次 close。

此测试必须模拟 pointer capture，不能只检查 `.stop` 字符串。

### 7.3 浏览器/官网回读

- 官网 Worker 保留 GET/OPTIONS CORS 单测，admin 路由无 wildcard CORS。
- 本地 `npm run dev:web` 下用实际 public endpoint 验证 fetch 成功，仅作为浏览器开发兼容性；桌面验收必须验证 `invoke` 路径。

## 8. 验证和发布顺序

### 8.1 本地验证

桌面项目：

```powershell
cd E:\CS2AS05
npm run typecheck
npm run lint
npm run test
npm run build:web
cargo test --manifest-path src-tauri/Cargo.toml
```

官网仅在 Worker 有新增测试/修改时：

```powershell
cd E:\cs2as
npm run typecheck
npm run test
npm run build
```

### 8.2 Windows 真实验收

需要重新启动由最新桌面代码构建的 Tauri 程序；不能拿 13:29 的旧 `dist` 或旧安装包证明 native command 修复。

1. 在线、清空 `cs2as:intro:*` 缓存启动：开屏在 2.2 秒内展示当前官网的真实昵称/留言/金额，至少切换两位，且场景碑位跟随更新。
2. 临时断网启动：有缓存时显示最后成功数据；清缓存后显示通用“暂未连接”，不伪造具体赞助者。
3. 将网络恢复后重启，失败状态不应永久粘住，真实记录重新出现。
4. 进入彩蛋，在 countdown、playing、finished、failed 各点击右上角 X；每一次均立即回主界面，版本按钮获得焦点。
5. playing 时点击 X 后不得留下时间倒计时、音效、RAF、pointer capture 或响应主界面的意外游戏输入。
6. 记录简短视频，至少覆盖“playing 点击 X 退出”和“真实赞助者碑位”，静态截图不足以验证交互/数据链路。

## 9. 完成标准

全部满足才可报告完成：

- 官方网站 `/api/supporters` 的 CORS 修复保留、测试通过、线上回读正确。
- 正式 Windows Tauri 使用 native command 获取公开数据；真实赞助者可出现在开屏，不依赖 WebView fetch 的偶然成功。
- 网络失败可缓存/降级，且在日志中有不含隐私内容的原因码。
- 关闭按钮不再在 `playing` 捕获路径内，四个状态和鼠标/触摸/键盘均可关闭，且没有重复 emit。
- 针对 native provider 和 playing 状态关闭的自动化测试通过。
- 所有应用版本及 plugin marker 仍是 `0.5.5`。

本轮方案制定仅新增本文档，不修改运行代码、不重新构建或发布。
