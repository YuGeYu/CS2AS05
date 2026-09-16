# 只开换肤 SkinOnly 摘要不匹配调查与修复方案

日期：2026-09-09  
目标版本：0.5.13 后续修复构建

## 用户反馈

选择“只开换肤”时出现：

```text
[GAMEINFO_ASSET_INVALID] 静态 gameinfo 资源摘要不匹配，已阻止写入。
```

## 调查证据

玩家实际目录已经存在：

```text
game/csgo/backup/SkinOnly/gameinfo.gi
```

并且该文件大小与摘要为：

```text
9473 bytes
AEFB44F51339F8422DA9F36EAC87E26E87D92129847BEDF2130227F8A0DFE780
```

该摘要与当前内置 ZIP 和 manifest 一致，因此本次不是 SkinOnly 文件缺失，也不是资源内容损坏。

玩家现有 sidecar：

```json
{
  "schema": 1,
  "resourceVersion": "0.5.13",
  "officialSha256": "B1391E73DBEC2E078BDBAF7279C2B955084CF2B38A47E3D8181662E7948679B8",
  "onlineSha256": "B1391E73DBEC2E078BDBAF7279C2B955084CF2B38A47E3D8181662E7948679B8",
  "botsSha256": "3CA9C2342366EC08428916F1F60D2935AD9354EB2916C7F0253EB1404F5132CC",
  "activeSha256": "3CA9C2342366EC08428916F60D2935AD9354EB2916C7F0253EB1404F5132CC"
}
```

其中没有 `skinOnlySha256`。

当前代码链路为：

1. `ensure_skin_only_gameinfo()` 已经能够补齐 SkinOnly 文件；如果文件已存在且内容正确，会直接返回。
2. `gameinfo_state()` 和 `write_mode_at()` 从 `GameInfoSidecar.skin_only_sha256` 读取摘要。
3. 旧 sidecar 通过 `#[serde(default)]` 可以解析，但字段值为 `None`。
4. `write_mode_at()` 对 SkinOnly 的预期摘要为 `None`，命中 `expected.is_none()`，于是即使磁盘文件正确也被拒绝。

## 根因

上一轮修复只迁移了新增的 SkinOnly 文件，没有同步升级旧安装的 `cs2as05-gameinfo-state.json`。可选字段的反序列化兼容保证了程序不会崩溃，却没有在首次进入 SkinOnly 时完成 sidecar 回填，导致“文件正确、元数据过期”的摘要拒绝。

## 修复目标

- 旧安装无需重新安装完整 BOT 包即可进入“只开换肤”。
- 文件正确但 sidecar 缺字段时自动补齐摘要。
- sidecar 回填不改变玩家活动 `gameinfo.gi`、BOT 难度和插件目录。
- 保持摘要校验：不能通过写入任意未验证的 gameinfo 文件绕过保护。
- 对 sidecar 已损坏、官方基线不一致的安装继续拒绝写入，并给出恢复提示。

## 推荐实现

### 1. 新增 SkinOnly sidecar 回填函数

在 `src-tauri/src/services/panel.rs` 增加专用函数，例如：

```text
ensure_skin_only_gameinfo_sidecar(csgo: &Path) -> Result<(), AppError>
```

行为：

- 读取现有 `GameInfoSidecar`。
- 读取并计算 `backup/SkinOnly/gameinfo.gi` 摘要。
- 若 `skin_only_sha256` 已存在且等于实际摘要，直接返回。
- 若字段缺失，或字段与实际摘要不一致，仅更新该字段并原子写回 sidecar。
- 写回前保留原有 `official_sha256`、`online_sha256`、`bots_sha256`、`active_sha256`、版本和生成时间。
- 若 sidecar 不存在、JSON 损坏、SkinOnly 文件不存在或官方/WithBots 基线不完整，返回明确错误，不创建伪造摘要。

### 2. 调整 SkinOnly 切换顺序

在 `set_mode_inner()` 的 SkinOnly 分支中：

1. 校验 Inventory Simulator。
2. 执行 `cs2::ensure_skin_only_gameinfo()`，确保文件来自已验证内置 ZIP。
3. 执行 sidecar 回填函数。
4. 再执行 `write_mode_at()` 的摘要比对和活动 gameinfo 原子替换。
5. 最后隔离 BOT 插件并保存模式偏好。

这样 `write_mode_at()` 永远只接受已写入 sidecar 的内置资源摘要。

### 3. 处理迁移期间状态

- `gameinfo_state()` 在旧 sidecar 缺少 SkinOnly 字段时，不应把 Online/BOT 状态立即误报为整体 `recoveryRequired`；可以将 SkinOnly 缺失视为“待迁移”，但保持 Online/BOT 的既有基线校验。
- 当用户实际请求 SkinOnly 时再完成回填，避免应用启动扫描阶段修改玩家文件。
- 如果目标文件摘要不一致，先重新从 ZIP 迁移，再回填 sidecar；迁移失败不得修改 sidecar。

### 4. 保持文件边界

- 不修改 `overrides/Low|Medium|High`。
- 不修改 `overrides/botprofile.vpk`。
- 不删除或覆盖玩家自定义插件。
- 不通过把玩家当前 `gameinfo.gi` 摘要写入 sidecar 来绕过资源校验。

## 测试方案

### Rust 测试

1. 创建旧格式 sidecar（无 `skin_only_sha256`）和正确 SkinOnly 文件，执行回填，断言字段被写入且其它字段保持不变。
2. 旧格式 sidecar + 缺失 SkinOnly 文件，先执行资源迁移，再回填并成功切换。
3. SkinOnly 字段存在但摘要错误，断言不会直接信任该值；重新迁移后更新为内置摘要。
4. sidecar JSON 损坏或缺少官方基线，断言返回错误且不写入活动 `gameinfo.gi`。
5. 回填后 `write_mode_at("skin_only")` 成功，Online/BOT 切换仍使用原有摘要。

### 静态契约

- ZIP、manifest、SkinOnly 摘要和 Rust 常量保持一致。
- `cargo check`、`cargo fmt --check`、`npm run typecheck`、`git diff --check` 通过。

### 真实 Windows/CS2 验收

- 使用当前出现问题的旧安装目录直接运行修复版。
- 不重新安装 BOT 包，选择“只开换肤”应成功。
- 确认 sidecar 出现 `skinOnlySha256`，值为内置 SkinOnly 摘要。
- 确认库存换肤正常、BOT 难度不被接管、创意工坊闯关下一关正常推进。
- 切回 BOT/Online 后原有难度和插件恢复。

## 发布边界

当前 0.5.13 签名候选尚未包含本次 sidecar 回填修复。重新实现、验证并签名构建前，不应把该报错标记为已解决，也不应上传或启用线上 updater。
