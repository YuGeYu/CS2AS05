# 0.5.9 饰品系统全面迁移到 Inventory Simulator 执行方案

日期：2026-08-17  
适用项目：`E:\CS2AS05`  
目标版本：CS2 人机增强助手 `0.5.9` 开发中版本  
方案状态：待执行

## 一、最终产品决策

0.5.9 的饰品能力全面转向：

```text
ianlucas/cs2-css-inventory-simulator
```

助手不再维护本地饰品数据库、饰品图片库和 `player_loadout.json` 编辑器。玩家在 `inventory.cstrike.app` 创建并装备库存，助手负责环境检测、旧引擎精确清除、新插件安装、启动本地对局、状态诊断和中文教程。

本次迁移采用以下不可变原则：

1. 只保留一个换肤引擎，Inventory Simulator 是唯一运行时实现。
2. 旧 `PlayerSkinMod`、CS2-Skin-Forge 和应用缓存直接删除，不备份、不归档、不提供恢复按钮。
3. 两套插件绝不共存，避免同时 hook `GiveNamedItem` 和反复覆盖同一武器实体。
4. 删除操作使用明确白名单，不递归清理 CounterStrikeSharp 或整个 `addons`。
5. 不删除、不覆盖、不重装 Metamod、CounterStrikeSharp、Bot Improver、NadeSystem、BotRandomizer、BotVision 或其他项目。
6. 不修改 Demo、复盘、热力图、公告、客服、安装诊断等无关功能。
7. 不在助手中嵌入 Steam 登录网页，不使用浏览器控件模拟网站操作；只通过固定白名单命令打开系统默认浏览器。
8. 不在官匹、竞技平台或受 VAC 保护的环境使用，只服务助手启动的 `-insecure` 本地 BOT 场景。

本方案覆盖完整迁移和验收，但本轮只新增方案文档，不执行删除、不修改代码、不构建、不发布。

## 二、为什么必须单引擎迁移

当前实现的决定性状态是助手写入 `player_loadout.json`，而不是 CS2 对装备结果的回执。`PlayerSkinMod` 在物品生成后立即、下一帧和延迟定时器中反复补写属性，刀、手套和已有武器还有额外延迟路径。其结果依赖游戏实体生成时序，容易出现默认外观回退、刷新延迟和状态不一致。

Inventory Simulator 的核心路径位于库存层：

```text
SteamID64
-> 远端装备库存
-> CCSPlayerInventory::GetItemInLoadout
-> GiveNamedItem Pre
-> CEconItemView
-> SendInventoryUpdateEvent
```

它以 `(SteamID, team, loadout slot)` 识别玩家和装备槽，并提供物品 `uid/hash`、CT/T 武器、刀、手套、角色、音乐盒、收藏品、贴纸、挂件和涂鸦等库存语义。它不是给旧 JSON 增加几个字段，而是替换旧系统的数据所有权和运行时模型。

旧插件和新插件都会介入 `GiveNamedItem`。若并存，旧插件的后续帧补写可能覆盖新插件提供的 `CEconItemView`，任何成功都无法归因。因此迁移不能使用“双引擎兼容期”，安装新引擎前必须精确移除旧引擎。

## 三、固定上游与供应链证据

实施时固定以下调查版本，不跟随浮动 `main`：

```text
repository: https://github.com/ianlucas/cs2-css-inventory-simulator
tag: 3.1.0
commit: 5e3c96283b3d3f5aeba44822a38031df2e213376
license: MIT
license file: License.txt
target framework: net10.0
CounterStrikeSharp.API: 1.0.371
release asset: InventorySimulator-v3.1.0.zip
release size: 72,342 bytes
release SHA-256: B42A716E331C89DCCD079DB78677B3087FC43947757D75B5AE43A9C4823912D6
```

固定 Release 文件哈希：

| 文件 | SHA-256 |
| --- | --- |
| `InventorySimulator.dll` | `31D94F3FABAA0A51C83165CDC2555BD7A4870BC551E45F9899608073F12CC187` |
| `InventorySimulator.deps.json` | `85D37A8E97FDC28242C523E49CC933F7FD68D8DDE2275E0C8E4A171D2FEE5024` |
| `InventorySimulator.pdb` | `8DF71BEB412AE15B63137C94A2AF22F33362375E4B0E4911310199AA7173ADA9` |
| `inventory-simulator.json` | `791BD0C6932D2B8E87627254AA942A9D0F7ACB720F3CAAC0F098D406B34C1077` |
| `lang/en.json` | `2A97299D5E627339A4E88CC7214E1AE0011DCEBA9296944A89768A1F84CD433B` |
| `lang/pt-BR.json` | `F04273F5C7E94C31C778590332949700633A51DAF8C07F81DF4931014DBB293D` |
| `lang/zh-Hans.json` | `C177F2C235504C6F6CA2A4C1322DFC7278A3707EB7D1EE3BAAE46A05999C20A3` |

调查时已从固定源码执行 `dotnet build InventorySimulator.csproj -c Release --nologo`，结果为 0 warning、0 error。该结果只证明固定源码能在调查环境编译，不代替迁移后的真实 CS2 验收。

实施时新增受管上游目录：

```text
third_party/cs2-css-inventory-simulator/
  upstream/                      # 固定 commit 源码快照
  patches/
    enable-ws-by-default.patch   # 唯一产品默认值 patch
  UPSTREAM.md                    # tag、commit、来源、更新流程
  BUILD.md                       # 固定 SDK、构建和校验步骤
  LICENSE                        # 原始 MIT License
  SHA256SUMS.txt                 # 上游和下游产物哈希

src-tauri/resources/inventory-simulator/
  InventorySimulator.dll
  InventorySimulator.deps.json
  InventorySimulator.pdb
  lang/en.json
  lang/pt-BR.json
  lang/zh-Hans.json
  inventory-simulator.json
  manifest.json                  # 助手自己的版本与哈希清单
```

`NOTICE.md` 新增 `ianlucas/cs2-css-inventory-simulator`、MIT、固定 tag/commit 和下游 patch 说明。下游重新编译后，必须记录新的 DLL/PDB 哈希，不能继续把上游 Release DLL 哈希写成下游产物哈希。

## 四、唯一允许的上游默认值 patch

上游 `3.1.0` 默认配置：

```text
invsim_ws_enabled false
invsim_ws_immediately false
invsim_ws_cooldown 30
```

`FakeConVar` 是插件进程内跟踪状态的 console command，不是自动持久化配置。为了让玩家教程中的 `!ws` 开箱可用，下游固定源码只修改：

```diff
- invsim_ws_enabled default false
+ invsim_ws_enabled default true
```

保持：

```text
invsim_ws_immediately false
invsim_ws_cooldown 30
```

这样玩家输入 `!ws` 后拉取新库存，装备在重生或换图时稳定应用。首版不默认启用即时删除并重发当前武器，避免弹药、投掷物、C4、拾取物和 Bot Improver 行为出现新的时序问题。

为保持上游完整库存能力，不擅自关闭 StatTrak 增量和喷漆消耗。页面隐私说明必须明确：库存拉取依赖 `inventory.cstrike.app`，StatTrak 和喷漆消耗可能向该服务写入状态。真实验收必须验证限流、失败降级和重复请求行为。

任何额外 patch 都必须单独立项、说明用户收益并提供真实 CS2 证据，不能在本迁移中顺手改变上游语义。

## 五、旧项目无痕删除边界

### 5.1 项目工作树中应删除

实施阶段只删除旧饰品项目专属内容：

```text
src/features/skin-forge/**
src/stores/skinForge.ts
src/types/skin-forge.ts
src/services/tauri/skinForge.ts
src-tauri/src/commands/skin_forge.rs
src-tauri/resources/skin-forge/**
third_party/CS2-Skin-Forge/**
scripts/sync-skin-forge-upstream.mjs
旧 skin-forge / PlayerSkinMod 专属测试
package.json 中 skin-forge:sync
src-tauri/tauri.conf.json 中 resources/skin-forge
src-tauri/tauri.conf.json 中 $APPLOCALDATA/skin-forge/cache/images/**
NOTICE.md 中旧 CS2-Skin-Forge / PlayerSkinMod 来源条目
```

同步移除所有旧命令、DTO、类型、store、路由、导航文字、未应用草稿提示、旧帮助文字和旧图片缓存逻辑。删除必须通过 `rg` 做残留检查，不能只让入口不可见。

### 5.2 玩家 CS2 目录中应删除

只允许删除经过规范化路径校验后的：

```text
<csgo_root>/addons/counterstrikesharp/plugins/PlayerSkinMod/**
```

删除前必须确认：

1. 用户选择路径确实包含 `game/csgo` 的必要标志文件。
2. 规范化目标的最后一级目录名严格等于 `PlayerSkinMod`。
3. 目标位于当前已验证 `csgo_root/addons/counterstrikesharp/plugins` 内。
4. 目标不是符号链接、junction 或指向白名单之外的 reparse point。
5. CS2 已完全退出，相关 DLL 没有被进程占用。

用户已经要求不备份，因此不复制此目录、不重命名为 `.bak`、不生成压缩包、不保留旧 DLL。

### 5.3 应用数据中应删除

使用 Tauri 实际解析的 `app_local_data_dir()`，只删除：

```text
app_local_data_dir()/skin-forge/**
```

包括旧 loadout、草稿、图片缓存、临时文件和旧备份目录。不得硬编码某个用户目录。删除后检查父目录中无 `skin-forge` 残留，但不清理应用数据根目录中的其他模块。

### 5.4 永远禁止触碰

```text
<csgo_root>/addons/counterstrikesharp/**             # 不能整体删除/覆盖
<csgo_root>/addons/counterstrikesharp/plugins/NadeSystem/**
<csgo_root>/addons/counterstrikesharp/plugins/BotRandomizer/**
<csgo_root>/addons/metamod/**
<csgo_root>/addons/BotVision/**
CounterStrikeSharp 的其他插件和配置
Bot Improver 资源和配置
用户 Demo、Steam 库文件、CS2 cfg
助手的公告、客服、诊断、复盘等数据
仓库中任何非 skin-forge 的未提交改动
```

`addons/counterstrikesharp/configs/core.json` 当前需要 `FollowCS2ServerGuidelines: false`。首选只读验证。若未来必须提供自动修复，只能使用结构化 JSON 解析后修改这一字段并保留所有未知字段，禁止用模板覆盖整个文件。

## 六、安装事务与失败恢复

用户要求不保留旧引擎，但这不等于允许半安装。安装使用同一文件系统 staging 和原子替换：

```text
1. 解析并规范化用户选择的 CS2 路径
2. 检查 CS2 进程已经退出
3. 检测 Metamod、CounterStrikeSharp 和版本兼容性
4. 校验助手内置 Inventory Simulator manifest 与所有文件 SHA-256
5. 在 plugins 同级创建唯一命名的临时 staging 目录
6. 完整写入新插件文件，并在 staging 中回读哈希
7. 在 gamedata 同级写入临时文件，并回读哈希和 JSON 结构
8. 精确删除 PlayerSkinMod 目录，不备份
9. 原子 rename staging 为 plugins/InventorySimulator
10. temp + rename 部署 gamedata/inventory-simulator.json
11. 删除 app_local_data_dir()/skin-forge，不备份
12. 回读新插件、gamedata、语言包、版本和哈希
13. 返回最终状态；只有全部成立才能显示“已就绪”
```

如果第 1 至 7 步失败，旧插件尚未删除，直接中止。第 8 步以后失败时，不恢复旧插件；保留可识别的新 staging 状态并允许用户点击“继续安装”幂等重试。重复执行不得删除其他插件，也不得累积多个临时目录。

安装过程需要逐阶段结构化记录，但日志不得记录 Steam 登录 cookie、会话或私密凭据。删除日志只记录规范化目标、结果和错误码，不复制旧配置内容。

## 七、Rust/Tauri 契约

新增状态模型：

```ts
interface InventorySimulatorStatus {
  selectedRoot: string | null
  csgoRoot: string | null
  cs2Running: boolean
  counterStrikeSharpInstalled: boolean
  counterStrikeSharpVersion: string | null
  legacyPlayerSkinModPresent: boolean
  legacyAppDataPresent: boolean
  inventorySimulatorPresent: boolean
  resourceVersion: string
  deployedVersion: string | null
  missingFiles: string[]
  hashMismatches: string[]
  gamedataPresent: boolean
  coreGuidelineCompatible: boolean | null
  serviceReachable: boolean | null
  serviceCheckedAt: string | null
  ready: boolean
  blockedCode: string | null
  blockedMessage: string | null
}
```

新增命令：

```text
inventory_simulator_get_status(root_path)
inventory_simulator_install(root_path)
inventory_simulator_open_workshop()
inventory_simulator_check_service()
```

`inventory_simulator_install` 内部完成旧插件和旧应用数据移除，不向前端暴露可随意删除路径的通用命令。若实现单独的 `remove_legacy`，也只能由后端从已验证 CS2 根目录计算固定目标，不能接收前端传入的删除路径。

`inventory_simulator_open_workshop()` 不接受 URL 参数，只允许打开：

```text
https://inventory.cstrike.app
```

Windows 继续复用项目现有 `open_external()` 系统浏览器路径，不新增任意 URL opener，不嵌入 WebView 登录。

结构化错误码：

```text
CS2_RUNNING
CS2_ROOT_INVALID
COUNTERSTRIKESHARP_MISSING
COUNTERSTRIKESHARP_INCOMPATIBLE
LEGACY_REMOVE_FAILED
RESOURCE_MISSING
RESOURCE_HASH_MISMATCH
DEPLOY_FAILED
DEPLOY_VERIFY_FAILED
GAMEDATA_INVALID
CORE_GUIDELINE_ENABLED
SERVICE_UNREACHABLE
```

网络检查不是安装硬阻断。服务暂时不可达时仍可完成本地插件部署，页面显示“服务暂不可用，稍后再同步”，不能把插件文件正确误报成库存已经拉取成功。

## 八、前端产品结构

导航名称改为：

```text
库存换肤
```

旧“皮肤工坊”本地编辑器整体移除。新页面采用紧凑工作台，不做营销 Hero，不加载上万张饰品图片，不使用卡片套卡片。

### 8.1 首屏信息层级

1. 顶部状态带：CS2 路径、基础环境、新插件、在线服务、最终就绪状态。
2. 主操作区：根据状态只突出当前可执行的一个主要动作。
3. 三步流程：启用插件、制作库存、启动并同步。
4. 常见问题与完整教程：默认收起，需要时展开。
5. 安全边界：明确仅限 `-insecure` 本地 BOT。

主按钮：

```text
一键启用库存换肤
打开饰品工坊
启动本地对局
```

辅助按钮：

```text
重新检测
查看完整教程
复制 !ws
```

### 8.2 页面状态

```text
未选择 CS2
CS2 正在运行
基础环境未安装
旧换肤待移除
正在安装
已就绪
服务暂不可用
插件文件异常
gamedata 需要更新
```

状态必须使用文字、Lucide 图标、边框和颜色共同表达，不能只靠红黄绿。按钮在执行期间禁用并显示当前阶段；超过 300ms 显示明确进度文案，不使用无限装饰动画。

### 8.3 交互改造对照

| Before | After | Why |
| --- | --- | --- |
| 助手内编辑并写入 `player_loadout.json` | 打开官网库存工坊，按 SteamID 保存装备 | 与新运行时真实数据源一致 |
| “已写入游戏”但没有游戏回执 | 分开显示“插件已就绪”“网站库存已保存”“等待重生应用” | 不把中间状态包装成最终成功 |
| 助手加载庞大本地饰品目录与图片缓存 | 页面只管理安装、状态、启动和教程 | 减少安装体积、缓存和维护面 |
| 旧插件与新插件可能同时存在 | 安装事务精确移除 PlayerSkinMod 后部署新插件 | 消除 hook 冲突和结果不可归因 |
| 高频步骤使用长动画 | 常规状态切换即时，模态/抽屉仅做短促进入退出 | 提高工具型界面的操作速度 |

### 8.4 视觉与动效

- 沿用现有助手字体和设计 token，不引入 Google Fonts、CDN 字体或新的单色主题。
- 采用密度较高的桌面工具布局，8px 以内圆角，清晰分区和可扫描状态行。
- 主要成功状态使用绿色，警告使用黄色，错误使用红色；文字仍保持高对比。
- 所有图标按钮有 tooltip 和可见 focus ring，按钮点击区域不小于 36x36。
- 模态/抽屉进入 `180--240ms cubic-bezier(0.23, 1, 0.32, 1)`，退出 `120--160ms`。
- 只 transition 明确的 `opacity`、`transform`、`border-color`，禁止 `transition: all`。
- `prefers-reduced-motion: reduce` 下立即展示最终状态。
- 验证 `1440x900`、`1100x700`、`980x640` 和窄窗口；操作区换行但不能遮挡状态或按钮文字。

## 九、玩家完整教程

下面文案应直接进入助手“查看完整教程”，不要求玩家理解插件目录或手工复制 ZIP。

### 第一步：一键启用库存换肤

1. 在助手中选择正确的 CS2 安装目录。
2. 完全关闭 CS2，然后进入“库存换肤”。
3. 点击“一键启用库存换肤”。
4. 助手会检查 CounterStrikeSharp，移除旧换肤插件并安装新的 Inventory Simulator。
5. 看到“已就绪”后再继续。旧搭配不会迁移，也不会保留，这是新旧库存模型不同导致的预期结果。

如果显示“基础环境未安装”，先回到“安装与诊断”完成助手环境安装；不要手工覆盖整个 `addons` 文件夹。

### 第二步：打开饰品工坊

1. 点击“打开饰品工坊”，系统默认浏览器会打开 `https://inventory.cstrike.app`。
2. 使用 Steam 登录。登录用于确认你的 SteamID，并把网站库存关联到正确玩家。
3. 浏览器地址栏应保持在 `inventory.cstrike.app` 或 Steam 官方登录域名；助手不会索取 Steam 密码或读取浏览器 cookie。

### 第三步：制作自己的库存

网站中可以创建和装备：

```text
CT/T 武器
刀
手套
角色
音乐盒
贴纸
挂件
收藏品
涂鸦
```

操作原则：

1. 先创建物品，再选择 CT 或 T 的对应装备槽。
2. 对枪械选择皮肤、磨损、模板、StatTrak、名称、贴纸和挂件。
3. 刀、手套、角色和音乐盒也要放入对应槽位，只有创建但未装备不会在游戏中出现。
4. 若网站支持检视链接导入，可粘贴合法的 CS2 检视链接，让网站生成对应物品。
5. 完成后点击网站的装备/保存操作，确认该物品已经出现在当前阵营槽位。
6. CT 和 T 是两套独立装备，不要只配置其中一边后误以为全局生效。

### 第四步：启动本地 BOT 对局

1. 回到助手，确认页面仍显示“已就绪”。
2. 点击“启动本地对局”。
3. 必须由助手以 `-insecure` 启动本地 CS2。
4. 进入本地 BOT 地图后等待玩家和插件完成连接。

不要带着本功能进入官匹、5E、完美或其他受保护竞技环境。退出本地体验后，如需正常联机，使用助手的正常启动流程恢复不带 `-insecure` 的游戏启动方式。

### 第五步：同步刚保存的装备

1. 在 CS2 聊天框输入 `!ws`。
2. 插件会重新拉取你在网站保存的库存。
3. 默认有 30 秒刷新冷却，请不要连续刷屏。
4. 为保证武器实体稳定，刷新后在下一次重生或换图时查看新外观。
5. 若只打开了网站但没有点击装备/保存，游戏不会自动猜测你想使用哪件物品。

### 第六步：修改与再次同步

以后修改不需要重复安装插件：

```text
打开饰品工坊
-> 修改并保存网站库存
-> 回到游戏输入 !ws
-> 重生或换图
```

插件更新、文件损坏或 CS2 更新导致 gamedata 失效时，助手会显示异常；此时点击“重新检测”，再按页面指引重新安装当前受管版本。

### 常见问题

**输入 `!ws` 只看到提示，没有刷新：** 先在助手中重新检测插件版本。0.5.9 使用的下游构建默认启用 `!ws`；上游原版 `3.1.0` 默认未启用，手工混装会造成行为不同。

**游戏里还是默认皮肤：** 确认网站物品已经装备到当前 CT/T 槽位；等待 30 秒冷却后输入一次 `!ws`，然后重生或换图。网站返回空库存时，默认外观是正确降级结果。

**网站暂时打不开：** 本地插件仍可保持已安装，但无法拉取新的库存。不要反复重装；稍后点击“检查服务”并再次同步。

**只有一部分物品更新：** 检查当前阵营和槽位，保存后输入一次 `!ws` 并重生。若稳定复现，提交助手诊断，记录物品类型、阵营、地图和操作顺序。

**StatTrak 或喷漆数量变化：** 这是上游完整库存能力的一部分，插件可能向公共服务写入 StatTrak 增量或喷漆消耗。断网时相关远程状态可能延迟或失败，不应影响助手其他功能。

**能不能恢复旧皮肤工坊：** 不能。0.5.9 使用单一库存模拟器，旧项目和旧搭配会被精确删除且不备份。

## 十、实施阶段

### 阶段 A：基线与删除清单测试

保存：

```powershell
Set-Location E:\CS2AS05
git status --short
git rev-parse HEAD
git diff --stat
```

先写失败测试，证明：

- 路径验证不会把 `plugins`、`counterstrikesharp` 或 `addons` 当成删除目标。
- junction/reparse point 不能越过已验证 CS2 根目录。
- 只识别精确 `PlayerSkinMod` 和 `app_local_data_dir()/skin-forge`。
- 未知插件和未知 `core.json` 字段被保留。
- 工作树删除清单不包含其他模块。

Gate：任何测试能删除白名单之外路径时立即停止。

### 阶段 B：固定上游、patch 与资源清单

1. 拉取固定 commit 源码和 MIT License。
2. 保存 Release ZIP、大小和 SHA-256 证据。
3. 应用唯一 `invsim_ws_enabled` 默认值 patch。
4. 使用固定 .NET SDK 构建 Release。
5. 生成助手资源 manifest 和所有下游哈希。
6. 新增 `NOTICE.md` 条目并移除旧来源条目。

Gate：源码 commit、patch、DLL 与 manifest 无法一一追溯时停止。

### 阶段 C：Rust 安装事务

实现状态检测、路径白名单、staging、精确删除、原子部署、哈希回读和结构化错误。所有文件系统测试使用临时目录，覆盖安装成功、重复安装、旧插件不存在、删除失败、DLL 被占用、资源损坏和中途失败。

Gate：必须证明重复安装幂等，且任意失败不会删除其他插件。

### 阶段 D：Vue 库存换肤页面

移除旧编辑器和旧 store，接入新 DTO。完成状态带、三步主流程、固定官网打开、中文完整教程、服务检查、错误恢复和无障碍交互。不要把安装逻辑堆入单个 View，状态管理和 Tauri service 分层。

Gate：页面只能根据后端实际磁盘和进程状态显示“已就绪”，不能沿用一次成功后的内存状态。

### 阶段 E：真实 Tauri 与 CS2 验收

至少验证：

1. 装有旧 PlayerSkinMod 的现有 0.5.8 用户升级。
2. 从未安装旧换肤功能的新用户。
3. 旧插件目录被占用时安装被阻止。
4. CounterStrikeSharp 缺失或版本不兼容。
5. 网站有库存、空库存和暂时不可达。
6. CT/T 枪械、刀、手套、角色、音乐盒、贴纸、挂件、收藏品和涂鸦。
7. `!ws` 30 秒冷却、重生应用和换图应用。
8. StatTrak 增量、喷漆消耗、断网和服务恢复。
9. Bot Improver、投掷物、C4、拾枪、切枪、死亡、重生和地图切换。
10. 退出本地对局后助手仍能点击和切换页面，无事件监听或后台任务泄漏。

真实游戏效果由用户执行并提供屏幕证据；执行 AI 负责准备日志、步骤、版本与哈希回读，不把本地单元测试冒充游戏验收。

### 阶段 F：构建安装器

只有阶段 A 至 E 全部通过后才构建 0.5.9 安装器。保留现有签名流程，私钥只注入单次构建进程，不写入仓库或方案。输出：

```text
安装器绝对路径
文件大小
SHA-256
Tauri updater .sig 路径和大小
Windows Authenticode 状态
构建开始/结束时间
每条命令 exit code
```

不得覆盖、删除或重新发布已固定的 0.5.8 安装器。

## 十一、测试矩阵

### Rust/Tauri

- 正确 CS2 根目录和错误根目录。
- `PlayerSkinMod` 存在、不存在、锁定、只读和 reparse point。
- `InventorySimulator` 不存在、完整、缺文件、哈希错误和版本旧。
- staging 写入失败、删除失败、rename 失败、gamedata 失败和幂等重试。
- `core.json` 合法、缺字段、目标字段为 true、未知字段和无效 JSON。
- CS2 运行时拒绝修改。
- 固定官网 allowlist，任意 URL 参数不可达。
- 服务 200、空 `{}`、超时、DNS/证书错误和异常 JSON。

### Vue/TypeScript

- 所有状态与按钮可用性。
- CS2/Demo 根目录切换后旧请求不能覆盖新状态。
- 卸载组件后无 timer、listener、AbortController 或异步回写泄漏。
- 安装超过 300ms 显示阶段文案。
- 错误可重试，但不能误显示已就绪。
- 教程键盘可展开，focus 顺序合理，屏幕阅读器能读出状态。
- `1440x900`、`1100x700`、`980x640`、窄窗口和 200% 缩放无重叠。
- reduced-motion 只显示最终状态。
- 不请求旧饰品 JSON、图片缓存或外部字体。

### 残留检查

实施后执行：

```powershell
rg -n -i "skin[-_ ]forge|PlayerSkinMod|CS2-Skin-Forge|player_loadout" `
  src src-tauri scripts package.json NOTICE.md tests
```

允许命中的内容只能是明确的迁移检测常量、删除目标和历史文档。运行时代码、导航、旧 DTO、旧资源 bundle、旧同步脚本和旧帮助文案必须全部消失。

### 回归范围

- 正常安装与诊断。
- Bot Improver 安装、启动和状态检测。
- 本地 BOT 启动参数仍包含正确的 `-insecure` 边界。
- Demo 解析、比分、Rating、表现雷达、地图回放和热力图。
- 公告、快快客服、故障提交和更新器。
- 助手关闭、CS2 关闭和 Seelen UI 等桌面环境变化后的响应性。

## 十二、证据目录

建议：

```text
E:\CS2AS05\workspace\release-evidence\inventory-simulator-0.5.9-<timestamp>\
```

至少保存：

```text
baseline.txt
upstream.txt
upstream-release.sha256.txt
downstream-patch.diff
downstream-build.txt
downstream-sha256.txt
rust-tests.txt
frontend-tests.txt
typecheck-lint-build.txt
legacy-removal-boundary.txt
tauri-status-before.json
tauri-status-after.json
service-check.json
desktop-1440x900.png
desktop-1100x700.png
desktop-980x640.png
cs2-plugin-list.txt
cs2-inventory-simulator-log.txt
real-cs2-acceptance.md
installer-sha256.txt
acceptance-summary.md
```

日志中的 SteamID 可在公开 API 证据中只保留必要尾号；不得保存 Steam 密码、cookie 或浏览器会话。

## 十三、完成标准

只有以下全部满足才能报告 implemented：

1. 项目和玩家环境中旧换肤运行时代码、资源、插件、缓存和入口已按白名单删除，且没有备份。
2. 白名单之外的插件、配置、Demo、Steam 文件和工作树改动均未受影响。
3. 新插件固定于 tag `3.1.0` / commit `5e3c962...`，MIT、patch 和哈希可追溯。
4. 助手状态、安装和官网打开使用统一 Tauri 契约；官网 URL 是固定 allowlist。
5. `!ws` 在下游构建默认启用，保持 30 秒冷却和非即时应用。
6. 助手不再加载或生成旧饰品数据库和图片缓存。
7. 网站创建的 CT/T 装备能在真实 `-insecure` 本地 BOT 中按 SteamID 应用。
8. 完整物品类型、空库存、断网、冷却、重生、换图和远程状态写入均有证据。
9. Bot Improver、投掷物、C4、拾枪、切枪、Demo 与助手响应性无回归。
10. Tauri UI、窄窗口、键盘、屏幕阅读器和 reduced-motion 验收通过。
11. 安装器构建成功，路径、大小、SHA-256、updater signature 与 Authenticode 状态如实记录。
12. 用户完成真实 CS2 效果确认后，才允许发布 0.5.9。

## 十四、停止条件

出现以下任一情况立即停止，不构建、不发布：

- 删除目标无法证明严格位于白名单路径。
- 需要删除或覆盖整个 `addons`、CounterStrikeSharp 或其他插件才能安装。
- 新旧插件必须共存才能工作。
- 上游源码、Release、License、patch 或下游 DLL 无法固定和校验。
- 只能通过覆盖整个 `core.json` 才能关闭服务器规范开关。
- 安装失败后留下不可重试的半安装状态。
- 真实 Tauri 页面与磁盘状态不一致。
- `!ws`、重生或换图无法稳定应用，或出现未解释的实体/游戏崩溃。
- Bot Improver、投掷物、C4、拾枪、切枪或助手响应性出现未定位回归。
- 网站/API 行为与固定源码契约冲突。
- 测试失败尚未定位，或工作树出现无法确认来源的冲突。

## 十五、本轮交付边界

本轮只交付这份新增方案：

```text
docs/inventory-simulator-complete-migration-plan-0.5.9-20260817.md
```

没有删除旧项目，没有下载或复制上游代码，没有修改程序代码，没有构建安装器，没有部署或发布。后续实际执行必须从当前工作树重新读取状态，并以本方案中“精确删除、不备份、不触碰其他项目”的最新产品决策覆盖此前调查文档中的旧回滚建议。
