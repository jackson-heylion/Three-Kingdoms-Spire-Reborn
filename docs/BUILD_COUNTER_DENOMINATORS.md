# P1 反制敌军的实际遭遇分母：运行级计数

2026-10-20 增量。此前 `tks-sim --build-archetypes` 报表只提供
`counter_last_encounter_deaths`，把最终战败前存活敌军名匹配到
蓝图声明的 `counter_enemy`。没有该敌军的**遭遇分母**，不应推断
「指定克制 Boss 使某构筑失败多少比例」。

现在 `BuildCell` 额外输出：

- `baseline_counter_exposed_runs`：普通起始牌组实验中，战役至少一次出现
  该反制敌军的**不同战役局数**；
- `blueprint_counter_exposed_runs`：等起始牌数、替换成型卡夹具实验中，
  战役至少一次出现该反制敌军的**不同战役局数**；
- `counter_last_encounter_deaths`：保留原本最后战败遭遇口径，不改历史含义。

每局通过实际 `Stage::Combat` 的战场敌军快照收集所有出现过的敌军稳定 ID；
跨楼层去重，不能假设「进入了某一层就遇到某 Boss」，没有接战的局数
也不会进入分母。排序后的唯一 ID 可跨平台确定性重现，不改变机器人选牌、
交战、RNG、难度或卡值。对每个 `BuildCell`，两个新值范围均为
`0..=samples`，它们是观察暴露的跑局数，不是总战斗遭遇次数。

**为什么仍不能宣布已验证 Boss 克制**：
一次遭遇可能逃过或在后来别处失败；一局也可能多次遭遇同类敌军；
最后败局口径不能提供该遭遇的 HP 损失、回合数和击败率。
真实 Boss 对照实验还需要记录每次出现的时点、开始与结束 HP、
实际敌军意图、胜负、资源与等价场景种子。自然奖励构筑与真人试玩也尚未完成。
报表增加了可审计的分母，但不应把
`counter_last_encounter_deaths / blueprint_counter_exposed_runs`
直接标成「Boss 单战胜率」。

复算：
```bash
cargo run --release --bin tks-sim -- --build-archetypes 2 tks-builds.json
```

继续保留 `schema_version=1`，因为只是给现有 `BuildCell` 增加字段，
旧读取脚本仍读取原字段；需要严格固定 JSON 结构的外部消费者应更新解析契约。
本增量须以 macOS/Linux CI 最新提交的通过结果验收，不代表完整 P1。
