# BOT 难度资源包修复记录（2026-09-29）

## 现象

部分用户在新版本启动台中切换 BOT 难度或启动 BOT 模式时收到：

`[PANEL_IO] 读取难度文件失败：...\\game\\csgo\\overrides\\Low\\botprofile.vpk；系统找不到指定的文件（os error 2）`

## 根因

当前 `src-tauri/resources/CS2BotImprover.zip` 只有三个 `botprofile.db`，缺少面板和启动流程要求的四个 VPK：

- `overrides/Low/botprofile.vpk`
- `overrides/Medium/botprofile.vpk`
- `overrides/High/botprofile.vpk`
- `overrides/botprofile.vpk`

安装流程会按资源包内容复制文件，因此新安装/自动修复后这些路径不会出现，界面仍会显示环境准备中，切换难度直接失败。

## 修复

从现有已审计的 v1.4.4 资源阶段补回四个 VPK，并重新打包 `CS2BotImprover.zip`。同时增加 `gameinfo-upstream-contract.spec.ts` 契约测试，要求四个条目都存在且非空。

资源包修复后 SHA-256：`F2E754FDA5D30EB8EBBD814CE88174D4D0A8BA05A1EDFCD3BADAEF116CB1A0B4`

## 验证边界

- 已验证 ZIP 内四个条目均可列出并读取。
- 尚未在用户的 D: Steam 安装目录执行真实安装/自动修复；用户需要使用新构建重新安装或触发一次 BOT 插件自动修复后，再切换 Low/Medium/High。
