# CS2 Demo：比分唯一问题最终修复方案（2026-08-04）

> 交接对象：下一位实际执行 AI。
>
> **本次只有一个开发机会，本方案只允许修复比分。** 暂停 Rating、BOT 身份、观战阵营、地图回放和其他功能。目标 Demo 为 auto-20260804-1210-de_nuke-advent.dem，最终必须得到：
>
>    CT 16 : 12 T
>
>    completed rounds = 28
>
> 16 + 12 = 28，三者必须同时成立。

## 1. 为什么之前多轮仍然错误

当前代码同时存在两条比分来源：

1. round_end winner 事件计数，经过 winner_team_number 和 team_label 转成 CT/T。
2. CCSTeam.m_scoreFirstHalf、m_scoreSecondHalf、m_scoreOvertime，从多个 dataframe 行取最大值。

parse_report 曾经优先使用 final team props，再 fallback 到 round winners；这两条来源没有固定唯一权威，也没有可靠证明 dataframe 中的 team number 与半场字段属于同一队伍快照。

当前 team_label 还固定把 team 2 认作 T、team 3 认作 CT；final props 可能返回与 UI 方向相反的 12:16。UI 又固定把第一个 summary 值显示为 CT、第二个显示为 T，于是错误会稳定地出现在页面上。

本场 16 + 12 = 28，说明回合数和正确比分是自洽的。此次不能再用错误的 12:16、15:16 测试断言，也不能通过硬编码交换 UI 文案解决。

此前 Demo 又出现过 CS2 运行中提前解析半成品，造成 rounds=0。执行 AI必须先使用 watcher P0 方案修复后的最终稳定 Demo；不能在半成品 report 上修比分。

## 2. 唯一修复原则

### 2.1 Canonical round winners 是主来源

新增纯函数 derive_canonical_score，输入 canonical rounds，输出 CT score、T score、completed rounds、source、quality 和 warnings。

规则：

- 每个 canonical completed round 只允许一个 winner_side：CT 或 T。
- duplicate round_end、round_officially_ended、重复 tick 只计一次。
- 已经归一化为字符串 CT/T 的 winner 不得再次按整数 team number 翻转。
- 缺 winner 或未正式结束的末尾回合不计入比分。
- 本场必须得到 CT winners=16、T winners=12、completed=28。

### 2.2 Team props 只能校验，不能覆盖

保留 CCSTeam props 作为诊断和交叉校验：

- 优先读取 CCSTeam.m_iScore，同时记录 m_iTeamNum、m_szTeamname、first-half、second-half、overtime 的原始行和来源 dataframe。
- props 与 canonical winners 一致：score_quality=complete。
- props 侧别相反但总和相同：保留 canonical 16:12，写 TEAM_PROPS_SIDE_MISMATCH，不允许覆盖。
- props 数值与 winners 不一致：保留 canonical score，写 partial/conflict warning。
- 只有 canonical winners 缺失时才允许 props fallback，并标记 partial/unavailable。

禁止继续执行“总和相同就 conflict 并清空比分”。本场 12+16 与 16+12 总和相同，但 CT/T 方向不同，必须保留 canonical 方向。

### 2.3 强制一致性

生成 report 前必须满足：

    ct_score + t_score == completed_rounds

本场自动化必须断言：

    summary.team_a_score == 16
    summary.team_b_score == 12
    summary.total_rounds == 28
    score_source starts with "round_winners"
    score_warnings 不包含 SCORE_ROUND_COUNT_CONFLICT

不能得到 16:12 时测试必须失败并保存 raw diagnostic；不能改成 -- 后声称本次修复完成。

## 3. 允许修改的范围

只允许修改：

- src-tauri/src/services/demo.rs 的 score resolver、round winner normalize 和真实 Demo test；
- src-tauri/src/models/demo.rs、src/types/demo.ts 的 score quality 兼容字段；
- src/views/DemoReviewView.vue 的比分质量文案；
- score 纯函数测试、docs 和 evidence。

禁止修改：

- simple_rating.rs；
- observer/BOT participant role；
- MatchViewer2D.vue、draw-viewer.ts、空间 parser；
- watcher/coordinator 行为；
- 地图资源、CSS 主题、scoreboard 窗口；
- vendored demoparser。

## 4. 执行步骤

### 4.1 先写失败测试

覆盖以下固定向量：

1. 28 canonical rounds，CT=16、T=12，输出 exactly 16:12:28。
2. 同一 round 同 tick 重复 round_end 只计一次。
3. raw winner 字符串 CT/T 不再被整数映射反转。
4. props=12:16、winners=16:12 时输出 winners=16:12，并写 side mismatch。
5. props=16:12、winners=16:12 时 quality complete。
6. score=15:16、completed=28 时必须失败，不能写 summary。
7. 未完成末尾回合不进入比分。

### 4.2 修复 score resolver

- parse_report 使用 canonical winner_side 计算 summary。
- final_team_scores 不再用 half props 的 max 值直接决定 summary。
- 必须保留 raw props 诊断，不能删除冲突证据。
- team_a/team_b 的数据库值固定对应 CT/T，Vue 只能按这个契约显示。
- 删除或改正旧的 12:16/15:16 真实 Demo 断言。

### 4.3 只在必要时更新质量字段

matches.quality_json 至少记录：

    canonicalCtScore
    canonicalTScore
    propsCtScore
    propsTScore
    scoreSource
    scoreQuality
    scoreWarnings
    completedRounds

当 canonical winners 可靠但 props 侧别冲突时，页面仍显示 CT 16 : 12 T，并显示可读质量提示；不能把可靠比分清空。

## 5. 真实 Demo 重解析与证据

1. 关闭 CS2 和旧 ai_pc_fac，确认 Demo 不再增长。
2. 备份并 hash SQLite，保留旧半成品 evidence。
3. 使用 watcher P0 修复后的最终稳定 Demo 运行一次 score diagnostic。
4. diagnostic.json 必须包含 Demo hash、全部 round_end winner/tick、canonical rounds、CT/T count、CCSTeam 原始 props、最终 summary 和 warnings。
5. 用同一最终 EXE hash 重新解析并写入 SQLite。
6. 验证 report_json、matches、match_rounds 三者均为 CT 16、T 12、28 rounds。
7. 真实 Tauri 页面显示 CT 16 : 12 T；不使用 Web fixture 冒充。

执行 AI必须能用 diagnostic JSON 解释：之前为什么产生 12:16/15:16，这次为什么固定得到 16:12。只报截图不算完成。

## 6. 自动化 Gate

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

所有命令保存 exit code 和日志。Cargo 串行运行；不修改 vendor warning。

最终必须全部通过：

- de_nuke 显示 CT 16 : 12 T；
- completed rounds=28；
- 16+12=28；
- canonical winners 与 summary 一致；
- props 不覆盖或反转 canonical side；
- 旧错误比分断言已删除；
- SQLite v7 integrity_check=ok；
- report_json、matches、match_rounds 一致；
- Rust/TS score tests、fmt、check、clippy、diff-check 通过；
- 真实 Tauri 页面和数据库/API一致。

## 7. 停止条件和禁止事项

出现任一项，结论必须是“比分专项修复未完成，不请求用户再次验收”：

- 仍使用半成品 Demo。
- 仍存在 12:16/15:16，或 score sum 不等于 28。
- 通过硬编码 Demo 文件名或交换 UI 文案得到 16:12。
- props 与 winner 冲突时覆盖 canonical score。
- 没有 raw diagnostic。
- 只改页面，不改 DB/API/report。
- 任一自动化、SQLite integrity 或真实 Tauri 证据失败。

本次不执行用户复测、不重新录制、不修改 Rating/BOT/viewer/watcher，不 commit、push、release、deploy 或构建 installer。

## 8. 最终报告模板

    结果：比分专项修复未完成 / 已完成，de_nuke 为 CT 16 : 12 T
    Demo：path、size、mtime、SHA-256
    Canonical rounds：logical、completed、CT winners、T winners
    Team props：原始行、归一结果、侧别冲突
    Report/SQLite：summary、quality、matches、match_rounds、integrity
    自动化：逐命令 exit code
    证据：score-diagnostic.json、最终 report、DB hash、Tauri 截图
    未执行：其他功能、用户复测、installer、commit/push/release
