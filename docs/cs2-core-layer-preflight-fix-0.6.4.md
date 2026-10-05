# CS2 BuildID 25640462 兼容修复（0.6.4）

## 根因与官方基线

Steam 将 CS2 从旧 BuildID `25588766` 更新到 `25640462` 后，旧资源基线与新官方文件布局不再一致。受影响机器曾出现：

```text
Failed to load layered mod 'csgo_imported'. Can't read 'csgo_imported\\gameinfo.gi'
```

完整性校验完成后，当前官方根文件为：

- 来源：`D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\gameinfo.gi`
- 大小：`25985` bytes
- SHA-256：`8835DD99A78AF3064018388E09CEBE384D48B6B1F1C21B848E47C4FAEEADD621`

新 BuildID 的根 `gameinfo.gi` 已将原先 `csgo_imported`、`csgo_core` 的内容合并到当前根文件，并且不再声明 `LayeredOnMod`。因此，启动检查不能再无条件要求历史目录 `game/csgo_imported/gameinfo.gi`、`game/csgo_core/gameinfo.gi` 存在，否则会误拦截已经完整的新版 CS2。

## 资源重建

以新官方根文件逐字节生成：

| 条目 | 大小 | SHA-256 | 规则 |
| --- | ---: | --- | --- |
| `gameinfo.gi` | 25985 | `8835DD99A78AF3064018388E09CEBE384D48B6B1F1C21B848E47C4FAEEADD621` | 新 BuildID 官方根文件 |
| `backup/Online/gameinfo.gi` | 25985 | `8835DD99A78AF3064018388E09CEBE384D48B6B1C21B848E47C4FAEEADD621` | 与官方根文件逐字节一致 |
| `backup/WithBots/gameinfo.gi` | 26053 | `4E664BF53D6887B11AB185378B4D326A2D63052B6A7C07C6507FEE01D050FD4E` | 仅增加 `csgo/overrides/botprofile.vpk` 和 `csgo/addons/metamod` |
| `backup/SkinOnly/gameinfo.gi` | 26014 | `248216247C37F774537A85C59986179ECABD35A570E564BF3308AE8498EA26E8` | 仅增加 `csgo/addons/metamod`，不加载 BOT profile |

资源清单 `gameinfo.manifest.json` 已同步记录上述四个条目、官方摘要和 `sourceBuildId: 25640462`。资源 ZIP 当前 SHA-256：

```text
E3521B7474B6AE702F25247D1DFBDA776EAEB50AF690A1CAC7189B3B3CD8FD5D
```

## 启动前检查

`ensure_core_game_layers` 现在先检查 Steam 官方根 `game/csgo/gameinfo.gi` 是否可读且非空；随后只解析该文件实际声明的 `LayeredOnMod`，并检查被声明层的 `gameinfo.gi`。新版合并布局没有 `LayeredOnMod` 时直接通过，不再强制要求历史目录。

检查仍发生在 Panel 初始化、插件安装、模式切换和 Steam 启动之前，失败时不会留下新的配置或模式写入。

## 验证边界

- 已完成新 BuildID 官方文件读取、结构审计、四份资源生成和 ZIP 回读。
- 已完成版本切换到 `0.6.4`。
- 自动化 Rust、前端和资源契约验证需要在本轮代码变更后重新执行。
- 真实 CS2 BOT/Online/SkinOnly 效果由用户在本机验收；本记录不将本地构建结果等同于实机验收。
