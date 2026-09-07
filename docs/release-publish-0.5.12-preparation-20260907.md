# 0.5.12 发布准备记录

日期：2026-09-07

## 版本状态

- 当前版本：`0.5.12`
- 上一正式版本：`0.5.11`
- 当前状态：已完成本地签名安装器、Tauri updater `.sig`、本地 updater manifest 和交付目录准备。
- GitHub、R2、官网 updater、夸克渠道尚未在本记录中宣称已发布；这些属于后续公开发布动作。

## 本次实际差异

- 标题栏增加应用语义、当前页面上下文、控制分组和跳转到主要内容入口。
- 导航改为核心工作台、BOT 工具、复盘与扩展、支持与维护四组，支持桌面侧栏收起/展开和响应式短标签。
- 安装诊断集中反馈目录、CS2 运行状态、资源环境和当前操作。
- BOT 强度工坊增加工具状态缓存、提取工作目录隔离、并发保护和打开档案前的 CS2 退出检查。
- 上游资源继续使用 CS2-Bot-Improver v1.4.4、Panel v1.4.4 和 NadeSystem v1.4.4 链路；队伍预设为 42 支。

## 签名构建

- 私钥来源：`C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key`
- 密码来源：同目录 `updater-password.dpapi`
- DPAPI 内容读取后执行 `.Trim()`，再传入 `ConvertTo-SecureString`；未使用带换行的原始内容。
- 私钥内容注入 `TAURI_SIGNING_PRIVATE_KEY`；密码注入 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。
- 未依赖单独的 `TAURI_SIGNING_PRIVATE_KEY_PATH` 触发签名。
- 构建完成后已清理两个签名环境变量；项目和交付目录未写入私钥、密码或 DPAPI 明文。
- updater 公钥仍使用项目 `src-tauri/tauri.conf.json` 中与既有发布线一致的公钥。

## 本地交付产物

目录：`E:\CS2AS05\artifacts\release-0.5.12-signed-20260907\`

| 文件 | 大小 | SHA-256 |
| --- | ---: | --- |
| `CS2人机增强助手_0.5.12_x64-setup.exe` | 117,828,112 bytes | `D08D09203D14ACEBAB2050AFFAB57101AA14C7EA98571B5041405C57329FB2C0` |
| `CS2人机增强助手_0.5.12_x64-setup.exe.sig` | 436 bytes | `ED23A277CDA56481835EA27E83AE4B383BB75BCF2DC8932591627029BB0343BB` |
| `updater-prod.json` | 859 bytes | 由本次 manifest 生成 |

安装器路径：`E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.12_x64-setup.exe`

签名路径：`E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.12_x64-setup.exe.sig`

Windows Authenticode 回读为 `NotSigned`。这与 Tauri updater `.sig` 是两套独立机制，不能把其中一项冒充另一项。

## 自动化验证

- `npm run typecheck`：通过。
- `npm run build:web`：通过。
- `npm run bundle:desktop`：通过，生成 0.5.12 NSIS 安装器和 updater `.sig`。
- 定向 Vitest：6 个测试文件、22 项通过，覆盖 BOT 强度工坊、标题栏、品牌、外观偏好、安装器契约和 Panel 资源 fixture。
- `npm run release:manifest`：通过，manifest 的版本、安装器文件名、SHA-256、大小和签名已对应本次构建。
- 资源 marker 已重建为 `version=0.5.12`，来源为 `CS2-Bot-Improver-v1.4.4`。

## 发布边界

- 本记录只证明本机最终构建、签名产物和本地 manifest 已准备完成。
- 尚未执行 R2 上传、R2 下载回读、官网 updater 启用、GitHub Release 上传或公开渠道发布。
- 尚未把 Windows Authenticode `NotSigned` 解释为 Tauri updater 签名失败。
- BOT 反应、投掷和插件行为仍需真实 CS2 BOT 环境验收，自动化检查不替代玩家设备验收。
