# Steam 启动权限错误修复记录（2026-09-29）

## 现象

新版本从概览页启动本地对局时，面板显示：

`[PANEL_IO] 启动 Steam 失败：D:\\sjume\\steam.exe 请求的操作需要提升。 (os error 740)`

## 原因

面板通过 `Command::new(steam.exe)` 启动 Steam。部分 Steam 安装带有需要管理员权限的执行清单，Windows 在非管理员进程直接创建它时返回 `ERROR_ELEVATION_REQUIRED (740)`。

## 修复

`src-tauri/src/services/panel.rs` 的 Steam 启动增加窄范围兜底：

1. 先按原逻辑普通启动，保持现有参数和权限行为。
2. 只有收到错误码 740 时，调用 Windows Shell `runas` 动词触发系统 UAC 提升。
3. UAC 取消或提升失败时返回明确错误；其他错误不改变原始错误路径。

因此不会把整个 CS2AS05 面板强制改成管理员运行，也不会改变 Online/BOT/SkinOnly 的启动参数。

## 验证

- `cargo test panel::tests --lib`：19 项通过；1 项既有临时路径测试失败（`mutations_converge_to_the_aggregated_disk_snapshot`，与本改动无关）。
- Rust 测试配置已成功编译 Windows `ShellExecuteW` 分支。
- 尚未在用户机器上点击 UAC 并实际启动 Steam，需用户在新构建中进行一次真实启动确认。
