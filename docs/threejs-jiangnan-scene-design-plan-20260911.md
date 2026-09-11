# 0.5.13 Three.js「水墨江南·幸与诸君同路」开屏与彩蛋重制设计方案

日期：2026-09-11  
项目：`E:\CS2AS05`  
版本：继续使用 `0.5.13`，不增加版本号、不新建独立页面  
参考：[水墨杭州](https://gamemcu.com/hz/)（仅参考空间气质，不复制其代码、素材或品牌）

> 本文是本次重新制作的设计与实施方案。当前工作区已经有旧版 `createIntroScene` / `createContributionGalleryScene`，本方案将它们收敛为同一个可中断 Three.js 场景，保留已有脏改动，不回退无关内容。

## 1. 最终体验

### 1.1 一个场景，两种状态

- **开屏状态**：应用启动时仅在用户明确开启后播放；镜头沿水面长廊依次掠过所有牌子，顺序为「远景建立 → 上游 → 赞助 → 鸣谢 → 总收束」，结束后回到助手。
- **彩蛋状态**：由现有版本彩蛋入口进入同一场景；不重新创建另一套世界，用户可以点击底部牌子、键盘方向键或触控选择任意牌子，镜头平滑前往目标。
- **共用状态**：牌子数据、场景材质、相机轨迹、焦点高亮、无障碍文本和降级视图全部复用。

推荐标题：`幸与诸君同路`  
副标题：`上游项目、赞助与公开鸣谢，共同点亮这一程。`

### 1.2 水墨江南空间构图

- 上方：宣纸留白、淡墨远山、低密度雾层和一轮暖日；不做高饱和霓虹。
- 中景：一条曲折水巷与拱桥，牌子沿岸/桥边分组布置，形成镜头可读的纵深。
- 近景：水面细波、浮叶、少量落花粒子；粒子只做呼吸感，不喧宾夺主。
- 牌子：木框 + 宣纸面 + 小型来源印章。上游使用青黛边线，赞助使用暖金，鸣谢使用梅红/朱砂；颜色表达来源，不表达金额排名。
- 中央收束牌：`感谢每一位同行者`，仅作为结尾视觉锚点，不显示排行或累计金额。

参考站点当前可见的核心气质是“米白留白、淡墨远山、灰蓝水面、竖向书法牌、低对比漂浮层”；实现采用自有程序化几何与 CanvasTexture，避免引入未授权素材。

## 2. 牌子信息架构

统一类型 `AcknowledgementPlaque`：

```ts
interface AcknowledgementPlaque {
  id: string
  kind: 'upstream' | 'sponsor' | 'acknowledgement'
  eyebrow: string
  title: string
  message: string
  detail: string
  url?: string
  unit?: 'cny' | 'beike' | 'unknown'
  visibleAmount?: number
  amountScope?: 'visible_record' | 'reported_total' | 'unknown'
  sourceLabel?: string
  updatedAt?: string
}
```

- 上游牌：项目名、作用、许可证、仓库入口。
- 赞助牌：公开昵称、留言、可核实记录；不出现“贡献排行”“最高支持”。
- 鸣谢牌：例如 `B站充电鸣谢 / SoyO_official / 给你充电啦！ / 后台可见 20.16 贝壳 · 暂定约 ¥20.16`。
- B站固定范围说明：`数量为我们当前后台可见记录，不代表平台上的完整充电总额。`
- 旧 `amountCents` 记录继续兼容；B站不可写成 `amountCents: 2016`，必须保留 `unit=beike` 与 `amountScope=visible_record`。

## 3. 相机与动画状态机

不使用不可中断的长 CSS keyframe；Three.js 每帧通过状态机插值：

```text
intro:establish → intro:plaque[i] → intro:transition → intro:plaque[i+1]
                         ↓ 任意用户操作
                    gallery:free-roam
                         ↓ Escape/关闭
                       closed
```

- 每张牌：进入 550ms、阅读 1.6–2.2s（按文字长度上限调整）、离开 400ms。
- 总时长设置上限 18s；牌子较多时降低单牌停留而不是无限延长。
- 使用 `easeOutCubic` 进入、`easeInOutCubic` 巡游；禁止 `scale(0)` 弹出。
- 开屏控件：`跳过开屏`、`暂停/继续`、当前牌子、进度条；点击场景、方向键、鼠标拖拽均立即转自由浏览。
- 彩蛋焦点：点击牌子或列表后 150–250ms 前往；不等待下一段自动时间线。
- `prefers-reduced-motion`：不自动巡游、不旋转粒子，仅保留可点击牌子与淡入淡出。

## 4. 实施文件边界

### 4.1 新增/收敛

- `src/features/acknowledgement/scene.ts`：唯一场景工厂，输出 `createAcknowledgementScene(mode, getState)`。
- `src/features/acknowledgement/types.ts`：牌子、场景模式、焦点和数据来源合同。
- `src/features/acknowledgement/camera.ts`：自动巡游、自由浏览、目标牌子插值和 reduced-motion。
- `src/features/acknowledgement/fallback.ts`：WebGL 失败、低性能和无 WebGL 的 DOM 牌廊降级。

### 4.2 替换调用

- `src/components/intro/StartupIntro.vue`：只负责开屏生命周期、按钮、焦点和 `mode='intro'`。
- `src/components/easter-egg/EasterEggGame.vue`：只负责彩蛋生命周期、牌子导航和 `mode='gallery'`。
- `src/features/intro/scene.ts`、`src/features/easter-egg/game-scene.ts`：迁移到新工厂后删除重复相机/展牌构建代码。
- `src/components/three/ThreeStage.vue`：保留现有 ResizeObserver、context lost、低性能帧率保护；增加 `reducedMotion` 与 fallback 事件。
- `src/styles/main.css`：新增水墨牌廊层、响应式导航、焦点环和静态降级样式，不改变其他页面主题契约。

## 5. 开屏默认与用户覆盖

目标规则：`用户明确设置 > 0.5.13 默认值(false)`。

- 新用户、首次安装、升级安装：开屏默认为关闭。
- 已存在 `skipIntro` 的用户：保留其明确选择，不能被本版本覆盖。
- 设置页提供“启动时播放水墨鸣谢”开关；旁边提供一次性“重新观看本次开屏”，重播不改变长期设置。
- 桌面业务设置不能新增 `localStorage` 依赖；沿用现有 Tauri 原生偏好存储迁移。Web 预览仅可使用临时内存状态。
- 默认关闭时不加载远程赞助数据；用户主动进入彩蛋时才触发一次数据准备。

## 6. 数据读取与缓存闸门

- 恢复 `loadIntroData()` 的公开线上读取，仅读公开接口，不携带管理员 Cookie、Token 或平台凭据。
- 进程内 Promise 去重：一次进程最多一次条件刷新；重播和切换牌子只复用内存数据。
- 原生应用数据目录缓存 24h；缓存过期才后台刷新，失败则使用静态上游牌和通用鸣谢牌，不伪造具体赞助者。
- 服务端 `/api/supporters` 增加边缘缓存、ETag、5–15 分钟 TTL，避免每次开屏直接命中 D1。
- 公开 payload 必须携带 `unit`、`amountScope`、`sourceLabel` 与 `scopeNotice`，前端不自行猜测汇率。

## 7. 响应式与可访问性

- 375px：底部横向牌子选择器，按钮至少 44×44px；隐藏非关键装饰，保留标题、当前牌子和返回。
- 768px：牌子选择器可折叠；镜头区域至少占 60vh。
- 1024/1440px：右侧信息卡 + 底部牌子索引，保证当前牌子始终可读。
- 牌子同步 DOM 标题/说明；`role=dialog`、`aria-modal`、`aria-current`、Esc 返回、方向键切换、焦点回收完整可用。
- 颜色对比度正文≥4.5:1；不以颜色作为唯一类别信息；不使用 Emoji 作为结构图标。

## 8. 验证顺序与证据

1. `npm run typecheck`、定向 lint、Three.js 场景单元测试、`npm run build:web`。
2. 牌子合同测试：总数、稳定顺序、B站单位/范围说明、旧金额兼容。
3. 网络闸门测试：默认关闭无请求；彩蛋一次请求；重播不重复；缓存命中/断网降级。
4. 浏览器/预览验收：375、768、1024、1440px；亮/暗主题；WebGL、低性能、context lost、reduced-motion。
5. Windows/Tauri 验收：首次启动默认不弹、打开设置后可播放、跳过/暂停/返回、彩蛋可选牌子；真实玩家设备最终确认镜头与牌面文案。
6. 发布证据写入 `docs/` 与 `artifacts/`，明确本地构建不替代真实设备验收；不覆盖已发布的 0.5.13 安装器，除非全部门槛通过。

## 9. 停止条件

- 开屏默认仍弹出或覆盖了用户原有选择；
- 彩蛋和开屏出现两套不一致的场景/牌子顺序；
- 默认关闭仍请求线上接口，或每次进入牌子都查询 D1；
- B站贝壳被渲染为人民币累计赞助；
- WebGL 失败白屏、不可跳过、Esc 不返回或 reduced-motion 仍自动运动；
- 参考站点素材/代码授权边界不清；
- 真实 Windows/玩家设备验收未完成。

## 10. 推荐文案

- 总标题：`幸与诸君同路`
- 副标题：`上游项目、赞助与公开鸣谢，共同点亮这一程。`
- 开屏动作：`跳过开屏` / `暂停镜头` / `继续浏览`
- 彩蛋入口：`浏览鸣谢长廊`
- B站牌：`B站可见 20.16 贝壳 · 暂按 1 贝壳 = ¥1 展示`
- 口径说明：`数量为我们当前后台可见记录，不代表该用户在平台上的完整充电总额。`
