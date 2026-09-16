# 聊天大厅文本/视频下载响应头修复（2026-09-14）

## 根因

下载路由原先把上传文件名直接写入 `Content-Disposition`。文本和视频文件常带中文、引号或控制字符，Node 在写入 HTTP 响应头时会抛出 `Invalid character in header content ["content-disposition"]`；图片通常以内联预览方式打开，因此没有触发下载头问题。

## 修复

- 新增 `contentDisposition()`：过滤换行、控制字符、引号和反斜杠。
- 下载响应同时提供 ASCII `filename` 兜底与 RFC 5987 UTF-8 `filename*`，保留中文文件名显示能力。
- 非下载的图片/视频 inline 预览不再发送 attachment 头，保持原行为。

## 生产部署与验证

- 已同步 `community-server/src/server.mjs` 到上海服务器 `/opt/cs2as-community/src/server.mjs`。
- 已重启 `cs2as-community` 容器，状态为 `Up`。
- 容器内 `node --check /app/src/server.mjs` 通过。
- `http://127.0.0.1:8787/health` 返回 `{"service":"cs2as-community","ok":true}`。

用户可重新下载之前上传的文本和视频文件；原消息链接无需重新上传。
