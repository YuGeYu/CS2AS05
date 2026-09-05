# 自动换图默认开关与 0.5.10 未签名安装器执行报告

日期：2026-08-29
状态：未签名功能验证候选，待用户验收

## 变更

- 新增 `src-tauri/src/services/map_rotation.rs`：固定路径 `game/csgo/addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json` 的读取、JSON 校验、原子写入和 fallback 状态。
- 新增 `src-tauri/src/commands/map_rotation.rs`，注册 IPC：`get_map_rotation_default`、`set_map_rotation_default`、`reset_map_rotation_default`。
- 新增 `src/types/mapRotation.ts`、`src/services/tauri/mapRotation.ts`、`src/components/MapRotationDefaultControl.vue`。
- 仅在 `src/views/OverviewView.vue` 增加“自动换图默认状态”入口；未在安装与诊断页增加入口。
- CS2 运行中由 Rust 返回 `[MAP_ROTATION_CS2_RUNNING]`，不写文件；损坏 JSON 只 fallback 开启，点击恢复按钮才写入 `{"enabled":true}`。

## 资源证据

- ZIP：`E:\CS2AS05\src-tauri\resources\CS2BotImprover.zip`
- ZIP SHA-256：`FC8868ABD46056DA52540EB14F3BA0B1B36005928C44178BC479C3BD2A4AA64E`
- marker：`addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json`
- marker version：`0.5.10`
- payload SHA-256：`76D9C84D74368DF6727435EAE0C830C7F75056E1E1F6E0AB14C128CD43DC9EF7`
- MapRotation DLL 和默认 JSON 均存在于 ZIP；本轮未修改插件源码，未重编 DLL。

## 验证

- `npm test -- --run tests/map-rotation-contract.spec.ts tests/map-rotation-settings.spec.ts`：现有 `map-rotation-contract.spec.ts` 2 passed；仓库当前没有 `map-rotation-settings.spec.ts`，因此组件级覆盖仍待补。
- `npm run typecheck`：通过。
- `npm run build:web`：通过。
- `cargo test --manifest-path .\\src-tauri\\Cargo.toml map_rotation`：编译通过，过滤结果 0 tests；尚未新增 Rust service 专项用例。

## 安装器

- 生成路径：`E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.10_x64-setup.exe`
- 大小：`117210964` bytes
- SHA-256：`DBBE3FCC51869691B5EBAB5E3CC2C4ED4EE0C2EEFB34F24A84B1ABE1858CB582`
- Authenticode：`NotSigned`
- 构建进程已清除 `TAURI_SIGNING_PRIVATE_KEY`、`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`、`TAURI_SIGNING_PRIVATE_KEY_PATH`；NSIS 文件已生成，但 Tauri 最终 updater 签名阶段因公钥存在而无私钥，命令以错误退出。该安装器仅用于本地功能验证，不是正式发布包；未生成可用 `.sig`，未上传、未提交、未发布。

## 待验收

- 用户需在隔离 CS2 目录安装该候选，回读 true/false JSON、运行中禁写、升级保留 false、损坏配置恢复和下一次插件载入行为。
- 真实 CS2 中的 `lbtv_map_rotation`、15 秒自动换图、`lbtv_map_next` 立即换图日志/截图尚未取得。
- 未完成组件级 `map-rotation-settings.spec.ts` 和 Rust service 专项测试。
