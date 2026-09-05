# MapRotation 自动换图修复记录

日期：2026-08-18

## 实际日志证据

日志：`D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\bin\win64\0818地图轮换.log`

日志显示手动命令可以工作：

```text
Command lbtv_map_next by advent#0: current=de_anubis, next=de_overpass
Executing command: changelevel de_overpass
```

但自动轮换没有进入调度分支：

```text
Map started: de_overpass, rotation enabled=0, index=1, next=de_inferno
target score reached, rotating: score=13-5, target=13
Not scheduling changelevel because rotation is disabled.
```

同时，状态查询返回 `enabled=0`，日志中没有 `lbtv_map_rotation 1`。根因是插件 `Load()` 每次载入都将 `_enabled` 初始化为 `false`，安装插件后自动换图实际上没有默认开启。

## 修复

- MapRotation 载入时默认 `_enabled = true`。
- 保留 `lbtv_map_rotation 0|1`，可手动暂停或恢复自动换图。
- 重新编译 `MapRotation.dll`，并重新注入 `src-tauri/resources/CS2BotImprover.zip`。
- 重新生成 `CS2AS05.plugin.json` 完整性清单。

## 产物校验

```text
MapRotation.dll SHA-256:
F6C2C8713200B754B1142C1DD862DACBCC753B721E488BA62623732F4ABCEC96

CS2BotImprover.zip SHA-256:
7E58C5E4739E9B2BDE99F274C5DC0CDA4D9142B1BD7CA8E672956A59DA3683A4
```

包内 `MapRotation.dll` 已回读并确认与构建输出哈希一致。Rust 真实安装事务测试和新增 MapRotation 契约测试均通过。
