# 关闭助手弹窗与托盘逻辑修复（0.6.4）

## 问题

截图中的关闭确认弹窗只能表达两个动作，没有明确说明退出会断开语音大厅连接；“记住选择”写入 `sessionStorage`，重启桌面助手后不会保留。Windows 的原生关闭事件还在 Rust 层无条件隐藏窗口，可能绕过前端确认弹窗。

## 修复

- 关闭请求统一由 `src/App.vue` 处理。更新已下载时仍优先显示更新退出确认。
- 移除 Rust `WindowEvent::CloseRequested` 的无条件 `hide()`，因此标题栏、Alt+F4 和其他窗口关闭请求都能进入同一套确认逻辑。
- “留在系统托盘”和“退出程序”改为带后果说明的整行选项；增加“返回助手”和 `Esc` 取消；弹窗打开后自动聚焦，并在弹窗内部循环 Tab 焦点。
- “记住这次选择，下次直接执行”改为写入应用数据目录：
  `%LOCALAPPDATA%\CS2人机增强助手\close-choice-preference`
  文件值为 `tray` 或 `exit`。
- 保存偏好完成后才执行隐藏或退出，避免窗口先销毁导致记忆失败；保存失败仍执行玩家当前选择。
- “退出程序”会先销毁战报窗口，再销毁助手主窗口；托盘菜单中的“退出程序”保持直接退出行为。
- “清除助手数据”同时清除关闭选择和主动推送偏好，便于恢复首次运行状态。

## 验证

- `npm run test -- --run tests/app-close.spec.ts`：6 tests passed。
- `npm run typecheck`：通过。
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`：通过。
- `cargo check --manifest-path src-tauri/Cargo.toml`：通过；仅保留第三方解析器及既有未使用代码警告。
- `npm run verify`：56 files / 199 tests passed，Web 构建通过。

## 构建产物

- 安装器：[CS2人机增强助手_0.6.4_x64-setup.exe](../src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.6.4_x64-setup.exe)
- SHA-256：`A331BDB8F92112FDE858484A98FD3FECF3374A4D61ADF9A9141A67B26649B0EB`
- 文件大小：`120,203,406` bytes
- Windows Authenticode：`NotSigned`
- Tauri updater `.sig`：未生成；当前环境只有公钥，没有 `TAURI_SIGNING_PRIVATE_KEY`。

## 玩家验收步骤

1. 安装新版 0.6.4，打开助手后点击标题栏关闭。
2. 选择“返回助手”或按 `Esc`，窗口应保持打开。
3. 再次关闭，勾选“记住这次选择”，选择“留在系统托盘”；窗口应隐藏，语音大厅连接继续保持。
4. 从托盘重新打开后再次关闭，应直接隐藏且不再弹窗。
5. 通过“清除助手数据”恢复首次运行状态，再次关闭应重新显示选择弹窗。
6. 选择“退出程序”，应结束助手并断开相关连接；重启后不应残留战报窗口。
