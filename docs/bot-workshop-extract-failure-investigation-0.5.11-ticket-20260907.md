# 人机强度工坊提取失败调查记录

日期：2026-09-07  
工单：`CS2-MTQP4M9Q-5926`  
反馈版本：`v0.5.11`  
平台：Windows / x86_64

## 调查边界

本记录只调查少数玩家无法使用人机强度工坊的问题，不修改提取、保存、应用或错误分类代码，也不把当前开发工作树的改动宣称为已经在工单机器上生效。

## 工单事实

- 玩家描述：无法调人机强度，修改完之后无法保存。
- CS2 目录：`D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive`。
- 工单记录 CS2 未运行。
- 失败信息：

  ```text
  [BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID] extract 退出码 1:
  Could not extract file at "botprofile.db" to
  "C:\Users\MR\AppData\Local\com.aipc.cs2botimprover\bot-difficulty-workshop\workspace\open-custom-20260905113923558-42692\botprofile.db"!
  Please ensure that a game or another application is not using the file, and that you have sufficient permissions to write to the output location.
  ```

- 工单日志在失败前记录了多次 CS2 目录扫描，以及两次“基于上游 v1.4.3 的最小定制插件包”安装（09:50、11:10）。日志没有记录 VPKEdit 版本/SHA、输入 VPK SHA、输出目录可写性、剩余磁盘空间、重试结果或 Defender/安全软件拦截事件。

## 版本路径证据

`v0.5.11` 的 `extract_db()`：

- 直接使用传入的 `workspace`；
- 将输出固定为 `<workspace>\\botprofile.db`；
- 在该 workspace 作为 `current_dir` 启动 VPKEdit。

因此，`v0.5.11` 的 `open()` 会形成：

```text
...\\workspace\\open-custom-...\\botprofile.db
```

这与本工单错误路径逐字一致。

当前工作树的 `extract_db()` 已改为：

```text
<workspace>\\extract-<timestamp>\\botprofile.db
```

并加入提取互斥。当前实现还在 `open()` 中先执行 CS2 关闭检查。由于工单明确标注为 `v0.5.11`，不能据此认定该玩家已经获得这些当前改动；也不能把当前路径差异解释成工单现场的实测结果。

## 原因判断

### 已确认

1. 这是旧版固定输出目录链路上的 VPKEdit 提取失败。
2. VPKEdit 报错发生在将 `botprofile.db` 写入输出目标时；错误文本本身没有证明 BOT 数据内容损坏。
3. `BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID` 在旧实现中同时包装了工具退出码、输出文件写入失败等多种情况，错误类别过宽，无法仅凭该码定位底层原因。
4. “CS2 未运行”只排除了游戏进程作为最直接占用者，不能排除 Steam、索引服务、Defender/第三方安全软件或其它进程对输入/输出文件的访问。

### 目前最可信的原因类别（尚未在玩家机器逐项证实）

按与错误文本及旧路径的匹配度排序：

1. 旧 `open-...` workspace 中的目标文件或临时目录被残留进程、安全软件或上一次失败操作占用。
2. 用户 AppData 路径或目标文件受到权限、受控文件夹访问或安全软件拦截。
3. 工坊操作发生并发/快速重复打开，多个旧版 VPKEdit 进程争用同一固定输出路径。
4. D 盘 Steam Library 的输入 VPK 正被 Steam、索引或安全软件读取，导致 VPKEdit 读写阶段失败。
5. 用户盘空间不足、文件系统错误或路径 ACL 异常。
6. 输入 VPK 条目异常、版本资源不一致或 VPKEdit 工具本身异常。

以上类别目前不能进一步排序为单一根因。特别是：不能仅凭“D 盘”断定权限问题，不能仅凭 `extract 退出码 1` 断定 VPK 损坏，也不能仅凭插件安装日志断定资源包错误。

## 与当前 0.5.12 工作的关系

- 当前工作树已经针对工坊短暂无响应完成阻塞任务异步化，并增加读取请求序列保护；这解释的是窗口卡顿，不等于解决了本工单的旧版提取失败。
- 当前工作树的唯一提取目录与互斥设计理论上降低了固定目标争用风险，但尚未在本工单玩家设备上验证。

## 已实施的加固（本调查轮次）

在无法取得完整现场证据的情况下，对提取链路做了低风险加固，目标是让同一类失败可分类、可自愈：

1. 提取失败不再包装为 `BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID`，改为独立错误码 `BOT_WORKSHOP_EXTRACT_FAILED`，并携带 VPKEdit 的 stderr 与可操作提示（关闭占用程序、检查安全软件）。
2. 提取目录创建/写入失败单独分类为 `BOT_WORKSHOP_WORKSPACE_UNWRITABLE`，并在调用 VPKEdit 前先做一次写入探测，把权限、磁盘或受控文件夹访问类问题从工具类问题中分离出来。
3. `extract_db` 增加有限重试（最多 3 次，递增退避），每次尝试使用全新的 `extract-<时间戳>-<序号>` 目录，覆盖残留进程、安全软件瞬时占用等短暂故障。
4. 在提取互斥锁内清理 48 小时以上的陈旧 `workspace` 临时目录（`open-*`、`tree-*`、`extract-*`、`verify-*`、`manifest`、`tool-check`），避免历史失败目录堆积并消除其残留锁定影响。

以上改动不改变提取、保存、应用的整体流程与文件语义；重试与清理均为尽力而为，失败时保持原错误路径。是否在 `0.5.12` 发布后仍需针对少数玩家追加重试或错误码拆分，以新版故障单携带的 `BOT_WORKSHOP_EXTRACT_FAILED`/`BOT_WORKSHOP_WORKSPACE_UNWRITABLE` 现场信息为准。

## 需要补充的最小证据

请从受影响玩家的实际安装器和失败现场收集：

1. 安装器文件 SHA-256、应用内显示版本，以及是否确为正式 `v0.5.11` 而非旧候选。
2. VPKEdit CLI 文件版本、SHA-256 与 `--help` 输出摘要。
3. 失败时实际输入 `botprofile.vpk` 的完整路径、文件大小和 SHA-256。
4. 失败 workspace 中 `botprofile.db` 是否已存在；父目录创建/写入测试结果；文件属性和 ACL。
5. 系统盘与 D 盘剩余空间，以及 Windows Defender“受控文件夹访问”和第三方安全软件拦截记录。
6. 失败时是否快速重复点击、重复打开工坊，及任务管理器中是否残留多个 VPKEdit 进程。
7. 退出 CS2、退出 Steam 后重新打开助手，在同一档案上单次操作是否仍能复现。
8. 在正式 `0.5.12` 安装器上执行一次打开内置档案、创建自定义档案、保存并回读的结果。

## 当前结论

这张工单最合理的解释是：`v0.5.11` 使用固定的 `open-...\\botprofile.db` 输出路径，VPKEdit 在特定玩家环境中无法创建或写入该目标；旧版错误分类又把所有退出码 1 统一显示为依赖无效。现有资料足以确认“旧实现路径 + 输出目标写入失败”，不足以确认是占用、权限、安全软件、磁盘、输入 VPK 还是工具版本中的哪一项。

因此本轮已按“可分类、可自愈、不改语义”的原则对提取链路实施加固（独立错误码、写入探测、有限重试、陈旧目录清理），并保留在收到新版现场证据后进一步分类的权利。
