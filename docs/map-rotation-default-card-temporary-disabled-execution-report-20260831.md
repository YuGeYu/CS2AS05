# 自动换图默认状态卡片临时下线执行报告

日期：2026-08-31  
状态：临时 UI 下线候选，插件问题仍独立待真实验收。

## 已完成

- `src/views/OverviewView.vue`
  - 移除 `MapRotationDefaultControl` 的 import 和模板挂载。
  - 概览页不再显示自动换图默认状态卡片，也不会因该组件后台调用 MapRotation IPC。
- `src/components/MapRotationDefaultNotice.vue`
  - 新增只读说明模态框，不接收 CS2 根目录，不调用 IPC、网络或存储。
  - 标题：`自动换图默认状态`
  - 状态：`功能开发中，当前暂时无效`
  - 明确说明：`助手中的配置回读不等于游戏插件当前运行状态。`
  - 支持关闭按钮、Esc、背景点击、`role=dialog`、`aria-modal=true`。
- `src/components/AppearanceSettingsDrawer.vue`
  - 新增“自动换图默认状态（开发中）”入口。
  - 说明模态打开时聚焦关闭按钮，关闭后恢复入口按钮焦点。
- `src/styles/main.css`
  - 增加说明模态的主题化尺寸、状态色和移动端宽度约束。

保留未修改：`MapRotationDefaultControl.vue`、MapRotation Tauri commands、配置服务、插件代码、自动换图延迟以及 `lbtv_map_rotation`/`lbtv_map_next` 命令语义。

## 自动化验证

- `npm test -- --run tests/appearance-preferences.spec.ts tests/ui-design-contract.spec.ts`：8 tests passed。
- `npm run typecheck`：通过。
- 新增 `tests/map-rotation-card-visibility.spec.ts`，覆盖概览移除、只读入口、状态文案、ARIA 与无 IPC 字符串契约；待与下一轮窄测试一起执行。

## 真实验收边界

本轮未启动真实桌面窗口，未生成浅色/深色、720x620、980x640、375px 截图；因此不能宣称多视口焦点和布局验收完成。更不能把卡片隐藏或开发中说明入口当成插件 `enabled=1` 问题已修复；插件侧仍需按独立方案完成真实 CS2 重启、`enabled=0/1` 回读和自动换图日志验证。
