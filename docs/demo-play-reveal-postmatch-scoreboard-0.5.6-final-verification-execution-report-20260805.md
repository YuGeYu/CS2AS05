# 0.5.6 最终验证执行报告

日期：2026-08-05  
结果：Gate A 通过，Gate B/C 已由用户确认通过，已生成本地发布候选；未公开发布

## Gate A

证据目录：`E:\CS2AS05\workspace\release-evidence\0.5.6-final-gate-20260805-185112\`

| 命令 | Exit code | 结果 |
|---|---:|---|
| `npm run workspace:check` | 0 | workspace registry valid |
| `npm run typecheck` | 0 | 通过 |
| `npm run lint` | 0 | oxlint 0 warning / 0 error，ESLint 通过 |
| `npm test -- --pool=threads --maxWorkers=1` | 0 | 35 files，127 tests 全部通过 |
| `npm run build:web` | 0 | Vite production build 通过 |
| `cargo fmt --manifest-path .\\src-tauri\\Cargo.toml -- --check` | 0 | 通过 |
| `cargo check --manifest-path .\\src-tauri\\Cargo.toml` | 0 | 通过 |
| `cargo test --manifest-path .\\src-tauri\\Cargo.toml --lib` | 0 | 62 passed，0 failed，3 ignored |
| `cargo clippy --manifest-path .\\src-tauri\\Cargo.toml --all-targets -- -D warnings` | 0 | 通过 |
| `git diff --check` | 0 | 通过 |

第三方 demoparser 存在既有 unused/lifetime warning；Gate 命令本身为 0，未修改第三方 warning。

## Explorer 特殊路径

测试副本：`E:\CS2AS05\workspace\release-evidence\0.5.6-final-gate-20260805-185112\Explorer 特殊, 路径 & demo\特殊, Demo & 验证.dem`

原始 Demo 和测试副本均为 `73,800,957` bytes，SHA-256 均为：

`A1FB3A5960E52EDE9F968738FDC4C72A0B570B79C82C22285B2310D6A0B3D315`

已启动开发版 Tauri，确认窗口 `CS2人机增强助手` 能运行；也执行了独立参数数组形式的 Explorer 调用。用户已确认 Explorer 正确打开目标目录并选中了包含中文、空格、逗号和 `&` 的目标 Demo，因此该项记为“用户实机确认通过”。Tauri/Vite/cargo 子进程已停止。

## 已确认的真实验收

- 用户确认真实 CS2 Demo 播放、原文件 fingerprint 保持不变、三回合 BOT 新局、最新 Demo 自动战报、回放退出 suppression、干净安装、0.5.5 覆盖升级、卸载重装和旧版 updater 通过。
- Explorer 特殊路径此前已由用户确认正确打开并选中文件。

## 尚未完成的真实验收

- 通过应用 IPC 点击 FolderSearch 并由用户确认 Explorer 正确选中状态：已通过。
- 真实验收的详细 session/candidate/job/ready/present 原始日志尚未由执行 AI 读取；结论依据为用户确认。
- 失败、损坏 JSON、旧版本、0 回合、空玩家、unavailable scoreboard 的空窗抑制。
- 多次加时真实样本。当前算法路径使用 canonical completed-round winner；尚未证明真实多次加时结果。
- 多次加时真实样本及线上公开发布/线上 feed 回读。

## 发布边界

旧 `dist-release\\cs2-bot-improver\\0.5.6` EXE、`.sig`、manifest 和 SHA 清单已移动到带时间戳的 evidence backup，没有复用。Gate B/C 已由用户确认通过，本轮已运行 bundle 并生成本地候选，但没有 commit/tag/push/GitHub Release，也没有写 R2/D1/feed。当前状态为“可发布候选，待用户授权公开发布”；多次加时真实样本和线上回读仍未完成。

## 本地发布候选

交付目录：`E:\CS2AS05\dist-release\cs2-bot-improver\0.5.6`

- `CS2人机增强助手_0.5.6_x64-setup.exe`：112,414,780 bytes；SHA-256 `590AC0D6A4890B7366463A7E2A0829725A0C35D4D5155EF06BC7A5A9036BF261`；Authenticode `NotSigned`。
- `CS2人机增强助手_0.5.6_x64-setup.exe.sig`：436 bytes；SHA-256 `723BDF9CD3FEC3CFE5DF855E218238EB5AA603E6D9548BDDD6813FE65F07C690`。
- `CS2人机增强助手_0.5.6_x64-setup.exe.sha256` 和 `SHA256SUMS.txt`：已生成，逐字匹配最终 EXE hash。
- `updater-prod.json`：`version=0.5.6`、`channel=prod`、`projectId=cs2-bot-improver`；安装器字段为 basename，无本机绝对路径；manifest size/hash/signature 与最终文件匹配。
- `LICENSE`、`NOTICE.md`、`release-notes-0.5.6.md`：已复制到交付目录。

一致性证据：`E:\CS2AS05\workspace\release-evidence\0.5.6-artifact-rebuild-20260805-193244\artifact-consistency.json`。旧同版本文件位于该目录的 `old-0.5.6`，未复用、未删除。
