# Demo 提前结算专项修复 0.5.8 执行报告

日期：2026-08-14

## 问题

0.5.7 在 CS2 仍运行时，后台若有一次进程枚举未找到 `cs2.exe`，会立即结束当前游戏会话。回合间 Demo 短暂停止增长时，监听器可能把只包含一两回合的文件视为稳定成品并生成战报。

## 修复

- Demo Rust 门禁新增共享进程状态机；单次漏检与同一时刻的并发检查均不能解除门禁。
- 只有从第一次未发现 CS2 起持续 8 秒，并取得至少 4 个有效停止样本，才确认游戏已经退出。
- 赛后会话使用同一确认状态；确认完成前不结束 session、不扫描、不领取解析任务、不提交战报。
- `refresh_watcher()` 为旧协调器设置停止令牌，目录刷新后旧线程会退出，不再长期保留历史 pending 集合。
- 应用版本、Cargo/npm/Tauri 元数据及内置插件包标记同步为 0.5.8。

## 自动验证

- Rust：79 passed、0 failed、3 ignored。忽略项均为既有的真实 Demo/真实 CS2 环境测试。
- 前端：类型检查通过；Oxlint 0 warnings / 0 errors；ESLint 通过。
- Vitest：完整运行原为 160 passed / 1 个资源哈希预期失败；同步本次 ZIP 标记哈希后，该文件 6/6 测试通过。
- Web production build：通过。
- Tauri release + NSIS + updater signature：通过。

## 本地候选

- 安装器：`E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.8_x64-setup.exe`
- 大小：114,529,298 bytes
- SHA-256：`08A6CEAC7B27829A8464B2F521E8AFAC75B8DA787E80FBFFEB8689E4E2BE4757`
- Tauri updater 签名：同路径 `.sig`，436 bytes
- `.sig` SHA-256：`4EF22C0EA9547FC65953EEDA182B1C576AA15B638A86E3A839C7DB678A36366D`
- FileVersion / ProductVersion：0.5.8
- Windows Authenticode：`NotSigned`；与 Tauri updater `.sig` 分属不同机制。

## 玩家实测重点

进入真实对局后至少连续完成数个回合，确认对局中途不出现赛后战报；正常退出 CS2 后等待约 8 至 15 秒，确认助手只生成一次包含完整回合的战报。真实 CS2 行为需要由玩家环境完成最终确认。
