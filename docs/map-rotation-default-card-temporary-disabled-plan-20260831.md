# 自动换图默认状态卡片临时下线与主题设置入口交接方案

日期：2026-08-31  
工作区：`E:\CS2AS05`  
交接对象：实际执行 AI  
状态：调查完成，待执行；本轮只制定 UI 交接方案，不代表插件问题已修复。

## 1. 用户最新现场与边界

助手程序已经确认外部配置回读 `enabled=0`，但游戏内 `lbtv_map_rotation` 仍返回 `enabled=1`，并且实际发生自动换图。由此可知：

- 当前助手侧“自动换图默认状态”读写链路不能作为游戏运行时状态的证明；
- 插件侧问题仍在单独的 `docs/map-rotation-plugin-enabled-config-fix-plan-20260831.md` 中跟进，本方案不修改或掩盖该问题；
- 自动换图延迟已经足够玩家查看最终结算页面，本轮不新增延迟、不改换图调度、不改 JSON 写入、不改命令语义；
- 为避免玩家误以为 `enabled=0` 已经在游戏中生效，概览页暂时不展示“自动换图默认状态”卡片；
- 该功能不是删除，而是从主界面临时隐藏，并在“安装与诊断 → 主题设置”中提供一个明确标注“开发中、暂时无效”的查看入口。

## 2. 已调查代码事实

- 卡片组件：`src/components/MapRotationDefaultControl.vue`，包含真实 IPC 读取/写入/重置操作和“下一次插件载入生效”文案。
- 唯一主界面入口：`src/views/OverviewView.vue` 的 `control-grid`，当前直接渲染 `<MapRotationDefaultControl ... />`。
- 主题设置抽屉：`src/components/AppearanceSettingsDrawer.vue`，由 `src/views/InstallView.vue` 通过 `appearanceOpen` 打开；当前抽屉包含主题、颜色、圆角、布局、启动体验和侧边栏设置。
- 主题抽屉没有 CS2 根目录 prop，也不应在其中调用 MapRotation IPC；本轮入口只用于展示“暂不可用”的说明卡片，不提供会误导用户的写入开关。
- 现有测试覆盖安装页/主题设置和 MapRotation 合约；新增测试必须证明概览卡片不渲染、主题设置入口可打开说明卡片且不可操作。

## 3. 实际执行方案

### P0：概览页暂时隐藏卡片

1. 修改 `src/views/OverviewView.vue`：移除 `MapRotationDefaultControl` 的 import 和模板节点，或使用明确的产品开关常量控制为 false；推荐直接移除主界面渲染，避免组件仍在后台调用 IPC。
2. 不删除 `src/components/MapRotationDefaultControl.vue`、类型、Tauri service 或 Rust 命令，保留后续修复和内部验证能力。
3. 概览页不显示“自动换图默认状态”、不显示读回 `enabled=0`，不提供可点击切换，避免把助手文件状态误称为游戏运行状态。
4. 不在概览页添加“功能暂时无效”的重复卡片；用户只在主题设置入口看到开发中说明，保持概览紧凑布局。

### P0：主题设置增加受控入口和说明卡片

1. 修改 `src/components/AppearanceSettingsDrawer.vue`，在抽屉主体中增加“实验功能”或“功能开发中”分组，加入按钮：
   - 可见文案：`自动换图默认状态（开发中）` 或 `查看自动换图默认状态`；
   - 使用 Lucide `Route` 图标；
   - 具备 `aria-label`、`title`，最小命中区 44px；
   - 点击只打开说明卡片/模态层，不调用 `get/set/reset_map_rotation_default`，不修改任何配置。
2. 说明卡片可复用 `MapRotationDefaultControl.vue` 的视觉结构，但必须是只读“开发中”版本；推荐新增 `MapRotationDefaultNotice.vue`，避免给原控制组件增加复杂的 disabled 分支。
3. 说明内容必须明确、友好且不制造错误承诺：
   - 标题：`自动换图默认状态`
   - 状态：`功能开发中，当前暂时无效`
   - 说明：`助手中的配置回读不等于游戏插件当前运行状态。插件侧兼容修复完成并通过真实 CS2 验证后再恢复此入口。`
   - 操作区不显示 Toggle、保存、重置或“已关闭”按钮；如需显示路径/当前回读，只能以“诊断信息，暂不作为游戏状态”标识，并且本轮建议完全不读取。
4. 说明卡片采用现有主题变量，支持浅色/深色、现有 palette/radius/density；不新增专属主题开关，不写 localStorage。
5. 关闭方式：右上角 `X` 图标按钮（`aria-label="关闭自动换图默认状态说明"`）、Esc、背景点击；打开时焦点进入关闭按钮，关闭后恢复入口按钮焦点；使用 Teleport/body 的现有模态层语义，`role="dialog"`、`aria-modal="true"`。
6. 如果执行 AI 选择将卡片内嵌在主题抽屉而不是二级模态，仍必须保留清晰的“开发中/暂时无效”状态和关闭回到主题设置的行为，不得出现两个可编辑入口。

### P1：组件与交互边界

- `AppearanceSettingsDrawer` 只负责入口和 modal open 状态；说明组件只负责展示和关闭事件；不引入 CS2 root、store、IPC 或网络请求。
- `MapRotationDefaultControl` 保持给未来内部/修复验证使用，但不得被概览页或主题设置直接复用为可编辑控件。
- 不新增版本号、发布说明或安装器资源；本轮是界面临时下线与可见状态澄清。

## 4. 自动化测试

新增/更新：

1. `tests/map-rotation-card-visibility.spec.ts`
   - mount 概览页时不存在 `.map-rotation-control`；
   - 不调用 `get_map_rotation_default`/`set_map_rotation_default`；
   - 现有概览启动、录制、模式和难度控件仍正常。
2. `tests/appearance-settings-map-rotation-notice.spec.ts`
   - 打开主题设置后存在“自动换图默认状态（开发中）”入口；
   - 点击后出现 `role=dialog`、`aria-modal=true`、标题和“功能开发中，当前暂时无效”；
   - 不渲染 Toggle/保存/重置按钮，不触发 MapRotation IPC；
   - Esc、关闭按钮、背景点击都能关闭，焦点可恢复；
   - 浅色/深色和 720x620、980x640、375px 无溢出。
3. 保留并运行现有 MapRotation 插件/助手合约测试；这些测试仍用于插件修复，不得因隐藏 UI 而删除。

建议命令：

```powershell
Set-Location E:\CS2AS05
npm test -- --run tests/map-rotation-card-visibility.spec.ts tests/appearance-settings-map-rotation-notice.spec.ts tests/map-rotation-contract.spec.ts tests/map-rotation-plugin-config.spec.ts
npm run typecheck
npm run lint
npm run build:web
```

## 5. 真实界面验收

1. 进入概览页：确认没有“自动换图默认状态”卡片，也没有该组件产生后台读取或错误提示。
2. 进入“安装与诊断”，点击“主题设置”：确认入口可见且不挤压现有主题、颜色、圆角、布局设置。
3. 点击入口：确认说明卡片清楚显示“功能开发中，当前暂时无效”，没有可误操作的 Toggle/保存/重置。
4. 在浅色、深色、不同 palette 和 720x620、980x640、375px 视口下打开/关闭，确认文字、关闭按钮和焦点均可见，无横向溢出。
5. 关闭说明后可继续使用主题设置；关闭主题设置后安装与诊断页的安装、更新、诊断、故障提交流程不受影响。
6. 记录截图到 `E:\CS2AS05\artifacts\map-rotation-default-card-disabled-20260831\`，至少保存概览页、主题设置、说明卡片浅色/深色各一张。

## 6. 停止条件与交付报告

任一条件成立即停止并报告“临时 UI 下线候选/待验收”：

- 概览页仍渲染可编辑卡片，或隐藏后组件仍后台调用 MapRotation IPC；
- 主题设置入口没有明确写“功能开发中，当前暂时无效”，或仍允许用户修改配置；
- 说明卡片把助手 `enabled=0` 描述为游戏当前已关闭；
- 主题抽屉在窄窗口溢出、模态无法通过 Esc/关闭按钮退出、焦点丢失；
- 为实现隐藏而删除 Rust 命令、配置服务或插件验证能力；
- 只有组件单测，没有真实桌面窗口多视口截图。

执行 AI 最终应新增执行报告，包含：实际修改文件、概览页隐藏证据、主题设置入口/说明文案、IPC 调用断言、浅深色多视口截图、自动化结果，以及插件侧 `enabled=1` 问题仍待独立修复/真实 CS2 验收的明确说明。不得把本轮 UI 下线当作插件故障修复完成。
