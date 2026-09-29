# BOT 模式 CFG 未自动执行调查

日期：2026-09-19。范围：只读调查用户本机最近游戏记录，未修改游戏配置、源码或发布产物。

## 已确认的原因

Steam 在 2026-09-18 01:53:31 校验 AppID 730 时截断 12 个 gamemode CFG 末尾附加内容，移除了调用 my_bot_normal_config.cfg / my_bot_ffa_config.cfg 的入口。助手 BOT 启动检查未覆盖该入口，插件仍可加载而用户 CFG 不会被对应游戏模式自动执行。

## 现场证据

- Steam 日志：C:\Program Files (x86)\Steamzhen\logs\content_log.txt，第 16031–16056 行。先出现 Files Corrupt、Verifying Installed，随后逐项记录 Validation: truncated unwanted ... bytes。
- 受影响模式：armsrace、casual、competitive、competitive2v2、competitive2v2_offline、competitive_offline、custom、deathmatch、dm_freeforall、retakecasual、teamdeathmatch、workshop。
- 游戏根目录：D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive。
- 当前 game/csgo/cfg/gamemode_*.cfg 不含上述 my_bot 配置调用；修改时间与 Steam 校验一致。
- competitive_offline 当前 135 bytes，上游带调用版本 164 bytes，差 29 bytes；custom 当前 64 bytes，上游 93 bytes，差 29 bytes，与日志截断长度吻合。
- my_bot_normal_config.cfg 本身仍存在，保留 bot_aim mixed、bot_nades less；面板持久状态 mode=bots，gameinfo 包含 metamod 与 botprofile 路径。
- 2026-09-19 的 CounterStrikeSharp log-all 日志显示 00:18:00 加载 BOT 插件；助手 runtime.log 记录 00:17:48 启动、02:30:25 关闭，以及后续 02:48、02:50 短时重启。
- console.log 最后更新于 2026-08-31，没有最近一局完整命令执行轨迹；不能据此指定该局地图或每个 cvar 的最终值。

## 源码缺口

- src-tauri/src/services/cs2.rs inspect_bot_plugin_version_at 仅检查 marker 身份/版本、BotState.dll、NadeSystem.dll 和插件目录，不检查 CFG 调用链。
- src-tauri/src/services/panel.rs ensure_current_bot_plugin 对合格版本返回 unchanged；launch_cs2_inner 没有恢复 gamemode CFG 调用入口。

## 后续修复方向

BOT 启动前应对明确管理的模式检查并幂等恢复配置调用，保留 Steam 最新原生内容与用户 CFG；优先验证官方支持的 server CFG 扩展挂接方式。Online、SkinOnly 必须保持隔离。不要直接以旧整份 gamemode CFG 覆盖新游戏文件，也不要只在菜单阶段增加 +exec 当作地图加载后配置生效的证明。
