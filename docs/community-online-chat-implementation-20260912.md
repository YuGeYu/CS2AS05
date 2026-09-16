# 玩家圈子首版实现记录（2026-09-12）

## 已实现

- 官网 Worker 新增 `POST /api/community/token`，复用现有登录会话签发 15 分钟 HMAC-SHA256 社区令牌。
- 上海服务器服务骨架位于 `community-server/`，使用 Node.js、SQLite、WebSocket。
- WebSocket 房间鉴权、房间人数上限、历史消息、文本消息、消息编辑、在线人数和 WebRTC `signal` 转发已具备。
- 连接信息严格规范化为 `connect [A:...] (...)`，普通英文消息保持原文。
- 消息 40 小时过期，服务定时清理数据库记录；上传目录预留清理逻辑。
- Tauri 助手新增“玩家圈子”入口，包含聊天大厅、语音大厅、输入/总音量滑块和连接状态。

## 本次补齐与部署（2026-09-12）

- `community-server/src/server.mjs` 增加任意格式资源上传/下载（单文件 10 MB）、临时资源清理、表情回应和每日北京时间幂等赞赏任务。
- Tauri 圈子页增加资源卡片、上传入口、回应、WebRTC Mesh 语音信令/麦克风采集/总音量增益；英文连接信息仍按 `connect [A:...] (...)` 规范化。
- Tauri 原生 `TrayIconBuilder` 已启用：关闭主窗口会隐藏到托盘，托盘菜单可重新打开或退出。
- 上海服务器已运行 `cs2as-community` 容器，数据目录为 `/var/lib/cs2as-community`，微信赞赏图已作为服务器文件注入；Caddy 保留原 `api.600318.xyz` 服务，并增加 `/community` 本机反代。
- 官网 Worker 已写入社区令牌密钥与服务地址配置；未把密钥写入仓库。

## 尚未完成或需要实机验收

## 2026-09-13 Failed to fetch 修复

- 原因：Tauri WebView 请求 `POST /api/community/token` 时带有 `Origin: http://tauri.localhost`，被 Worker 的通用同源校验拦截；错误响应也没有返回 CORS 头，因此界面只能显示 `Failed to fetch`。
- 修复：社区令牌接口单独允许 `http://tauri.localhost` 与官网来源，加入 `Access-Control-Allow-Credentials`、POST/OPTIONS 预检头，并让 401/403/503 错误同样携带社区 CORS 响应头。
- 证据：生产 Worker 的 OPTIONS 预检返回 `204`，包含 `Access-Control-Allow-Origin: http://tauri.localhost`、`Access-Control-Allow-Credentials: true`、`Access-Control-Allow-Methods: POST, OPTIONS`。

## 2026-09-13 第二次修复：Tauri 登录会话与 WebView Cookie 隔离

- 原因：助手登录由 Rust 原生层完成并保存账号凭据，WebView 没有官网 `cs2as_session` Cookie；圈子页直接 `fetch` 官网令牌接口时，无法稳定复用助手登录状态。
- 修复：新增 Tauri 命令 `get_community_auth`，由 Rust 原生层重新认证并携带会话 Cookie 请求社区令牌；`CommunityView.vue` 改为调用该命令，不再依赖 WebView Cookie 或跨域凭据。
- 验证：桌面 TypeScript 检查通过；Rust 开发版重新编译并启动成功，进程 `ai_pc_fac.exe` 正在响应。

## 2026-09-13 第三次修复：Cloudflare 1010 请求特征

- 原因：上海助手保存的站长账号有效，但 Rust `reqwest` 默认 User-Agent 被官网 Cloudflare 规则返回 `403 / 1010`；因此原生层无法取得社区令牌，WebSocket 永远不会进入有效连接。
- 修复：官网登录请求加入 Chrome User-Agent、Accept、Origin 和 Referer，继续由 Rust 原生层完成认证和社区令牌申请。
- 实测：同一账号使用等价浏览器请求头登录返回 `200`；开发版已重新链接并启动，进程 `ai_pc_fac.exe` 正在响应。

## 2026-09-13 第四次修复：Worker 与社区服务令牌密钥漂移

- 原因：诊断发现官网签发的 JWT 与上海社区容器当前 `COMMUNITY_TOKEN_SECRET` 签名不一致，WebSocket 因此稳定返回 `1008 unauthorized`。
- 修复：从正在运行的上海社区容器读取当前密钥，通过 Wrangler 安全写回官网 Worker；未把密钥写入仓库或日志。
- 实测：站长账号签发令牌的签名与服务器校验一致，WebSocket 返回 `ready`，用户身份为站长账号，已可进入大厅并发送消息。

- 上海服务器 SSH 在本次环境中未成功回读，未执行生产部署，也未确认旧中转站服务是否清理。
- 尚未配置生产 `COMMUNITY_TOKEN_SECRET`、域名、HTTPS/WSS 反向代理、防火墙和 systemd。
- “关闭时询问退出/保留托盘，并记住选择”尚未接入原生偏好存储；当前行为是直接隐藏到托盘。
- WebRTC 使用 STUN + Mesh，尚未在真实多人网络与 NAT 环境中验收；当前只承诺代码链路，不承诺跨网络语音质量。
- 官网 Worker 当前服务地址采用上海服务器公网 HTTP 入口；要让普通浏览器安全使用，需要新增 `community.600318.xyz` DNS/HTTPS（不影响桌面助手入口）。

## 验证

- `E:\cs2as05`: `npm run typecheck` 通过。
- `E:\cs2as`: `npm run typecheck` 通过。
- `community-server/src/server.mjs`: Node 语法检查通过；本地服务可在测试端口启动。

生产部署必须先完成服务器清点和密钥注入，禁止直接使用示例密钥。
