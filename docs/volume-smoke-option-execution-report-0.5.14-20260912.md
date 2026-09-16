# 0.5.14 体积烟可选功能执行记录

## 目标

- 玩家可在“安装与诊断”中取消勾选“体积烟功能”。
- 关闭时删除 `Counter-Strike Global Offensive\game\csgo\addons\metamod\BotVision.vdf`。
- 提醒玩家：如果一场游戏出现卡顿，可以尝试关闭体积烟。
- BotVision 只来自上游 CS2-Bot-Improver v1.4.4 整包，不再作为助手额外组件写入自定义插件清单。

## 已实现

- 默认启用体积烟；关闭与重新启用均要求 CS2 已退出。
- 偏好保存在游戏目录 `game\csgo\cfg\cs2as05-volume-smoke.state`，不使用 WebView 存储。
- 关闭时只删除加载入口 `BotVision.vdf`，保留上游整包中的 DLL 和数据文件。
- 重新启用时从内置且已校验的 v1.4.4 资源包恢复 `BotVision.vdf`。
- 覆盖安装和自动插件更新后继续应用玩家原有选择。
- 插件清单不再记录独立 `botvision` 组件，也不再把 BotVision 文件计入助手自定义 payload 摘要。

## 验证

- `npm run typecheck`：通过。
- `cargo test --manifest-path src-tauri\Cargo.toml services::cs2::tests --lib`：9 项通过，0 项失败。
- 内置 ZIP SHA-256：`9785FE77F507E7E7F682B231206255387162CB849540596A0BD17856BBABBBBA`。
- 新增回归测试证明关闭后 VDF 被删除、其他 BotVision 文件保留、disabled 状态持久化、重新启用状态恢复。

## 尚未执行

- 未在真实 CS2 对局中验证帧率改善，由玩家设备验收。
- 未构建 Windows 安装程序，未提交、推送或发布。
