# 0.5.6 正式发布执行交接方案

> 交接对象：下一位【实际执行 AI】。两个 AI 不共享上下文；本文包含执行所需的路径、版本、证据、命令、远端边界和停止条件。
>
> 用户已授权正式发布、上传、线上写入和部署，但本轮角色是【方案制定 AI】，只调查并写方案，不执行 commit、tag、push、GitHub Release、夸克、R2/D1/feed 或官网部署。

## 1. 目标和成功定义

将已经通过本地 Gate A/B/C 的 0.5.6 候选发布为公开版本，并让 GitHub、夸克、R2/D1、updater feed 和旧版客户端形成同一份可回读的发布记录。正式成功必须同时满足：

- 源码有包含本轮功能的最终 commit，并创建 v0.5.6 tag。
- GitHub 仓库 YuGeYu/CS2AS05 的 Release 和附件来自同一最终 commit/候选文件。
- 夸克分享链接、官网 D1 记录、R2 对象和 feed 元数据与最终 EXE 的 size/SHA-256/signature 完全一致。
- 从公开 feed 用真实 0.5.5 客户端完成下载、签名校验、安装和重启。
- 发布说明按“新增、修改、修复、其他”四个大类组织 0.5.5 到 0.5.6 的全部变更；每个大类可包含任意数量的条目，并诚实保留多次加时未验证和 Authenticode NotSigned 限制。

任一步未完成，结果只能写“发布未完成”或“部分发布”，不能写“0.5.6 正式发布”。

## 2. 已调查基线

项目目录：E:\CS2AS05；调查时分支为 main，HEAD 为 8552b554993fec66866d2133d315342be3e0cbca。恢复执行前必须重新读取：

PowerShell 命令：
Set-Location E:\CS2AS05
git status --short --branch
git rev-parse HEAD
git diff --stat
git diff --check
git remote -v

工作树很脏，用户已有改动全部保留。不得使用 git reset --hard、git checkout -- 或 git clean。提交前逐项审阅完整 status，只暂存属于 0.5.6 的源码、测试、资源、许可证、文档和发布说明；不得暂存 workspace\release-evidence、target、*.dem、SQLite、日志、截图、临时副本、私钥或 token。

本地交付目录：

E:\CS2AS05\dist-release\cs2-bot-improver\0.5.6

当前最终候选（来自同一次重建）：

- CS2人机增强助手_0.5.6_x64-setup.exe：112,414,780 bytes，SHA-256 590AC0D6A4890B7366463A7E2A0829725A0C35D4D5155EF06BC7A5A9036BF261，Authenticode NotSigned。
- CS2人机增强助手_0.5.6_x64-setup.exe.sig：436 bytes，SHA-256 723BDF9CD3FEC3CFE5DF855E218238EB5AA603E6D9548BDDD6813FE65F07C690。
- 同目录已有 .exe.sha256、SHA256SUMS.txt、updater-prod.json、LICENSE、NOTICE.md、release-notes-0.5.6.md。
- updater-prod.json 已确认 version=0.5.6、channel=prod、projectId=cs2-bot-improver，installer 为 basename，不含本机绝对路径，size/hash/signature 与候选一致。

一致性证据：

E:\CS2AS05\workspace\release-evidence\0.5.6-artifact-rebuild-20260805-193244\artifact-consistency.json

旧同名候选已隔离到该证据目录下的 old-0.5.6，不可复用、不可删除。

### 已确认的 Gate

执行报告为：

E:\CS2AS05\docs\demo-play-reveal-postmatch-scoreboard-0.5.6-final-verification-execution-report-20260805.md

Gate A/B/C 已记录通过：Vitest 35 files/127 tests 全部通过；Rust library 62 passed/0 failed/3 ignored；cargo check、clippy、typecheck、lint、web build、format、diff check 通过；Explorer 特殊路径、真实 CS2 Demo 播放、原文件 fingerprint、三回合 BOT 新局、最新战报、回放 suppression、干净安装、0.5.5 覆盖升级、卸载重装和旧版 updater 已由用户确认通过。

仍必须公开声明：没有多次加时真实 Demo，不能声称多次加时已实证；Windows Authenticode 为 NotSigned；BotVision 上游再分发条款按 NOTICE.md 的人工复核边界处理。

## 3. 更新日志必须先修订

在最终 commit 前修改 E:\CS2AS05\docs\release-notes-0.5.6.md，并同步复制到交付目录。使用以下四个大类；每类可以包含多条，所有条目都必须可由源码、执行报告或许可证材料证明：

- 新增：Demo 录像库增加 CS2 播放；增加 Explorer 文件定位；增加逐行 busy、按钮禁用态、tooltip 和 ARIA 名称；增加基于最新 Demo 的独立赛后计分板；增加回放 session suppression；增加地图资源、双向对枪矩阵和 BOT/观战身份展示。
- 修改：Demo 会话由统一 coordinator 管理；比分改按已完成回合的规范化 winner 累计；报告展示改为 schema/adapter/metrics、core job、玩家、回合和 scoreboard 状态联合门禁；simple-rating-v1 保持多因子公式；窄窗口录像表格改为容器横向滚动。
- 修复：修复 CS2 session 与 Demo 异步解析竞态；修复旧 Demo 或 parsing 中 Demo 被误选为最新战报；修复未完成、损坏、零回合、空玩家和版本过期报告打开空计分板；修复回放退出误触发赛后战报；修复外部 Demo 播放可能改写原文件的问题；修复 Explorer 特殊路径参数处理风险。
- 其他：并行集成 BotVision 0.2.2；补充第三方来源、许可证和 NOTICE；明确播放前必须退出运行中的 CS2、旧 Demo 兼容性限制、Authenticode NotSigned 状态；说明多次加时比分累计路径已具备但真实样本尚未验证。

不要把多次加时写成已验证；可写“比分累计路径支持多段加时，真实样本待补”。保留四个大类的标题和条目结构，不要把所有变更压缩成单条。保留 CS2-insight-agent 固定 commit 和 PolyForm Noncommercial“仅行为参考、未复制源码/测试/样式/资产”的 NOTICE 边界。

## 4. 源码提交和 GitHub Release

重新读取 git status，确认没有私密文件。结构化读取 manifest，断言 version/channel/project、installer basename、size/hash/signature；用 Get-FileHash 和 Get-AuthenticodeSignature 重新记录。确认密钥环境变量为空，绝不输出私钥或 DPAPI 密码。

BotVision 再分发许可证人工复核必须留证；不能确认条款则停止公开发布。多次加时未验证不是阻塞，但必须写进 release notes 和 Release body。

只暂存 0.5.6 相关源码、测试、资源、文档，执行 git diff --check 和 staged diff 审阅。创建最终 commit（建议 release: prepare CS2AS 0.5.6），创建 v0.5.6 tag 指向该 commit。tag 前执行：
git fetch --tags origin
git ls-remote --tags origin refs/tags/v0.5.6
gh release view v0.5.6 --repo YuGeYu/CS2AS05 --json url,isDraft,isPrerelease,publishedAt,assets

远端已有不同 v0.5.6 或 feed 已提供另一 hash 时立即停止，不能覆盖，改更高 SemVer 并重新确认。

推送最终分支和 tag 只允许一次，不 force；GitHub Release 上传只允许一次完整尝试。Release 标题为 CS2 人机增强助手 v0.5.6，附件使用同一交付目录的 EXE、.sig、.sha256 和 SHA256SUMS.txt。不得上传私钥、DPAPI、evidence、SQLite、Demo 或含本机路径的 manifest。

## 5. 夸克、R2、D1 和 feed

官网仓库为 E:\cs2as，调查时分支 codex/all-command-library、HEAD 5eb0263，存在用户改动和未跟踪 evidence；不得清理、回退或顺手提交。生产元数据通过已有 scripts/publish-self-update.mjs 写入，不要求 push 官网仓库。

夸克链接必须是 https://pan.quark.cn/ 域名下的用户分享链接；没有可用分享链接时停止，不伪造。先 dry-run，再唯一一次 remote。命令中的 <夸克分享链接> 必须替换为真实链接：

Set-Location E:\cs2as
npm run self-update:publish -- --installer "E:\CS2AS05\dist-release\cs2-bot-improver\0.5.6\CS2人机增强助手_0.5.6_x64-setup.exe" --sig "E:\CS2AS05\dist-release\cs2-bot-improver\0.5.6\CS2人机增强助手_0.5.6_x64-setup.exe.sig" --version 0.5.6 --quark-url "<夸克分享链接>" --title "CS2 人机增强助手 v0.5.6" --summary "Demo 播放、文件定位与解析成功后自动战报" --items "新增 Demo 播放与文件定位|修改赛后战报和比分统计|修复异步解析竞态与回放误弹|其他：BotVision 与许可证说明"
npm run self-update:publish -- --remote --installer "E:\CS2AS05\dist-release\cs2-bot-improver\0.5.6\CS2人机增强助手_0.5.6_x64-setup.exe" --sig "E:\CS2AS05\dist-release\cs2-bot-improver\0.5.6\CS2人机增强助手_0.5.6_x64-setup.exe.sig" --version 0.5.6 --quark-url "<夸克分享链接>" --title "CS2 人机增强助手 v0.5.6" --summary "Demo 播放、文件定位与解析成功后自动战报" --items "新增 Demo 播放与文件定位|修改赛后战报和比分统计|修复异步解析竞态与回放误弹|其他：BotVision 与许可证说明"

脚本顺序为 R2 上传、R2 GET 回读、size/hash 比对、D1 disabled upsert、确认 feed 未提前变化、最后 enable updater。失败时先将 release 的 updater_enabled=0，保留 R2 对象，不覆盖旧 key。

远端回读保存到 E:\cs2as\release-evidence\self-update-0.5.6-<timestamp>：

- R2 GET 完整下载 size/SHA-256 等于 590AC0D6A4890B7366463A7E2A0829725A0C35D4D5155EF06BC7A5A9036BF261。
- D1 version/project/channel、R2 key、signature、sha256、size、夸克 URL 一致。
- updater feed 对 0.5.5 返回 200/version 0.5.6；对 0.5.6 返回 204。
- download URL 完整 GET 回读 EXE，记录 status、content-length、hash、CORS/cache。
- 旧版客户端 updater 下载、验签、安装、重启成功。

## 6. 官网部署和发布后回滚

若 Worker 已有正确路由，优先只写 R2/D1，不改无关页面。确需部署时最多尝试 3 次，每次保存 deploy 输出和公开 URL 回读。发布后打开 GitHub Release，确认附件和 Release notes；用公开 URL 下载并核对 hash；用 0.5.5 客户端走 updater，0.5.6 客户端检查 feed 204。

任一线上 hash/size/signature 不一致、feed 提前暴露半成品、旧版验签失败、R2/D1 部分成功或 Release 上传失败：

1. 将 D1 release 的 updater_enabled=0，必要时关闭全局开关。
2. 保留 R2、D1 和 evidence 审计数据，不覆盖同 key。
3. 恢复上一已验证版本 feed。
4. 修复后发布更高版本，禁止同版本覆盖。

## 7. 最终回报格式

结果：未完成 / 部分发布 / 已正式发布
Git：branch、最终 commit、tag、push、GitHub Release URL
更新日志：新增/修改/修复/其他四类内容及文件路径
本地候选：EXE/.sig/hash/manifest 路径、size、SHA-256、Authenticode
夸克：分享链接及回读状态
R2/D1/feed：key、size/hash/signature、0.5.5 -> 0.5.6、0.5.6 -> 204
公开回读：GitHub 资产、下载 URL、CORS/cache、完整 GET hash
旧版 updater：下载、验签、安装、重启结果
未验证：多次加时真实 Demo、BotVision 许可证或其他缺口
回滚：是否触发、当前线上开关和 feed 状态

只有源码、GitHub、夸克、R2/D1/feed、公开下载回读和旧版 updater 全部闭环后，才报告“已正式发布”。

## 8. 当前交接结论

本地 0.5.6 候选和 Gate A/B/C 已由用户确认通过，但多次加时真实 Demo 尚未提供，Authenticode 为 NotSigned，源码最终 commit/tag、GitHub、夸克、R2/D1/feed 和官网公开回读尚未完成。本轮只新增本方案，不执行任何发布动作。
