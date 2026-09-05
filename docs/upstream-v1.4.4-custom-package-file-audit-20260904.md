# v1.4.4 定制包逐文件审计（2026-09-04）

## 审计结论

审计对象：

- 上游基线：`workspace/vendor-cache/CS2-Bot-Improver-v1.4.4/extracted`
- 当前定制包：`src-tauri/resources/CS2BotImprover.zip`
- 当前 ZIP SHA-256：`D953265267DC205CAABBE031C81A70BF189948B19C08F808AE43B72CA4397E74`

对上游 staging 的 654 个文件与当前 ZIP 解压内容逐一计算 SHA-256 后，结果为：

| 分类 | 数量 | 说明 |
| --- | ---: | --- |
| `UNCHANGED` | 654 | 与 v1.4.4 上游逐字节一致 |
| `ADDED` | 4 | 本项目新增，不属于上游原包 |
| `CHANGED` | 0 | 没有修改上游文件 |
| `REMOVED` | 0 | 没有删除上游文件 |

因此，当前资源包中没有发现第二个类似 NadeSystem 的“旧定制 DLL 直接覆盖 v1.4.4 官方组件”问题。

## 当前相对 v1.4.4 的差异

### ADDED

- `addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`
  - MapRotation 的外置可变默认配置。
  - 在插件 marker 的 `mutableConfigEntries` 中声明，不参与固定 payload 摘要。
- `addons/counterstrikesharp/plugins/MapRotation/MapRotation.dll`
  - 本项目重新编译的 MapRotation 插件。
- `addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json`
  - 本项目的版本、来源和 payload 门禁 marker。
  - `version=0.5.11`，组件来源声明为 `CS2-Bot-Improver-v1.4.4`。
- `gameinfo.manifest.json`
  - 三份 gameinfo 资源的文件大小和 SHA-256 manifest。

### CHANGED / REMOVED

没有。`gameinfo.gi`、`backup/Online/gameinfo.gi`、`backup/WithBots/gameinfo.gi` 以及所有上游插件、配置、VPK、Panel 文件均保持 v1.4.4 上游字节。

## 本次已修复的历史覆盖风险

此前构建输入中的 `addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll` 是旧定制源码编译的 `ModuleVersion=1.1.7`，它替代了 v1.4.4 官方 NadeSystem `1.2.1`，导致上游新增的 BOT 投掷手雷 radio、sound、chat 逻辑没有进入安装包。

现已恢复 v1.4.4 官方 NadeSystem 的三个配套文件：

- `NadeSystem.dll`
- `NadeSystem.deps.json`
- `NadeSystem.pdb`

官方 DLL SHA-256：`9E4FC0CFD6B78D67C5EAEC86F76C76A7ACCC59820FCA3B02CBC517451D4074E4`。

这次恢复不是对 v1.4.4 文件的二次修改，而是将错误的旧覆盖移除，回到唯一允许的新基线。

## 本项目功能边界

- MapRotation、BotVision、面板和安装/回滚逻辑属于项目代码或新增资源；它们没有在当前 v1.4.4 上游 ZIP 文件中留下覆盖式改动。
- `gameinfo.gi` 保留上游内容；BOT 变体仍通过上游包内既有 `backup/WithBots/gameinfo.gi` 提供。
- VPKEdit 隐藏、自动换图默认状态隐藏等行为由应用/UI 代码控制，不通过篡改上游 ZIP 文件实现。

## 本次测试安装程序

- 文件：`artifacts/upstream-v1.4.4-migration-0.5.11-20260904/CS2人机增强助手_0.5.11_x64-setup-nadesystem-v1.4.4.exe`
- SHA-256：`AEEFDCF9022DE44051AC16136BECDFAFD8CF6955491BF862E4BFFC6FC70FB302`
- 用途：本机 BOT/在线模式功能测试，未做真实游戏内 radio、sound、chat 验收。
- 构建日志提示未提供 Tauri 私钥；按本次要求不影响本机测试使用。

