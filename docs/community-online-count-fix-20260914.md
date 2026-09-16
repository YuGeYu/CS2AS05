# 玩家圈子在线人数修复记录（2026-09-14）

## 问题

只有站长账号使用开发中助手时，“玩家圈子 → 聊天大厅”显示 2 人在线。排查确认上海服务器运行的旧版 `community-server/src/server.mjs` 按 WebSocket 连接数统计；同一账号多窗口会被误计为多人。

## 修复

- `community-server/src/server.mjs` 的 `distinctMemberCount(members)` 现在只统计仍为 `OPEN` 的连接。
- `userId/sub` 在统计前统一 `String().trim()`，再按账号去重。
- 已通过 SFTP 同步到上海服务器 `/opt/cs2as-community/src/server.mjs`，并重启 `cs2as-community` 容器。

## 生产回读

- 容器：`cs2as-community`，重启后状态 `Up`。
- 容器内 `node --check /app/src/server.mjs` 通过。
- `http://127.0.0.1:8787/health` 返回 `{"service":"cs2as-community","ok":true}`。
- `http://8.133.185.37/community/health` 返回 `{"service":"cs2as-community","ok":true}`。

## 边界

本次只修复在线人数统计并重启社区容器；未改聊天消息、账号令牌、QQ 机器人或其他 API。前端真实界面最终应由用户重新打开助手进入聊天大厅确认显示为“1 人在线”。
