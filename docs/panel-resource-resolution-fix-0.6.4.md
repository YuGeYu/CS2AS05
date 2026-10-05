# Panel v1.4.5 资源选择修复记录

## 问题

点击“安装定制插件包”时，运行时可能从旧的 Tauri `resources` 目录选择一个仍然可读取的 `CS2BotImprover.zip`。旧包只含 `Panel v1.4.4.exe`，但旧的选择逻辑只检查 ZIP 是否能打开，因此后续安装校验才报：

```text
[PANEL_ASSET_INVALID] 上游资源缺少 Panel v1.4.5。
```

## 现场证据

- 当前项目资源：`src-tauri/resources/CS2BotImprover.zip`
  - SHA-256：`AE37B86533ABFE0547C5AD4346D478CD846727509FC092842A81240EB0130140`
  - 包内精确条目：`Panel v1.4.5.exe`
  - Panel 大小：`6,032,896` bytes
  - Panel SHA-256：`9C6BD8E2503AFC9CAEB5DD64C8B8BF0EC5967BF50CD442015E7CEBEB69038410`
- 本机旧安装缓存：`%LOCALAPPDATA%/CS2人机增强助手/resources/CS2BotImprover.zip`
  - SHA-256：`DFB6907BAE020F57FE98FA33DC77A01F5FA05F636EAB1296C472AEB53F597A95`
  - 只发现 `Panel v1.4.4.exe`，大小 `5,890,560` bytes
- 运行日志曾记录实际选中旧副本：`C:/CS2AS05/workspace/resources/CS2BotImprover.zip`

## 修复

1. `resolve_zip_path` 不再接受“能打开的 ZIP”；每个候选包在返回前都执行完整 `verify_custom_zip`，包含上游核心文件、gameinfo、禁止下游组件和 Panel v1.4.5 大小/SHA-256 校验。
2. 旧包会被记录为跳过项，继续寻找其它候选；如果没有通过验证的外部候选，则从当前版本的 `include_bytes!` 内嵌资源构建应用数据缓存。
3. 内嵌资源写入使用临时文件、校验后原子改名；旧缓存先改名为临时 previous 文件，写入失败会恢复，避免在缓存修复过程中丢失旧文件或留下半包。
4. Panel、SkinOnly gameinfo、WithBots gameinfo 和 BotVision 的读取统一使用标准化 ZIP 条目查找，兼容上游包的 `./` 前缀。
5. Panel 提取失败会清理临时文件，错误会带资源路径；不会把 Panel 写入 CS2 游戏目录。

## 自动化验证

- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- Rust 回归测试：旧版 Panel 候选被拒绝并继续选择 v1.4.5；`./Panel v1.4.5.exe` 条目标准化匹配。
- `cargo test --manifest-path src-tauri/Cargo.toml bundled_custom_zip_verifies_and_extracts_into_fake_cs2_root`
- `npx vitest run tests/panel-data.spec.ts tests/installer-contract.spec.ts --pool=threads --maxWorkers=1`

## 最终本机安装器

- 路径：`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.6.4_x64-setup.exe`
- 大小：`119,955,256` bytes
- SHA-256：`8E83D11D14D54304A4EF020D11BC69B22B7B27E6745BC9B043E9CD726A461857`
- Windows Authenticode：`NotSigned`
- Tauri updater `.sig`：未生成（本机只有公钥，没有 `TAURI_SIGNING_PRIVATE_KEY`）

## 验收边界

本修复不修改真实 CS2 目录，也不删除本机旧缓存。新安装器首次执行安装时会自动跳过旧副本并在应用数据目录创建已验证的 v1.4.5 缓存。Panel 启动、BOT 行为和 Steam 文件验证仍需在目标机器上使用新安装器进行真实验收。
