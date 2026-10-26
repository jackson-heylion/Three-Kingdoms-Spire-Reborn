# P1：卡牌、回合与药剂的已结算事件因果审计

新增 `src/event_projection/action_causality.rs`，由 `Game::audit_event_archive_authority` 读取完整战斗事件史。它与现有敌军死亡/召唤校验并列，独立于 `Action` 重放、RNG 和最终 `Game` 快照，只核查**已成功提交**的事件语法：

1. 每个 `CardPlayed` 必须恰好对应同源 `CardResolved`，中间允许伤害、状态、抽牌、被动等真实效果；拒绝孤立结算、嵌套打牌、源 ID 篡改、卡牌尚未结算便封场或跨战斗。
2. 每个 `TurnEnded` 之后必须有且只有一个 `TurnStarted`，或在敌军攻击、毒伤、反伤、粮尽导致终局时以 `BattleEnded` 收尾。拒绝无前置的开回合、重复结束回合和提交边界遗留半个回合。
3. `PotionUsed` 的数量必须为1，来源限于10种正式药剂；火油/毒药指定规范敌军槽位，弱化药指定 `all_foes`，其余药剂不允许伪造目标。药效仍由游戏原逻辑处理，不改变药剂强度。
4. 每次遇到新战斗都要求上一场完整封场；当前最后一场可以仍在进行，但不能在完整提交的事件档案中留下半次卡牌或半次回合行动。检查过程只读、线性复杂度，不修改存档格式或游戏数值。

Rust 回归：孤立/错配/嵌套卡牌，非法回合时序与半提交，药剂来源/目标/数量篡改，毒伤致命回合封场，以及五武将×两种子真实首回合、V2 存档往返与九域权威审计。运行 `cargo test --lib event_projection::action_causality`，并执行全量 `cargo test --all-targets`、BDD、模拟和最终 macOS/Linux GitHub Actions。

**验收边界**：这是一段已结算事件语法验证，不能从战斗日志独立恢复牌堆、HP、敌军意图或整个 Game；也不能证明日志未被同一攻击者同步伪造。它不替代外部插件事件 ABI、所有道具组合、15套构筑自然成型、120回异本校勘或真人试玩。最终机器验收只认本 PR 最新 SHA 的双平台 CI，不发布 1.0。


## 2026-10-26：按提交修订号封闭卡牌和回合生命周期

此前只对最终完整档案检查 `CardPlayed → CardResolved`、`TurnEnded → TurnStarted/BattleEnded`；攻击者可将一张牌的打出和结算伪装成跨两个成功动作提交，即使最终档案顺序正确，也违反动作原子性。现将同一事件语法重构为 `ActionLifecycleProjector`，由 `Game::audit_event_archive_boundaries()` 按持久化 `TelemetryEvent.archive_len` 游标逐修订消费。每个成功动作结束必须不存在未结算卡牌或半个敌方回合，拒绝事件不移动归约游标。与现有九域权威、战斗窗口和计数核对共同执行，线性遍历档案，避免每修订重扫整个历史。

新增合成事件测试覆盖打牌/回合拆分、非法结算的原子拒绝；真实关羽出牌、回合结束后篡改历史修订游标，最终事件流本身仍合法，但逐修订审计必须拒绝；五职业两种子存档往返继续逐修订验证。运行 `cargo test --lib event_projection::action_causality`、`cargo test --all-targets` 和双平台完整 CI。

**边界**：这是提交边界的事件语法与计数一致性，不是事件源恢复整个 `Game`，也不保证攻击者无法同时伪造所有日志、游标及存档。旧 V2 无游标时仍返回未认证状态；不可将其误报为完整 P1。
