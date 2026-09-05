# 0.5.11 本地签名候选报告

## 版本与状态

- 当前 checkout 版本：`0.5.11`
- 上一正式版本：`0.5.9`
- `0.5.10`：已撤销，未重新启用线上发布，不覆盖其历史产物。
- 当前状态：资源修复候选 / 0.5.11 本地签名候选，待真实 BOT 验收，不是正式发布。

## 资源证据

- Steam 官方 `gameinfo.gi`：9433 bytes，SHA-256 `B1391E73DBEC2E078BDBAF7279C2B955084CF2B38A47E3D8181662E7948679B8`
- WithBots `gameinfo.gi`：9501 bytes，SHA-256 `04B867124656BC9E768ED50B45193225D799E273D2ADC193632ED03543B37EB1`
- 最终 `CS2BotImprover.zip`：SHA-256 `AF5401B359B76D3EC22B4B3693EC61D670CC85D141C1FD7788DFE18B8EF2FD4B`
- Low VPK：115353 bytes，SHA-256 `5EC7F50BCD97E3678C7545306407110E7E87C6E09CBC604DA70CFA56A612206A`
- Medium VPK：406110 bytes，SHA-256 `495FCAE5DFAABB1B47CEDF636FD48E906085823C09BEC4C6D5589A33BEBEF695`
- High VPK：115601 bytes，SHA-256 `EA7D7FC6DEF8E0342AA817F64A7E9BC3844E3A6372F1567C3C60C74F6AECE233`

## 自动化验证

- `npm test -- --run tests/gameinfo-upstream-contract.spec.ts tests/map-rotation-contract.spec.ts tests/map-rotation-plugin-config.spec.ts tests/installer-contract.spec.ts`：4 个测试文件，13 项通过。
- `npm run typecheck`：通过。
- `cargo check --manifest-path src-tauri/Cargo.toml`：通过；存在既有 warning，无 error。

## 签名与产物

构建时从 `C:\Users\GOPtZ\Documents\CS2AS05-release-keys` 读取 DPAPI 文件并对内容执行 `.Trim()`，仅在单一构建进程中注入 `TAURI_SIGNING_PRIVATE_KEY` 与 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。未使用单独的 `TAURI_SIGNING_PRIVATE_KEY_PATH` 触发签名，构建结束后清理环境变量。实际产物如下：

- 安装器：`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.11_x64-setup.exe`
  - 117412601 bytes
  - SHA-256 `A540D81B528A66EF25C84CE8E8B75C6847C93AAAD5028B8FEC64415EE0A295D7`
- updater 签名：`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.11_x64-setup.exe.sig`
  - 436 bytes（解码后签名载荷 327 bytes）
  - SHA-256 `583921A1E39AD3AEEE0E53FDD864E3F85E48282ED2787CC89FB1342D0DE0BE28`

## 验收边界

当前未完成真实 CS2 BOT 行为验收，不能宣称 Medium 档反应或射击强度已修复。后续须在真实 BOT 环境验证后，才能决定是否将该候选升级为正式发布。
