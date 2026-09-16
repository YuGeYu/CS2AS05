# 版本整理执行记录（2026-09-11）

## 执行范围
本次只整理文件职责和发布文档，不删除历史版本，不修改业务源码，不改变版本号。

## 已执行
- 将根目录 CS2人机增强助手_0.5.13_x64-setup.exe 及同名 .sig 移入 elease-artifacts/candidates/0.5.13/。
- 对 elease-notes-0.5.13.md 去重，保留四个发布分类的单一主稿。
- 新增 docs/release-artifacts-and-versioning-rules.md，规定开发、候选、已发布、归档四种状态和迁移门禁。
- 新增候选目录 README 与 CHECKSUMS，记录当前文件 hash/size。
- 将 elease-artifacts 加入 .gitignore，避免大二进制污染源码仓库；规矩和日志仍以 docs/、根目录主稿维护。

## 未执行
- 未删除 dist/、dist-release/、rtifacts/ 或旧版本；它们可能包含验收证据，后续按归档任务逐项确认后再迁移。
- 未把候选复制到 published/；GitHub、官网、R2、夸克和 updater 回读未在本次操作中重新验证。

## 规则补充（2026-09-11）
- 已将 0.5.13 主日志固定至 `E:\CS2AS05\docs\releases\release-notes-0.5.13.md`。
- 后续日志只允许在 `docs/releases/` 下按版本号维护；允许编辑/追加，不允许无记录重写。
