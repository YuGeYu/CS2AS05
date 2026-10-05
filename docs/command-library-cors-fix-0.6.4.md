# 指令研究所 Failed to fetch 修复（0.6.4）

## 现象

在桌面助手的“指令研究所”页面，搜索框和首字符筛选可以渲染，但资料区固定显示英文 `Failed to fetch`，结果为 `0` 条。

## 根因证据

直接请求 `https://cs2as.600318.xyz/api/command-library` 返回 `HTTP 200` 和完整 JSON；响应是 `application/json`，但没有 `Access-Control-Allow-Origin`。桌面 WebView 直接调用浏览器 `fetch` 时会被 CORS 拦截，因此页面实际拿不到已经正常发布的指令资料。

## 修复

- 新增 Tauri `get_command_library(force)` 命令，由 Rust `reqwest` 后端请求官网接口，绕开 WebView CORS 限制。
- Rust 后端设置 20 秒超时、JSON `Accept` 头和 `CS2AS05/0.6.4` User-Agent；对 HTTP 错误、读取失败、超过 16 MiB、无效 JSON、空资料和缺少命令名称返回中文错误。
- 前端桌面运行时改为 `invoke('get_command_library')`；网页预览仍保留直接请求，网络/CORS 失败时显示中文提示，不再把浏览器原始 `Failed to fetch` 直接暴露给玩家。
- “重新加载”通过 `force` 参数追加时间戳，避免继续使用旧的 CDN 快照。

## 验证

- 线上接口现场回读：`HTTP 200`，返回 `schemaVersion/source/commands` 完整 JSON；确认缺失 CORS 头是根因。
- `npm run test -- --run tests/command-library.spec.ts tests/app-close.spec.ts`：7 tests passed。
- `npm run typecheck`：通过。
- `cargo fmt --manifest-path src-tauri/Cargo.toml`：通过。
- `cargo check --manifest-path src-tauri/Cargo.toml`：通过。

## 构建产物

- 安装器：[CS2人机增强助手_0.6.4_x64-setup.exe](../src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.6.4_x64-setup.exe)
- SHA-256：`32318D6C15C20BA3BB553235DA214FE913D5A2C42FE61808C29684027F90120D`
- 文件大小：`120,171,346` bytes
- Windows Authenticode：`NotSigned`
- Tauri updater `.sig`：未生成；当前环境只有公钥，没有 `TAURI_SIGNING_PRIVATE_KEY`。

## 玩家验收

1. 安装新版 0.6.4，打开“指令研究所”。
2. 页面应显示命令总数和命令表格，不再显示 `Failed to fetch`。
3. 输入 `bot`、点击 `A`/`#` 首字符筛选，确认结果数量和列表会变化。
4. 点击任意命令或复制图标，确认显示复制成功提示。
5. 点击“重新加载”，确认仍能加载资料；网络不可用时应显示中文连接错误和可重试按钮。
