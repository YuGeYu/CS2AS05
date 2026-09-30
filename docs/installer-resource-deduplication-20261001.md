# 安装器资源去重记录

## 问题

`CS2BotImprover.zip` 同时通过 Rust 的 `include_bytes!` 嵌入主程序，又通过 Tauri `bundle.resources` 外置打包，导致正式安装器中出现两份相同资源。

## 修复

- 从 `src-tauri/tauri.conf.json` 的 `bundle.resources` 移除 `resources/CS2BotImprover.zip`。
- 保留 `src-tauri/src/services/cs2.rs` 的内嵌资源和 `bundled-resources` 回退逻辑。
- 安装时仍可从程序内嵌资源恢复 ZIP；外置资源目录只保留 Demo 地图、库存模拟器和 VPK 工具等确需按文件访问的资源。
- 增加安装器契约，防止以后重新把同一个 ZIP 加回外置资源清单。

## 体积对比

| 构建 | 安装器大小 |
| --- | ---: |
| 0.6.2 原构建 | 190,370,414 bytes |
| 去重后构建 | 119,679,970 bytes |
| 减少 | 70,690,444 bytes（约 67.42 MiB，37.13%） |

去重后构建仍包含约 73 MB 的单份内嵌 ZIP，但不再额外携带一份外置 ZIP。
