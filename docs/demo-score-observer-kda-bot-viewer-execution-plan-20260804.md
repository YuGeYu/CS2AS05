# CS2 Demo：比分、观战阵营、KDA Rating、BOT 回放交接方案（2026-08-04）

> **比分专项更新：** 用户已将本次开发机会收窄为只修复 de_nuke 正确比分。下一位执行 AI必须先执行 demo-score-only-final-fix-execution-plan-20260804.md，并暂缓本方案中的 Rating、观战者和回放颜色工作，直到明确得到 CT 16 : 12 T。

> 交接对象：下一位实际执行 AI。
>
> **优先级：P0。** 用户真实测试 auto-20260804-1210-de_nuke-advent.dem：用户本人全程观战，场上是 10 个 BOT 分成 CT/T 两队。用户看到的最终比分为 CT 15 : 16 T，但应用当前又认为只有 28 个正式回合；地图回放中的 BOT 全部显示为白色圆点。执行 AI 必须先利用现有最终 Demo 完成修复、重解析和取证，再安排用户复测，不要求用户现在重新打一局。

## 1. 本轮必须交付

1. 比分与正式回合数必须正确且数学自洽；来源冲突时显示待确认和质量警告，不显示伪造比分。
2. 像 advent 这样的观看者归入“观战阵营”，保留身份，但不计入 CT/T 比分、双方人数和 Rating。
3. BOT 作为正常 player participant，拥有与真人相同的 K/D/A、Rating、事件和地图回放能力。
4. Rating 改为 KDA 主导的新版本，旧模型数值不得静默改变。
5. 地图回放按阵营着色：CT、T、观战者各自明确，BOT 不再统一白色。
6. 必须先完成 demo-watcher-no-early-parse-execution-plan-20260804.md；CS2 运行中绝对禁止解析仍是前置 Gate。

“BOT 当真人”不等于给 BOT 伪造 SteamID，也不允许按昵称合并。BOT 使用 userid/slot 等稳定身份，只是在产品统计和显示层享有与真人相同的参与者语义。

## 2. 当前事实与根因假设

### 2.1 现场已被提前解析污染

当前隔离数据库：

    C:\Users\GOPtZ\AppData\Roaming\com.aipc.cs2botimprover.preflight\demo-review\demo-review-v1.sqlite3

当前 de_nuke 记录曾出现 core job 很早完成、Demo 文件继续增长、demo_files.status=parsing、回合数为 0。不能在这个半成品 report 上直接改比分。必须先修 watcher 解析闸门，再用最终稳定文件重新解析。

### 2.2 当前代码事实

- final_team_scores 从 CCSTeam 的 first-half、second-half、overtime props 汇总比分：src-tauri/src/services/demo.rs:2570。
- round_winner_scores 按 DemoRound.winner 计数：src-tauri/src/services/demo.rs:2632。
- parse_report 无条件优先 final team props，再 fallback 到 round winners，两条来源没有一致性 Gate：src-tauri/src/services/demo.rs:3241。
- 没有 SteamID 的 event identity 会直接成为 is_bot=true：src-tauri/src/services/demo.rs:2644。
- 玩家筛选会移除 team_number=1，观战者没有正式角色模型：src-tauri/src/services/demo.rs:2907。
- 当前 Rating 是 simple-rating-v2，权重为击杀 50%、伤害 15%、生存 20%、助攻 15%：src-tauri/src/services/simple_rating.rs。
- draw-viewer.ts 已有 CT/T 颜色，但 teamNumber 为空时回退白色；用户看到全部白色说明空间点身份到队伍的映射失败，而不是只缺 CSS。
- 现有真实 Demo test 曾锁定 de_nuke 为 12:16、28 rounds、10 BOT，并识别 advent 为非参赛观战者：src-tauri/src/services/demo.rs:3701。该错误分数断言必须被事实诊断替换。

### 2.3 必须正面处理 28 与 15:16 的矛盾

每个正式回合只产生一个赢家，因此必须满足：

    CT_score + T_score <= completed_rounds

15 + 16 = 31。如果最终原始事件证明比分确实是 15:16，正式回合至少是 31，当前 28 就是 round normalization 缺失 3 回合；如果最终只有 28 个正式回合，15:16 就不可能成立。不得硬编码任一数值。执行 AI必须通过原始 Demo 和用户截图确定哪个字段错误，并在 evidence 中写清结论。

## 3. 阶段 A：从最终 Demo 建立事实基线

1. 关闭 ai_pc_fac、Vite、CS2，确认 Demo 文件不再增长。
2. 备份并 hash SQLite；保留当前半成品记录，不直接修改成 done。
3. 记录最终 Demo path、size、mtime、SHA-256、parser commit、adapter/schema/metrics version。
4. 先运行 parser diagnostic，不写生产 report。
5. 输出新的 score-identity-diagnostic.json，至少包含：
   - 所有 round_start、round_end、round_officially_ended 的 tick、winner、reason、事件序号；
   - dedupe 前后 logical/completed/unfinished round 数；
   - CT/T canonical winner count；
   - CCSTeam 每行 team number、first-half、second-half、overtime 和总值；
   - 每个身份的 SteamID、userid/slot、raw bot evidence、team number、derived participant role；
   - 当前 parser score/round count 和所有一致性错误。

运行行为和 raw diagnostic 是权威证据，不能用旧 report JSON 或现有单元测试反推事实。

## 4. 阶段 B：修复 round 与比分模型

### 4.1 Canonical round

新增独立 round normalization 纯函数：

1. 使用有效 round_start、match-start、freeze-start 建立 logical round 起点。
2. 合并同一 logical round 的重复 round_end 和 round_officially_ended。
3. 有正式结束边界且 winner 可归一为 CT/T 的 round 才计 completed。
4. 未完成末尾 round 保留为 logical round，但不计比分和 Rating 分母。
5. 半场换边不重复创建 round；team number 2/3 归一为 T/CT winner side。
6. quality 中记录重复事件、缺 start、缺 official end、缺 winner。

每个 canonical round 至少保存 start_tick、end_tick、official_end_tick、winner_side、source_event_ticks、is_completed 和 quality。

### 4.2 统一 derive_match_score

删除 parse_report 中简单的 final_team_scores.or_else(round_winner_scores)，改为单一 derivation：

1. canonical completed rounds 的 winner count 是首要、可逐回合解释的来源。
2. final team props 是交叉校验和 overtime 辅助来源，不可无条件覆盖 round winner。
3. 两者一致：scoreSource=round_winners+team_props，scoreQuality=complete。
4. 只有一条来源可靠：使用可靠来源，scoreQuality=partial，并记录 warning。
5. 两条来源冲突：summary score 返回 null，scoreQuality=conflict，warning=SCORE_SOURCE_CONFLICT。
6. score sum 大于 completed rounds：summary score 返回 null，warning=SCORE_ROUND_COUNT_CONFLICT。
7. 没有未完成 round 且 match 已结束时，score sum 应等于 completed rounds；否则质量不得为 complete。

扩展 DemoDataQuality 和 matches.quality_json：

    scoreSource
    scoreQuality: complete | partial | unavailable | conflict
    scoreWarnings
    logicalRounds
    completedRounds
    unfinishedRounds

Vue 在 conflict 时显示“比分待确认”或 --，绝不渲染 CT x : y T。

### 4.3 比分测试

- 重复 round_end 不重复计分。
- round_officially_ended 缺失时按质量规则处理。
- halftime/overtime 不重复累计。
- final props 与 winners 一致时 complete。
- props 15:16、completed 28 时必须 conflict，不能显示 15:16。
- winners 共 31 且比分 15:16 时 completed 必须为 31。
- 未完成尾回合不进入比分和 Rating 分母。

## 5. 阶段 C：观战者和 BOT 身份模型

### 5.1 拆开参与者角色与 BOT 证据

新增字段：

    participantRole: player | observer | unknown
    identitySource: steamid | userid | slot | controller | event | name
    botEvidence: true | false | null

规则：

- team_number=1 或明确 spectator 且不参与正式回合：participantRole=observer，UI 阵营为“观战阵营”。
- team_number=2/3 且有 controller/event/userid 数据：participantRole=player，不论 botEvidence。
- 无 SteamID 但有 userid/slot 的 BOT：仍为 player，有完整 K/D/A、Rating、空间点。
- 无法判断是否 BOT但有 CT/T 参赛证据：player，botEvidence=null，不丢弃。
- observer 不进入双方人数、比分、KDA 和 Rating；但保留在报告的观战分组。

isBot 可暂时保留为兼容字段，但不得再作为过滤、Rating eligibility 或 UI 差别待遇。普通 UI 不显示 BOT 标签；原始 botEvidence 只在诊断/详情中保留。

### 5.2 Stable identity

1. 真人优先 SteamID64。
2. BOT 使用 userid/slot 组合，不能用昵称作为唯一 key。
3. observer advent 使用独立 stable key，不能并入任何 BOT。
4. 同名不同 userid/slot 保留为不同参与者。
5. controller、event、spatial 必须共用同一 identity map。
6. 真实 de_nuke 应得到 10 个 player participants 和 1 个 observer，而不是 11 个双方玩家。

### 5.3 UI

- 记分板分别显示 CT、T、观战阵营。
- BOT 行与真人行样式、指标、交互一致，不显示 BOT 徽标。
- 总览分别显示“参赛玩家 10”“观战者 1”，不混为总参赛人数。
- observer 详情明确“观战者不参与 Rating”。

## 6. 阶段 D：KDA 主导 Rating

### 6.1 新模型版本

新增 simple-rating-kda-v1，旧 simple-rating-v2 保持可回读。建议透明公式：

    kda_ratio = (kills + 0.5 * assists) / (deaths + 1)
    kda_component = clamp(kda_ratio / 1.50, 0, 2.5)
    kill_component = clamp((kills / rounds) / 0.70, 0, 2.5)
    survival_component = clamp((1 - deaths / rounds) / 0.68, 0, 2.5)
    damage_component = clamp((damage / rounds) / 82, 0, 2.5)
    rating = clamp(
      0.70 * kda_component +
      0.15 * kill_component +
      0.10 * survival_component +
      0.05 * damage_component,
      0,
      3
    )

KDA ratio 占 70%，明确满足用户“更接近 KDA”的要求。deaths+1 防止零死亡无限值。执行 AI可用真实分布微调基准 1.50，但不得降低 KDA 主权重到 60% 以下，也不得再让 ADR 主导。

### 6.2 DTO 与适用范围

DemoRating 增加 kdaComponent 并保存 modelVersion=simple-rating-kda-v1。UI 显示“简易 KDA Rating · 非 HLTV 官方模型”。

- CT/T 的 BOT 和真人使用相同输入与公式。
- observer 永不计算 Rating。
- K/D/A、rounds 或可信统计缺失时返回 unavailable，不补 0。
- round/score 尚为 conflict 时 Rating 不输出 complete。

### 6.3 Rating 测试

1. 相同 rounds/damage 下，KDA 更高者 Rating 必须更高。
2. 伤害单独变高不能压过明显 KDA 差距。
3. 0 deaths 有限；全 0 K/D/A 为 0。
4. BOT 与真人相同输入得分完全相同。
5. observer 返回 unavailable。
6. v2 固定向量继续通过；v3 有独立固定向量和模型版本。

## 7. 阶段 E：地图回放阵营颜色

### 7.1 修复空间身份映射

当前 draw-viewer 已能给 team 2/3 着色，全部白色说明 PositionPoint.teamNumber 为空。后端必须：

1. 建立 raw player id -> report stable key -> participantRole/team 的统一 map。
2. 不再把 df_per_player 数值直接拼成 bot key 后猜测 report key。
3. 真人按 SteamID，BOT 按 userid/slot 映射。
4. 原始 team prop 缺失时从 report identity map 补 teamNumber。
5. observer point 保留 teamNumber=1 或 participantRole=observer。
6. 返回 team2Points、team3Points、observerPoints、unknownTeamPoints、identityFallbackCount 诊断。

### 7.2 绘制和图例

使用现有 Data-Dense Dashboard 视觉，固定 token：

    CT:       #38BDF8
    T:        #F59E0B
    Observer: #94A3B8，灰色带外圈
    Unknown:  #CBD5E1，仅诊断 fallback
    Outline:  #0B1120

- BOT 和真人使用相同队伍圆点。
- observer 使用第三种中性标记，图例写“观战阵营”。
- CT/T 不能因为 isBot 走白色 fallback。
- unknown 点必须计数；不能以白色遮掩 mapping 失败。
- 图例和 tooltip 同时写阵营文字，不能只靠颜色表达。
- 保持 focus visible、reduced motion、150-300 ms 状态反馈，不增加持续动画。

### 7.3 回放测试

- draw-viewer 纯函数验证 CT/T/observer/unknown 四类颜色和形状。
- 真实 de_nuke spatial diagnostic 必须出现 CT/T 两类非零点。
- 10 个 BOT 不得全部 white，identity mapping 覆盖率必须记录。
- Nuke upper/lower 各取一张真实 Tauri Canvas 截图。
- 播放、暂停、页面离开继续保持 RAF cleanup。

## 8. 数据版本和重新解析

1. 为 participant role、score quality、Rating v3 提升 parser adapter/metrics version，不能静默改变旧 report。
2. SQLite v7 修改前做 WAL-safe backup、hash、integrity_check。
3. 新 report 成功前旧 report 保持可读，成功后原子切换。
4. 先重解析最终 de_nuke，再回归历史 dust2/inferno，确认真人不被误判 observer、BOT不被删除。
5. 旧模型报告可继续打开；只有重新解析后显示新模型。

## 9. 自动化和真实 Gate

必跑：

    npm run workspace:check
    npm run typecheck
    npm run lint
    npm test -- --pool=threads --maxWorkers=1
    npm run build:web
    cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
    cargo check --manifest-path .\src-tauri\Cargo.toml
    cargo test --manifest-path .\src-tauri\Cargo.toml --lib
    cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings
    git diff --check

真实 de_nuke Gate：

1. raw diagnostic 明确 28/31、CT/T winner count、team props 和冲突原因。
2. summary score 与 completed rounds 数学一致；不一致时 UI 为 conflict/--。
3. 10 个 BOT 都是 player，advent 是 observer“观战阵营”。
4. BOT 输入可靠时都有 simple-rating-kda-v1；observer 无 Rating。
5. viewer 中 CT/T 两种颜色非零，BOT 不再统一白色，observer 有第三种标记。
6. no-early-parse P0 同时通过，CS2 运行中无 parser/job。
7. 使用最终 debug EXE hash 完成真实 Tauri截图和 DB 回读，不用 Web fixture冒充。

## 10. 停止条件

出现任一项，不得请求用户复测：

- 仍能在 CS2 运行中解析。
- score sum 大于 completed rounds 却显示正常比分。
- 硬编码 15:16 或 28，不做 raw diagnostic。
- observer 被删除或混入 CT/T。
- BOT 因 isBot 被排除、无 Rating、无队伍颜色。
- BOT 按昵称合并。
- KDA 新模型未版本化、缺值补 0、observer 参与 Rating。
- CT/T 空间点仍全部 white，unknownTeam 未计数。
- 只改 UI/旧 JSON，没有重新解析最终 Demo。
- 自动化、clippy、DB integrity 或真实 Tauri证据失败。

## 11. 执行 AI 最终报告

最终报告必须包含：

    结果：未完成 / 已完成，可请求一次用户针对性复测
    输入：Demo hash、parser/adapter/schema/metrics
    比分：logical/completed/unfinished、CT/T winners、team props、score source/quality
    身份：player/observer 数量、botEvidence、stable-key 冲突
    Rating：公式、版本、固定向量、BOT/observer 结果
    回放：CT/T/observer/unknown 点数、mapping 覆盖率、截图
    数据库：backup、v7/integrity、旧/新 report
    自动化：逐命令 exit code、测试数、warning 归属
    用户下一步：只允许一次针对性复测，不要求重新完整对局

所有 Gate 通过前，结论固定为：

    Demo 比分/身份/Rating/回放修复未完成，不请求用户再次 BOT 验收。
