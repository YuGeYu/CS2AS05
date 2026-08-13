# CS2 皮肤本地修改器

一个用于反恐精英2的本地皮肤自定义插件，允许玩家自定义武器皮肤、刀具、手套、角色模型和音乐盒。

**[English Documentation](README.md)**

## 演示

以下截图来自搭配 [CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver) 插件的本地机器人对局：

> CS2-Bot-Improver is a plugin for Counter-Strike 2 that improves bots' aim, movement, nade throwing, personalities, strategies, etc. Aims to enhance your experience when playing against bots offline or with friends. It can be installed on both clients and servers.

![Demo 1](imgs/demo1.png)
![Demo 2](imgs/demo2.png)
![Demo 3](imgs/demo3.png)

[点击查看更多演示截图](./screenshots.md)

## VAC 风险警告

**重要：此插件会修改 CS2 游戏文件，并需要 `-insecure` 启动选项。**

虽然此插件：
- 仅修改本地客户端渲染
- 不影响真实 Steam 库存
- 仅设计用于离线/本地游戏

**但使用任何第三方软件与 CS2 都存在固有风险。** Valve 的反作弊系统（VAC）可能会标记修改过的游戏文件。作者不对可能发生的 VAC 封禁或账户限制承担任何责任。

**使用风险自负。**

> **安全提示：** 为保障账户安全，建议使用 Steam 小号来进行本地游戏。

## 功能特性

- **武器皮肤**：自定义 30+ 种武器的皮肤
- **刀具**：20 种刀型 + 多款刀皮肤可选
- **手套**：8 种手套类型 + 多款皮肤变体，支持分队配置（CT/T 独立手套），手套名称多语言显示
- **角色模型**：35 款 CT + 44 款 T 角色模型（多语言名称：EN/ZH/JA/KO）
- **音乐盒**：18+ 首音乐盒可选，多语言名称
- **挂件/饰品**：为武器挂载饰品，支持自定义位置和种子
- **武器命名**：通过命名标签设置自定义武器名称
- **StatTrak**：启用 StatTrak 击杀计数器，支持自定义击杀数
- **随机模式**：每次重生随机选择皮肤
- **自定义模式**：精确选择每个皮肤
- **桌面面板**：原生 Tauri 应用，方便自定义
- **多语言支持**：English、简体中文、繁體中文、日本語、한국어、Русский

## 安装说明

### 前提条件

1. 已安装反恐精英2
2. CounterStrikeSharp — 点击"部署插件"时面板会自动检测并安装，无需手动操作。

### 第一步：下载并安装面板

下载适用于您平台的最新版本：
- Windows: `CS2-Skin-Mod_x.x.x_x64-setup.exe`
- Linux: `cs2-skin-mod-panel_x.x.x_amd64.AppImage`
- macOS: `CS2-Skin-Mod_x.x.x_aarch64.dmg`

安装并启动应用程序。

### 第二步：部署插件

1. 启动 CS2 Skin Mod Panel
2. 点击右上角齿轮图标进入 **设置**
3. 设置 CS2 路径（例如 `C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive\game\csgo`）
4. 点击 **"部署插件"** -- 此操作会自动安装 CounterStrikeSharp（如缺失）并将插件文件复制到 CS2 目录

### 第三步：配置 CS2 启动选项

1. 打开 Steam
2. 右键点击反恐精英2
3. 选择"属性"
4. 在"启动选项"中添加：`-insecure`

**警告：这将阻止您在 VAC 保护的服务器上游戏。**

### 第四步：自定义并开始游戏

1. 在面板中自定义您的装备
2. 点击 **"应用装备"** 保存
3. 使用 `-insecure` 启动 CS2 并开始本地比赛
4. 在游戏中重生即可看到您的皮肤

## 使用方法

### 快速开始

1. 启动 CS2 Skin Mod Panel
2. 进入 **设置**，设置 CS2 路径
3. 点击 **"部署插件"** 安装插件
4. 在面板中自定义您的装备
5. 点击 **"应用装备"** 保存
6. 使用 `-insecure` 启动 CS2 并开始本地比赛
7. 在游戏中重生即可看到您的皮肤

### 游戏内命令

| 命令 | 说明 |
|------|------|
| `skin_menu` | 从面板重新加载装备（在面板中更改皮肤后使用） |
| `skin_random` | 启用随机皮肤模式 |
| `skin_reset` | 重置所有皮肤为默认 |

### 增强机器人体验（推荐）

为了获得最佳的机器人对战体验，我们推荐配合使用 [CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver)。

> **提示：** 建议先安装 CS2-Bot-Improver —— 它也会自动部署 CounterStrikeSharp 框架。之后再安装我们的换肤插件，它会检测到已有的 CounterStrikeSharp，仅部署自身的模组文件。

### 工作原理

面板和插件通过 JSON 文件（`player_loadout.json`）通信：
1. **面板** 将您的装备配置写入插件目录
2. **插件** 在玩家重生时读取此文件并应用皮肤
3. 插件监视文件变化，在您从面板保存时自动重新加载

插件使用 CounterStrikeSharp 钩入 CS2 的武器系统：
- 拦截 `GiveNamedItem` 调用，在武器发放时应用皮肤
- 拦截 `OnEntitySpawned`，捕获绕过 GiveNamedItem 钩子的武器
- 修改 `FallbackPaintKit`、`FallbackSeed` 和 `FallbackWear` 属性
- 通过 `ChangeSubclass` 输入更改刀具模型
- 在玩家重生时替换角色模型和手套

所有更改**仅限客户端**，不影响其他玩家或服务器。

### 语言支持

面板支持多种语言。点击设置图标切换：
- English
- 简体中文
- 繁體中文
- 日本語
- 한국어
- Русский

## 从源码构建

### 前提条件

- Node.js 18+
- Rust 工具链
- Tauri CLI

### 构建步骤

```bash
# 克隆仓库
git clone https://github.com/kaecho/CS2-Skin-Forge.git
cd CS2-Skin-Forge

# 安装前端依赖
cd Panel
npm install

# 构建 Tauri 应用
npm run tauri build
```

## 故障排除

### 皮肤不显示

1. 验证启动选项中包含 `-insecure`
2. 确保 CounterStrikeSharp 已安装（在设置中点击"部署插件"即可自动安装）
3. 确保在设置中正确设置了 CS2 路径
4. 点击"部署插件"确保插件已安装
5. 应用更改后重生
6. 检查服务器控制台是否有错误

### 手套纹理错位或显示异常

这通常是由于手套类型（DefIndex）与涂装（PaintKit）不匹配导致的。每种手套类型（血猎手套、运动手套、驾驶手套等）使用不同的 3D 模型和 UV 布局——将为一种手套设计的涂装应用到另一种手套上，会导致纹理扭曲。

**修复方法：** 在面板中切换手套类型时，涂装会自动重置为该类型的有效默认值。如果仍有问题：
1. 先选择目标手套类型
2. 然后从列表中选择涂装（面板只显示该类型的有效涂装）
3. 点击"应用装备"并在游戏中重生
4. 如果纹理仍有问题，先设置为"随机模式"，应用并重生，再重新选择目标手套

### 面板无法保存装备

1. 确保在设置中正确设置了 CS2 路径
2. 检查 `addons/counterstrikesharp/plugins/PlayerSkinMod/` 目录是否存在
3. 尝试再次点击"应用装备"

### 游戏崩溃

1. 验证游戏文件完整性
2. 移除冲突的插件
3. 检查 CS2 控制台是否有错误

## 文件结构

```
CS2-Skin-Forge/
├── addons/
│   └── counterstrikesharp/
│       └── plugins/
│           └── PlayerSkinMod/
│               ├── PlayerSkinModPlugin.cs  # 主插件
│               ├── PlayerSkinMod.csproj    # C# 项目
│               ├── Services/
│               │   ├── WeaponService.cs    # 皮肤/手套/刀具应用
│               │   └── LoadoutService.cs   # 装备文件解析
│               ├── Data/
│               │   └── StaticData.cs       # 静态皮肤/刀具/手套数据
│               ├── Models/
│               │   └── PlayerLoadout.cs    # 装备数据模型
│               ├── skins_en.json           # 皮肤数据（英文名 + legacy 标记）
│               └── player_loadout.json     # 装备配置（由面板生成）
├── Panel/
│   ├── src/                                # React 前端（含多语言支持）
│   ├── src-tauri/                          # Tauri 后端
│   └── package.json
├── .github/
│   └── workflows/
│       └── build.yml                       # CI/CD 配置
├── README.md                               # 英文文档
└── README_CN.md                            # 本文件
```

## 致谢

- 基于 [CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver) by ed0ard
- 手套皮肤应用参考 [Nereziel/cs2-WeaponPaints](https://github.com/Nereziel/cs2-WeaponPaints)
- 使用 [CounterStrikeSharp](https://github.com/roflmuffin/CounterStrikeSharp)
- 基于 [Tauri](https://tauri.app/) + [React](https://react.dev/) 构建

## 许可证

GPL-3.0

## 免责声明

本软件仅供**教育目的**使用。使用风险自负。作者不对以下情况负责：
- VAC 封禁或账户限制
- 游戏崩溃或不稳定
- 使用本软件造成的任何其他后果

使用本软件即表示您理解所涉及的风险。
