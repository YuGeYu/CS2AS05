# MapRotation 自动换图延迟与外部默认开关执行报告（2026-08-27）

## 已完成

- `MapRotation.cs` 增加 `System.Text.Json` 配置读取、配置相对路径、默认开启 `true` 和 `AutoChangeDelaySeconds=15.0f`。
- `Load()` 从 `Server.GameDirectory/addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json` 读取下一次载入默认值；缺失、损坏、非 object 或字段非法时回退 `true` 并记录 `[MapRotation]` 日志。
- 最终回合检测和 `EventGameEnd` fallback 均使用 15 秒；日志继续输出 `delay=15.0s`。
- `lbtv_map_next` 仍立即执行；`lbtv_map_rotation 0|1` 只改变内存状态，不写回 JSON。
- 新增默认配置 `addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`，内容为 `{"enabled":true}`。
- 安装事务对该配置实行“目标已存在则保留”，避免升级覆盖玩家设置。
- ZIP 已原子合并 DLL 与 JSON，重新生成插件 marker。

## 构建与包证据

- `dotnet build .\third_party\CS2-Bot-Improver-map-rotation\addons\counterstrikesharp\plugins\MapRotation\MapRotation.csproj -c Release --nologo`：通过，0 warning，0 error。
- `npm test -- --run tests/map-rotation-contract.spec.ts`：通过，2 tests passed。
- ZIP `testzip()`：通过。
- JSON entry：`addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`，内容为 `{"enabled": true}`。
- DLL SHA-256：`D0393895EE8B18EC76923BED19541DC9884D8F9815766C45B930727B5168008C`。
- ZIP SHA-256：`98AAE825B880790A7967BBB1B567E7096E4865D2D7D11D5A4F536766A1AC01B7`。
- marker payload SHA-256：`76D9C84D74368DF6727435EAE0C830C7F75056E1E1F6E0AB14C128CD43DC9EF7`。

## 保留边界

安装过程中生成的备份位于 `src-tauri/resources/CS2BotImprover.zip.before-map-rotation-20260827-160708.bak` 和 `src-tauri/resources/CS2BotImprover.zip.before-plugin-manifest-20260827-160938.bak`，未删除原资源。未修改地图顺序、比分判定、`lbtv_map_next`、游戏内运行时开关语义、VAC 或版本号。

## 尚未完成的真实验收

用户仍需在 CS2 未运行时编辑目标配置验证 `enabled=false/true` 的下一次载入行为、确认运行中编辑不热覆盖、完成最终比分后约 15 秒才自动换图，并确认手动 `lbtv_map_next` 立即换图。上述实机画面和日志证据未由本轮执行产生，不能以静态测试替代。
