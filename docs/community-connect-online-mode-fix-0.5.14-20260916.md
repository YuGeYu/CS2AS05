# 玩家圈子连接前强制在线模式修复

日期：2026-09-16  
版本：开发中 0.5.14

## 问题

玩家在“玩家圈子”右键 `connect [A:...] (...)` 消息并选择“连接这台服务器”时，程序直接把连接 URI 交给 Steam，没有执行概览页的模式切换事务。如果上一次使用的是 BOT 或只开换肤模式，磁盘上的 `gameinfo.gi` 可能仍是对应的本地插件版本，CS2 启动阶段可能报 `Source2ServerConfig001`。

## 修复

- 前端必须传入当前已选择的 CS2 根目录；没有目录时直接提示玩家先前往概览页选择。
- 后端按“校验连接信息 -> 定位 Steam -> `set_mode_with_app(..., "online")` -> 启动 Steam URI”的顺序执行。
- 在线模式切换复用概览页已有事务，校验静态资源摘要、写入官方 Online `gameinfo.gi`、更新模式偏好并关闭只开换肤插件状态。
- 任何模式切换错误都会阻止 Steam 启动，不允许在未知或 BOT 配置下继续连接。
- 后端返回最新 `PanelSnapshot`，前端写回 Pinia，使概览页立即显示“在线模式”。

## 自动化验证

- `cargo check --manifest-path src-tauri/Cargo.toml --lib`：通过，仅有项目既有警告。
- 连接 URI 编码单测：1 passed，0 failed。
- `npm run typecheck`：通过。
- `npm run build:web`：通过。
- 新增 API/store 文件定向 ESLint：通过。
- `git diff --check`：通过。

## 验收边界

当前完成代码与构建验证，未替玩家启动真实 CS2。安装包含本修复的新构建后，需要从 BOT/只开换肤模式出发点击圈子连接，确认概览模式变为在线、CS2 不再出现 `Source2ServerConfig001`，并能进入目标服务器。

2026-09-16 只读检查开发机游戏目录时，活动 `gameinfo.gi` 与 `backup/Online/gameinfo.gi` 均为 9433 bytes、SHA-256 `B1391E73DBEC2E078BDBAF7279C2B955084CF2B38A47E3D8181662E7948679B8`。这说明检查时磁盘已处于在线文件状态，本修复是必要的入口一致性保护，但尚不能证明模式残留是该报错的唯一根因。若新包实测仍复现，应继续隔离 Steam URI/启动参数传递，不回退本次在线模式保护。

## 本地测试安装包

- 路径：`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.14_x64-setup.exe`
- 大小：116,514,122 bytes
- SHA-256：`EC8E69C7BB1C0E34FE5F9E6A157D74DD6FAAEB1E0F42DEC59BE8EDD28404A137`
- Windows Authenticode：`NotSigned`
- 本次进程未提供 Tauri updater 私钥，因此安装包已生成，但自动更新 `.sig` 未生成；该候选只用于本地安装测试，不作为自动更新发布件。
