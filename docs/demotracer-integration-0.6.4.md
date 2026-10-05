# DemoTracer Demo 重玩集成报告（0.6.4）

## 目标

0.6.4 将 Demo 重玩作为与人机增强同等地位的核心工作台能力。现有“对局复盘”继续负责
本地解析、报告、二维查看和普通 `+playdemo` 入口；“Demo 重玩”负责上游 DemoTracer 的
Demo 导入、回合筛选、`.dtr` 与 manifest 生成，以及 BOT 重演环境的安装和状态检查。

## 固定上游

- 仓库：[https://github.com/unicbm/demotracer](https://github.com/unicbm/demotracer)
- GUI：DemoTracer v1.5.3，SHA-256 `C1C8C23EEEEDAD97D9E7C00A7D68473817AEEF468D7EFEF5941E24595EF13402`
- Playback：DemoTracer CSS v1.5.2，SHA-256 `E19C420661ABE62CC3684C217BA93E0858BEBB7CC671F0CAB7ED3520CAFB909D`
- 源码快照：SHA-256 `6DEC83E8D9FDE90B5E4C0B464350FB6893786F2AF72B48D4928B811E765151DE`
- 许可证：上游第一方代码为 AGPL-3.0-only；源码包、Playback 内置许可证和第三方通知保留在项目资源中。

## 安装事务

Rust 服务 `src-tauri/src/services/demotracer.rs` 只接受 Playback 包内的
`addons/demotracer-install.v1.json` 清单，验证 ZIP 路径、每个文件大小和 SHA-256，并拒绝
绝对路径、盘符路径、父目录路径、重复条目、链接路径和非普通文件覆盖。

每次写入前会把原始文件复制到独立事务目录；所有文件成功写入并通过 ownership 清单写入后，
才提交 `cfg/cs2as05-demotracer-install-ledger.json`。写入或提交失败时会反向恢复已处理文件。
旧 ownership 清单引用的原始备份缺失或摘要不一致时，安装会停止，不会继续覆盖文件。

卸载前会先完整校验原始备份，并使用事务目录保存当前 Playback 文件。只有当前摘要仍等于助手
记录的 `installedSha256` 的文件才会恢复或移除。玩家、Steam 或其他插件修改过的文件会保留，
并继续留在 ownership 清单中等待下一次人工处理。助手不会通过目录名或文件名批量删除 Steam
官方 CS2 文件。

## 共存策略

Playback 包中的 `BotRandomizer` 是 DemoTracer 官方重玩所需的匹配版本，安装事务会保存同名
旧文件。`dtr-controller`、`dtr-hider`、DemoTracer managed plugins 和 API 放在上游规定的
目录中。CS2-Bot-Improver v1.4.5 的 Panel、BotController、BotHider 和其他资源继续由原有
安装事务管理；DemoTracer 的 `dtr-hider` 只在实际重玩时按上游逻辑接管，换图后恢复原 Hider。

## 页面和操作

“Demo 重玩”页面显示 GUI/Playback 版本、CS2 目录、CS2 进程、Metamod、CounterStrikeSharp、
Playback ownership 数量和漂移状态。页面明确提示 DemoTracer 不是普通 `.dem` 播放器，实际
重玩命令由上游 GUI 生成：

```text
dtr_preset 0x15; dtr_go seq "<manifest.json>" 0
```

页面不使用 `localStorage` 保存 DemoTracer 状态；状态来自 Rust IPC 和当前 Pinia CS2 store。

## 已完成验证

- `cargo fmt --manifest-path src-tauri/Cargo.toml`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `npm test`：56 个测试文件、200 个测试全部通过
- `cargo test --manifest-path src-tauri/Cargo.toml --lib services::demotracer::tests -- --nocapture`：3 个 DemoTracer 安装清单测试全部通过
- DemoTracer ZIP 清单、必需组件、路径穿越和同名条目校验代码已加入 Rust 服务
- GUI、Playback 和源码快照资源已复制到打包目录并记录 SHA-256

测试过程仅出现既有依赖的编译警告、Node `localStorage` experimental warning、测试资源解包提示和
jsdom canvas warning；没有失败测试。上述自动化结果不能替代真实 CS2 回合重演验收。

## 0.6.4 本地安装包

- NSIS：`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.6.4_x64-setup.exe`
- 大小：`131,239,656` bytes
- SHA-256：`4270C550034087842EEEC6CA46C843024D3EFC5E6706433E4354082493140E7B`
- Tauri updater `.sig`：同目录 `CS2人机增强助手_0.6.4_x64-setup.exe.sig`
- `.sig` 大小：`436` bytes
- `.sig` SHA-256：`ED22FD5743DD585CD07BFC8C896E35B5B2719985F9D4B8E3735A9BAC937A2F8F`
- Tauri 主程序：`src-tauri/target/release/CS2BotImproverAssistant.exe`
- 主程序 SHA-256：`B8A18C22F1B3CA08ED014C330B00951DE6DB439617C8DC83C8F113DED63A8E5D`

本次构建按项目签名流程读取当前 Windows 用户 DPAPI 密码，去除 DPAPI 内容首尾空白后解密，并在
当前构建进程注入 `TAURI_SIGNING_PRIVATE_KEY` 与 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。Tauri
已生成与本次 NSIS 文件配对的 `.sig`；生产 manifest 已逐字段匹配安装器大小、SHA-256 和签名正文。
签名公钥与 `src-tauri/tauri.conf.json` 一致。Windows Authenticode 仍按项目既有状态保持
`NotSigned`，这与 Tauri updater `.sig` 是两套独立机制。

## 待本机验收

构建安装包后，需要在真实 Windows CS2 环境完成：安装 Playback、打开 GUI、导入 `.dem`、导出
若干回合、执行上游 `dtr_go` 命令、确认 BOT 实际移动/输入/投掷物重演，再卸载 Playback 并在
Steam 中验证 CS2 文件完整性。网页、Rust 或安装包构建成功不等于真实回合重演已验收。
