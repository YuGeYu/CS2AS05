# Panel 首次启动缺少 core.json 修复记录

## 现象

少数玩家在 CounterStrikeSharp 已安装、但尚未首次启动本地服务端时，启动助手会看到：

`[PANEL_IO] 读取 CounterStrikeSharp core.json 失败：系统找不到指定的文件。（os error 2）`

## 原因

CounterStrikeSharp 会在首次服务端启动后懒创建 `addons/counterstrikesharp/configs/core.json`。Panel 初始化此前在其他基础文件齐全时无条件严格读取该文件，因此把“尚未生成”误判成 IO 故障。

## 修复

- Panel 初始化遇到缺少 `core.json` 时，按全开默认项创建规范 JSON。
- 已存在的 `core.json` 仍然严格解析；损坏、非对象或字段类型错误继续报告，避免覆盖玩家配置。
- 新增回归测试覆盖“文件缺失 -> 自动创建 -> 状态可读”流程。

## 验证

- `cargo fmt --check`：通过。
- `cargo test services::panel::tests::missing_core_json_is_created_during_panel_initialization --lib`：应通过。
- 同一测试组中的既有 `mutations_converge_to_the_aggregated_disk_snapshot` 仍受临时测试环境路径问题影响，与本修复无关。
