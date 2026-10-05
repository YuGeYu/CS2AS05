# Inventory Simulator v3.5.0 合入记录

## 来源

- 上游仓库：<https://github.com/ianlucas/cs2-css-inventory-simulator>
- Release：<https://github.com/ianlucas/cs2-css-inventory-simulator/releases/tag/3.5.0>
- tag：`3.5.0`
- tag commit：`157086c73718920285d3d761fe22c47a98090ec1`
- 用户提供源码包：`E:\dow\cs2-css-inventory-simulator-3.5.0.zip`
- 用户提供资源包：`E:\dow\InventorySimulator-v3.5.0.zip`
- 源码包 SHA-256：`A6F024A42D46ECB9267943B4C3E4E0F042D215BCED1DE408428B9F93877D6B74`
- 资源包 SHA-256：`9119DA8EFA655156DDB65D0F79F395A4DC1EEB158D3F357545797A8133FB8315`
- Target framework：`.NET 10.0`
- CounterStrikeSharp API：`1.0.375`
- 许可证：MIT

## 同步内容

- `src-tauri/resources/inventory-simulator/` 已替换为官方 `InventorySimulator-v3.5.0.zip` 的原始 Windows 资源。
- `third_party/cs2-css-inventory-simulator/upstream/` 已替换为用户提供的 v3.5.0 源码快照，没有加入下游代码补丁。
- 受管版本从 `3.3.0-release` 更新为 `3.5.0-release`，状态页会显示上游 tag 和固定 commit。
- `invsim_ws_enabled` 仍保持官方默认 `false`；使用 `!ws` 需要玩家在本地 BOT 服务端显式开启。
- v3.5.0 新增宠物生成、暖身阶段复活和宠物自由漫游能力；页面增加完整能力说明和对应服务端配置提示。
- `InventorySimulator.deps.json`、PDB、三种语言文件和 gamedata 均按官方资源包同步，没有只替换 DLL。

## 受管资源摘要

| 文件 | SHA-256 |
| --- | --- |
| `inventory-simulator.json` | `D645A656C89FBD61C5F99A655724A5C722AD40F0BEEE5A22AAF063AFBE236B50` |
| `InventorySimulator.dll` | `6E2AA5FBD58B634898C74E730D9A0D833225856CF9E8C984D9FD836420723829` |
| `InventorySimulator.deps.json` | `2A6336953530FBCF4E2716ED9B1AD4FB465D114B2EA2D6C91472AE4EC84EBC1E` |
| `InventorySimulator.pdb` | `DEF0103313EC2A851B1E3B4C3594B3E01C030CFC32DEED44C1404DDF4C7084AA` |
| `lang/en.json` | `2A97299D5E627339A4E88CC7214E1AE0011DCEBA9296944A89768A1F84CD433B` |
| `lang/pt-BR.json` | `F04273F5C7E94C31C778590332949700633A51DAF8C07F81DF4931014DBB293D` |
| `lang/zh-Hans.json` | `C177F2C235504C6F6CA2A4C1322DFC7278A3707EB7D1EE3BAAE46A05999C20A3` |
| `manifest.json` | `B21591C21DE1FC0E74A8031A10A2A461BB1E249CE321DDDB6E5DAF433B1C69C3` |

## 兼容边界

上游 v3.5.0 资源包依赖 CounterStrikeSharp API `1.0.375`。当前核心 BOT 官方包 v1.4.5 中的 API 资源为 `1.0.376`。本次只使用上游官方 Inventory Simulator Release，不自行重编译或改写其依赖；因此 Rust 资源校验和前端契约可以通过，但插件在本机 CS2 中的加载、`!ws` 刷新、宠物生成和喷漆行为仍需在退出 CS2 后安装并启动本地 `-insecure` BOT 对局进行验收。

Inventory Simulator 只用于助手启动的本地 BOT 场景。不要在官匹、5E、完美或其他受保护环境中使用。

## 验证与本地安装包

- 上游源码 `dotnet build -c Release --nologo`：0 warning、0 error。
- `npm run verify`：56 个测试文件、200 个测试通过；TypeScript、Oxlint、ESLint、Vite 构建通过。
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`：通过。
- `cargo test --manifest-path src-tauri/Cargo.toml --lib inventory_simulator`：5 passed、0 failed。
- 安装器：`E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.6.4_x64-setup.exe`。
- 安装器大小：`119,985,183` bytes；SHA-256：`ED11449852B2F148C6B087490C4C76A3167C71FF141AA3E2F5E574502879C044`。
- Windows Authenticode：`NotSigned`；updater `.sig` 未生成，因为当前环境只有公钥，没有 `TAURI_SIGNING_PRIVATE_KEY`。
- 以上自动化和打包结果不代替真实 CS2 运行验收；需要在本机安装后确认插件加载、`!ws`、喷漆和宠物行为。
