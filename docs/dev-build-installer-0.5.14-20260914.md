# 开发中助手安装程序构建记录（2026-09-14）

- 已关闭并确认没有工作区开发预览端口（5173/1420/8787）或 Vite/Tauri 开发进程。
- 执行 `npm run bundle:desktop`，前端生产构建与 Windows NSIS 打包完成。
- 安装程序：[CS2人机增强助手_0.5.14_x64-setup.exe](../src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.14_x64-setup.exe)
- 大小：116,421,008 bytes
- SHA-256：`D91DB434C6BDC1CFAE74BBC37E16FCB6A5A74B6777BB54045F348D6B55232757`
- 这是本地开发中安装包，未上传、未发布、未提交 Git。
- Tauri 最后一步提示未注入 `TAURI_SIGNING_PRIVATE_KEY`，因此没有生成新的 updater `.sig`；这不影响 NSIS 安装包本身生成。Windows Authenticode 签名也未额外注入。
