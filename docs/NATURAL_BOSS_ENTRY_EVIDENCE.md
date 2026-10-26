# P1：自然构筑在真实 Boss 入场时是否已经成型

## 修复的证据口径

旧 `tks-sim --natural-builds` 仅检查**终局牌组**是否持有构筑核心牌。即使角色从未遇到 Boss，或在 Boss 战后才获得核心牌，也可能被记为「已成型」。终局牌组不等于 Boss 对位时的可用牌组。

现在只对 `DeckTreatment::Natural` 采样：在真实 `Node::Boss` 战斗开始时冻结该战役已有的核心/辅助牌数量、Boss 敌军 ID、楼层；战斗结束时补充该次战斗是否存活。没有注入牌、改写敌人、指定路线、跳过前置关卡或额外消耗 RNG。

每个构筑 × 飞升难度的报告同时保留终局持牌指标与新指标：

- `samples`：该条件的自然战役数；`complete_core` / `complete_support` 只代表**终局**。
- `boss_battles`：真正进入的 Boss 战次数，可能为零；不把未抵达 Boss 的战役算作「未成型 Boss 战」。
- `boss_core_formed`：Boss **入场前**四张核心牌均已获得的 Boss 战次数；`boss_full_formed` 还要求两张辅助牌。
- `boss_wins`：这些真实 Boss 战存活次数；`boss_core_wins` 是核心已成型且存活次数，不是条件胜率的独立样本数。
- `boss_floor_sum` / `boss_core_owned_sum` / `boss_support_owned_sum`：用于计算实际到达 Boss 时的平均楼层与部分成型程度。
- `counter_boss_battles` / `counter_boss_wins`：仅当该 Boss 的实际敌军 ID 与构筑登记的 `counter_enemy` 相符；如果反制对象是精英而非 Boss，合法结果可以为零。

同一战役可能有多次 Boss 战，故 Boss 战次数不是独立战役数，不能直接套用二项独立样本置信区间。Boss 战生存也不等于三幕通关。结果只代表固定策略、种子与自然奖励；机器人仍可能不会使用构筑关键连招。

## 复验

```bash
cargo test --bin tks-sim natural_plan_report
cargo run --release --bin tks-sim -- --natural-builds 1 /tmp/natural-boss-entry.json
```

`--natural-builds 1` 会遍历15套构筑×11难度×基线/偏好策略，共330局正常开局；时间开销较高。运行时不预设必胜、必成型或必遇到 Boss，零分母保持原始计数，不伪造百分比。报告结构由版本1升为2，新增字段不影响 Game V2 存档格式。

本次代码由 GitHub Actions 最终 SHA 的 Ubuntu/macOS Rust、BDD、CLI E2E 与跨平台确定性检查验证；无本地 Cargo 时不能声称本地已运行。15套成熟构筑、自然胜率显著性、完整 Game Event Sourcing、真人试玩、120回异本史料校勘和正式1.0仍未验收。
