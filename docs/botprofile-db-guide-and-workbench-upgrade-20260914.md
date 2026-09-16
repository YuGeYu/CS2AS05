# 人机强度工坊说明页与编辑区升级（2026-09-14）

## 变更

- 人机强度工坊模态框由约 960px 扩大到 1240px，内容编辑区最小高度由 330px 提升到 520px，减少长篇 `botprofile.db` 编辑时的频繁滚动。
- 新增玩家可读的 `botprofile.db` 说明页，整理 Skill、Aggression、ReactionTime、AttackDelay、Teamwork、AimFocus、武器偏好、地图 nav 和 `bt_config.kv3` / `bt_default.kv3` 的作用边界。
- 说明明确：工坊只服务本地 BOT / `-insecure` 对局，不修改 Online 模式官方文件；保存仍经过回写校验并保留备份。
- 说明材料参考用户提供的小黑盒帖子《如何让CSGO的BOT更加强力（保姆级教程）》，并标注经验内容需以当前 CS2 版本实测为准。

## 首次阅读闸门

- 首次进入工坊时显示说明页，关闭按钮、遮罩关闭和 Escape 均暂时不可用。
- 阅读计时至少 2 秒，倒计时完成后才能点击“我已阅读，进入工坊”。
- 阅读完成状态保存到 Tauri `app_local_data_dir` 的 `onboarding-botprofile-db-v0.5.14.done`，不使用 `localStorage`；后续进入不再强制显示。

## 验证

- `npm run typecheck` 通过。
- `cargo check --manifest-path src-tauri/Cargo.toml` 通过（保留既有第三方依赖警告）。
