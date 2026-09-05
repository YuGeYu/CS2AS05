# v1.4.4 整包迁移执行报告

日期：2026-09-04  
工作区：`E:\CS2AS05`

## 结论

`v1.4.4 整包迁移和 0.5.10 定制恢复已通过自动化，待真实 BOT 验收`。

真实 CS2/BOT 尚未由用户执行，因此不宣称游戏行为或安装后运行完成。

## 已执行

- 官方 Windows 资产下载至 `workspace/vendor-cache/CS2-Bot-Improver-v1.4.4/CS2BotImprover.zip`。
- 资产大小 `68,536,430` bytes，SHA-256 `CBA05FDC239BF0E3E95DFF7D8670863FE69DE9FB03ADE8512B6D4F79F88664DB`，与发布证据一致；证据见 `artifacts/upstream-v1.4.4-migration-0.5.11-20260904/release-api.json`。
- 以 v1.4.4 原包为唯一基底，重新编译 MapRotation 和 NadeSystem（`dotnet build -c Release --nologo`，均 0 警告/0 错误），叠加 DLL 与 MapRotation 可变配置。
- 资源 ZIP 已原子替换，旧资源保存在同目录 `CS2BotImprover.zip.before-v1.4.4-20260904-185044.bak`，迁移前副本保存在 artifacts 目录。
- marker 已由 `scripts/generate-plugin-manifest.ps1` 重建：版本 `0.5.11`、`generatedFrom=CS2-Bot-Improver-v1.4.4`、payload 与 `mutableConfigEntries` 门禁已生成。
- Rust 资源门禁已同步 Panel v1.4.4 文件名、大小、SHA，以及候选 ZIP 最终 SHA `F53B1472924D362CE5F23B6114AF6576829ADCB791C496534BF1B83EE7AA31CC`。
- `gameinfo.gi`、`backup/Online/gameinfo.gi`、`backup/WithBots/gameinfo.gi` 保持 v1.4.4 官方包原始内容；未用本机 Steam 文件覆盖。
- 生产概览移除 VPKEdit/强度工坊入口；主题设置移除自动换图默认状态入口。源码、服务、Rust 命令、组件和 sidecar 均保留。
- 全量输入 manifest 和差异分类已生成：`input-manifests/{v1.4.4,custom-0.5.10,upstream-v1.4.3}.json`、`file-diff-classification.json`。

## 自动化状态

- MapRotation/NadeSystem .NET 构建：通过。
- `npm run build:web`：通过；`npm run typecheck`：通过。
- `cargo fmt --manifest-path .\\src-tauri\\Cargo.toml -- --check`：通过。
- `cargo check --manifest-path .\\src-tauri\\Cargo.toml`：编译进程已结束，工具窗口未回传最终 stdout/退出码，保守记录为未取得可引用退出证据。
- Vitest 定向测试（单 worker）：5 个文件、15 个测试通过。

## 待用户验收

退出 CS2 后安装候选，分别验证 Online/BOT、Low/Medium/High、NadeSystem、BotVision、MapRotation、Panel、切回 Online 及升级后自定义 profile；记录启动参数、插件日志、实际目录回读的 gameinfo/VPK/marker SHA 与异常退出码。

## 本机测试安装器

- 路径：`artifacts/upstream-v1.4.4-migration-0.5.11-20260904/CS2人机增强助手_0.5.11_x64-setup.exe`
- 大小：`114,824,735` bytes
- SHA-256：`64BCAE728B2C0D1024B8F3C9A4355DD5BC3E3008DEE15E273D76E8C2CE9D88AE`
- Authenticode：`NotSigned`。
- NSIS 已完成；Tauri 命令最后因 updater 公钥存在但未提供私钥而返回非零码。该错误不影响本机测试安装器文件本身。

## BOT 启动阻断修复（2026-09-04）

- 用户实测 BOT 启动报 `[BOT_PLUGIN_AUTO_INSTALL_FAILED] [GAMEINFO_ASSET_INVALID] 内置资源缺少 gameinfo manifest`；原因是资源 ZIP 缺少校验器要求的 `gameinfo.manifest.json`，并非 `gameinfo.gi` 内容错误。
- 按要求保留 v1.4.4 上游 `gameinfo.gi`、`backup/Online/gameinfo.gi`、`backup/WithBots/gameinfo.gi` 原始字节，仅补入与实际条目大小/SHA 完全对应的 manifest。
- 修复后资源 ZIP SHA-256：`D953265267DC205CAABBE031C81A70BF189948B19C08F808AE43B72CA4397E74`；Rust 门禁、ZIP 契约测试及 fixture 已同步。
- 追加修复：此前资源内 NadeSystem DLL 为旧定制版 `1.1.7`，覆盖了 v1.4.4 官方 `1.2.1`，因此缺少 BOT 投掷手雷时的 radio、sound、chat。现已恢复 v1.4.4 官方 NadeSystem DLL/deps/pdb；官方 DLL SHA-256：`9E4FC0CFD6B78D67C5EAEC86F76C76A7ACCC59820FCA3B02CBC517451D4074E4`。
- `gameinfo.gi` 及其备份仍保持 v1.4.4 上游原始内容，未使用本机 Steam 文件覆盖。
- 定向 Vitest：3 个文件、13 个测试通过；`npm run typecheck` 与 `cargo check --manifest-path src-tauri/Cargo.toml` 通过（仅既有 warning）。
- 新本机测试安装器：`artifacts/upstream-v1.4.4-migration-0.5.11-20260904/CS2人机增强助手_0.5.11_x64-setup-with-gameinfo-manifest.exe`，大小 `114,819,559` bytes，SHA-256 `BF1BF8B8094EFC5E3BAD709C7224F34FB6782E382D33D1A4C3B9C434688EDE65`，Authenticode `NotSigned`。
- 旧安装器缺少 manifest，不能用于本次 BOT 验收；真实 BOT 启动仍需安装上述新安装器后由用户实测确认。

## 逐文件基线审计（追加）

- 审计报告：`docs/upstream-v1.4.4-custom-package-file-audit-20260904.md`。
- 当前 ZIP 与 v1.4.4 staging 的 654 个上游文件逐一 SHA-256 对比：`UNCHANGED=654`、`CHANGED=0`、`REMOVED=0`；仅新增 4 个项目条目（MapRotation 配置/DLL、插件 marker、`gameinfo.manifest.json`）。
- NadeSystem 已改回官方 v1.4.4 三文件，当前包不再存在旧定制 DLL 覆盖上游组件的情况。
- 最新安装器：`artifacts/upstream-v1.4.4-migration-0.5.11-20260904/CS2人机增强助手_0.5.11_x64-setup-nadesystem-v1.4.4.exe`，SHA-256 `AEEFDCF9022DE44051AC16136BECDFAFD8CF6955491BF862E4BFFC6FC70FB302`。
