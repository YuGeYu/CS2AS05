# 0.5.8 公告同步、官网部署与安装程序报告

日期：2026-08-16

## 结果

- 助手顶栏新增公告按钮；默认不打开，最新公告为“警告”或“错误”时启动自动打开。
- 助手从 `https://cs2as.600318.xyz/api/site-notices?scope=idea` 同步全部已发布意见页公告。
- 官网支持多公告、成功/警告/错误三档强度、新增、修改、删除、分钟级发布日期和未来定时发布。
- 官网 Worker 已部署，版本：`c5939cef-1584-4841-a57a-bbbe6067adb6`。
- D1 `cs2asd1` 已应用 `0010_multi_site_notices.sql`；应用后无待迁移项。
- Windows x64 NSIS 安装程序已重新构建并生成 Tauri updater 签名。

## 生产证据

- D1 迁移前备份：`E:\cs2as\release-evidence\announcements-0.5.8-20260816-165809\cs2asd1-before-0010.sql`
- 线上 API 回读：`E:\cs2as\release-evidence\announcements-0.5.8-20260816-165809\online-site-notices-response.json`
- 线上 API 响应头：`E:\cs2as\release-evidence\announcements-0.5.8-20260816-165809\online-api-headers.txt`
- 线上迁移回读：`E:\cs2as\release-evidence\announcements-0.5.8-20260816-165809\remote-migrations-after.txt`
- 管理后台编辑模态框截图：`E:\cs2as\release-evidence\announcements-0.5.8-20260816-165809\admin-edit-modal-1440x900.png`

线上 API 回读为 HTTP 200，`Access-Control-Allow-Origin: *`，返回 `latest/notices`，当前线上公告为 `success`，已过滤未到发布时间的内容。真实浏览器检查覆盖 1440x900 和 375x812，均无横向溢出；后台已确认公告列表、添加模态框、三档强度、分钟级未来日期提示和修改模态框。验收期间没有保存、新增或删除生产公告。

## 安装程序

- 安装程序：`E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.8_x64-setup.exe`
- 大小：`114636008` bytes
- SHA-256：`67A669FF710D393A1E24B6E47752992A1172A1BCB90A963B88212F70EFA9B087`
- updater 签名：`E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.8_x64-setup.exe.sig`
- `.sig` 大小：`436` bytes
- `.sig` SHA-256：`8FAFA4E64C8EC7CFEBEBEF5727448E91ECDEB1F1C5C83BB4D3F81008E42EB92B`
- Tauri updater 公钥与 `tauri.conf.json` 一致。
- Windows Authenticode：`NotSigned`；这不等同于 Tauri updater `.sig`。
- 未安装、未上传、未提交、未推送安装程序。

## 检查命令

- 助手公告定向测试：`4/4 passed`（此前完成）
- 官网 Worker 完整测试：`8 files / 26 tests passed`（此前完成）
- 助手 `npm run typecheck`：exit `0`
- 公告文件 Oxlint：exit `0`
- 公告文件 ESLint：exit `0`
- 助手 `npm run build:web`：exit `0`
- 官网 `npm run build`：exit `0`
- `npx wrangler d1 migrations apply cs2asd1 --remote`：exit `0`
- `npm run deploy`：exit `0`
- `npm run bundle:desktop`：exit `0`（第二次，签名成功）

完整助手测试仍为 `171/173 passed`；两个失败是既有字符串契约漂移：`installer-contract.spec.ts` 的旧清理列表断言，以及 `ui-design-contract.spec.ts` 的单引号 import 断言，均与公告功能无关，未为通过旧断言回退现有功能。完整全局 lint 仍受既有 `artifacts/thank-you-video/edge-profile-*` 生成文件影响。

## 上游与发布边界

- 本次公告功能未复制 Rock-Radar 源码，不新增 NOTICE；表现雷达上游引用仍按既有报告记录 MIT 固定提交。
- 未执行 Git commit、push、GitHub/R2 上传或正式发布；仅按用户授权部署官网并构建本地安装程序。
