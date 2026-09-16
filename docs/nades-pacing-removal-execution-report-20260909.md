# bot_nades 道具节奏限制取消执行记录

日期：2026-09-09  
工作区：`E:\CS2AS05`

## 已完成

- 使用 `third_party/CS2-Bot-Improver-v1.4.4/addons/counterstrikesharp/plugins/NadeSystem` 上游源树构建 NadeSystem。
- 未使用旧的 `third_party/CS2-Bot-Improver-v1.4.3/nades-pacing` 编译输入。
- 新构建产物：
  - `NadeSystem.dll`：71,680 bytes，SHA-256 `F9BBE17D1CA4729144A4F7CC37E1CD47B76729BC476E741987D0AC77F142F907`
  - `NadeSystem.deps.json`：34,686 bytes，SHA-256 `4483C91952AFF344AC05A6962280867B1BBA441F65D5EB87B687C64061FDB021`
  - `NadeSystem.pdb`：31,136 bytes，SHA-256 `5E30156CD195D6F9B4D60F9CB0D50C6F95E5E17BE53583000552BC3AA8D823BA`
- 将上述三个文件替换到 `src-tauri/resources/CS2BotImprover.zip`，其余 ZIP 条目保持原候选内容。
- 重新生成 NadeSystem marker，payload digest 为 `170EF10093023672510641D083DD2AC5439E123B7E9617850CCCF732B2EFBC70`。
- 当前候选 ZIP：70,171,530 bytes，SHA-256 `E985832CCBE41AEB031914C969F59878B3EDF650E153EB7BDF40FB10FC81AE16`。
- 候选前 ZIP 备份：`artifacts/nades-pacing-removal-0.5.12-candidate-20260909-014623/CS2BotImprover.zip.before`。
- 可重复替换脚本：`scripts/replace-nadesystem-zip-entries.ps1`；默认 DLL 路径已改为上游源码树构建输出。
- Rust `CUSTOM_ZIP_SHA256` 与 ZIP 实际摘要同步；安装器契约中的 NadeSystem DLL 摘要同步。

## 自动化结果

- `dotnet build third_party/CS2-Bot-Improver-v1.4.4/addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.csproj -c Release --nologo`：通过，0 错误；有 1 个 `RayTraceApi` 引用解析警告。
- `cargo test --manifest-path src-tauri/Cargo.toml bundled_custom_zip_verifies_and_extracts_into_fake_cs2_root -- --nocapture`：通过。
- `cargo fmt`：通过。
- `git diff --check`：通过。
- `npm run typecheck`：通过。
- `npm run build:web`：通过。
- 定向 Vitest 因 fork worker 60 秒启动超时未执行到测试用例，不能记为通过；该超时发生在测试 worker 启动阶段，没有测试断言结果。

## 关键行为变化

候选 NadeSystem 不再编译或运行 `NadePacingPolicy`。因此不再有 CS2AS05 额外的：

- BOT 5 秒投掷间隔；
- 队伍 0.5 秒投掷间隔；
- 开局 15 秒预算；
- 开局烟雾/进攻道具上限；
- `less` 四枚总量硬上限；
- 每 BOT 每类型自定义数量上限；
- `normal` 计划烟雾每队一回合上限。

上游 NadeSystem 的经济、地图投掷点、视线、回合状态、特殊投掷、失败处理和投掷通知仍由上游代码负责。

## 尚未完成

- 尚未在真实 CS2 客户端启动候选包。
- 尚未由用户执行 `max`、`more`、`normal`、`less`、`off` 五档真实回合对照。
- 尚未证明创意工坊地图或真实玩家环境中的投掷数量变化。
- 尚未构建、签名、上传或发布新的修复版本。

## 停止边界

0.5.12 已公开发布，本次没有覆盖其线上安装器、R2 对象、GitHub Release 或更新 feed。当前工作区使用版本 `0.5.13` 的 marker/资源门禁作为候选开发状态；真实游戏验收通过前，不得把候选称为正式修复版本。
