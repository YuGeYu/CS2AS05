# 安装与诊断支持中心执行报告

日期：2026-08-13  
版本：0.5.7（未增加版本号）

## 客户端

- 重构“安装与诊断”页面的信息层级和响应式布局，保留原有安装、覆盖更新、卸载、原版 Panel、更新与来源能力。
- 新增当前用户级 Windows 开机启动开关，不请求管理员权限。
- 新增“清除数据”，仅清理助手缓存、日志和 WebView 界面偏好；不删除 CS2 文件、插件、Demo 或复盘数据库。
- 新增故障工单表单。正文限制 10-4000 字符，附带日志在客户端先裁剪并隐藏用户目录、密码、令牌、私钥和授权头相关行。
- 故障表单明确提示不会上传 Demo、游戏文件或皮肤配置。

## 官网

- 仓库：`E:\cs2as`
- 新增 D1 表 `client_fault_reports`、匿名客户端提交接口和管理员工单管理接口。
- 客户端来源限定为 Tauri 与官网；同一匿名来源每小时最多 5 单；日志最大 32000 字符。
- 后台刷新由 30 秒固定轮询改为首次加载、手动同步、页面恢复可见或重新聚焦且数据超过 60 秒时同步；操作后只刷新对应区域。
- 远程迁移 `0008_client_fault_reports.sql` 已应用。
- Cloudflare Worker 已部署，生产版本 ID：`3c6f256a-b19c-4c20-940b-e42e1fdceedb`。
- 生产探针：OPTIONS 204；无效正文 POST 400；两者均返回 `Access-Control-Allow-Origin: http://tauri.localhost`。

## 验证

- 客户端 `npm run verify` 通过：TypeScript、Oxlint 0/0、ESLint、48 个测试文件 / 160 项测试、生产构建。
- Rust `cargo check` 通过；支持服务测试 2/2 通过。第三方 demoparser 的既有警告未在本次处理。
- 官网 TypeScript 和生产构建通过；4 个测试文件 / 14 项测试通过。
- 内置浏览器实景检查确认支持区无横向越界，故障模态框布局正常，控制台 0 warning / 0 error。

## 安装程序

- 路径：`E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.7_x64-setup.exe`
- 大小：114,536,279 bytes（109.23 MiB）
- SHA-256：`302C150F606C2826C0A6FF0A599F4486171B34CB5DC3D3CEF662A664AD822087`
- Authenticode：`NotSigned`
- NSIS 已成功生成；之后的 updater 签名阶段因没有 `TAURI_SIGNING_PRIVATE_KEY` 返回非零，不影响 NSIS 安装程序。
