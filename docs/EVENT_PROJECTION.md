# 事件档案独立投影：可复算的战斗事实（P1 增量）

本增量增加 `event_stack::projection::project(&[RunEvent])`。它只读取完整战斗事件档案，不依赖 `Game` 快照、动作日志、随机种子或 RNG，按 encounter/floor 重建可审计的**事件事实投影**：

- 每场事件数、序号区间、打出卡牌次数；
- 已结算敌方 HP 伤害、英雄 HP 承伤（不是攻击原始伤害）；
- 击败与召唤敌军次数、敌军特殊技能次数；
- 动态事件取消和替换次数。

投影先调用既有 `verify_record` 拒绝序号缺口、跳跃的 encounter、非法楼层和不从首场开始的档案。已结算伤害不得为负；累计计数使用 checked addition，不静默饱和。空档案产生空投影，**不能**视为战役已完整验证。

回归见 `tests/event_projection.rs`：跨两场合成事件、序号缺口和负伤害拒绝、序列化往返、真实 `Game::apply(Action::Choose)` 的战斗档案在 V2 存读前后投影一致，以及不依赖动作日志的事实重建。

**严格边界：这不是完整 Event Sourcing。** 当前事件缺少手牌/牌堆、军令、敌军意图、监听器生命周期、RNG、奖励与地图等所有权威状态变更，无法只凭事件恢复 `Game`；投影不校验档案真伪，篡改金额且保持合法序号仍可得到不同投影。完整状态重建须继续设计完整状态事件、确定性归约器（reducer）、初始状态/快照锚点及跨版本迁移测试。不能以此代替 `verify_action_replay()`、真人试玩、120回正文校勘、全部流派平衡或P1正式验收。

复验：`cargo test --test event_projection`；合并前以最终提交的 macOS/Linux 全部必要 CI 成功为准。

## 2026-10-24：增量归约器与原子失败语义

新增 `CampaignProjector::push(&RunEvent)` / `projection()` / `finish()`，允许按事件顺序分批消费档案，不必将所有事件同时载入内存。归约器只保留每场汇总及最后一个 `(encounter, floor, seq)` 游标；`project(&[RunEvent])` 复用同一实现，避免批处理与流式统计口径漂移。事件坐标校验与 `verify_record` 共享同一检查函数。

每次推入先验证序号、楼层和跨战斗连续性，再在临时副本计算计数；缺口、重复、跳场、非法楼层、负已结算伤害或溢出均不得改变已提交投影或游标。调用方可以修正同一序号后重试。新增3项集成测试，涵盖跨战斗分批等价、非法输入原子拒绝、真实V2事件档案逐条归约。

**边界不变**：流式消费不是持久化事件存储，也不是完整状态重建。它不能从缺失的历史事件推算状态，不能验证来源真实性，更不能重建牌堆、生命、军令、RNG、地图和监听器。完整 Event Sourcing 仍需为所有权威状态变更定义可回放事件及版本迁移规则。

复验：`cargo test --test event_projection`，以及最终PR SHA的macOS/Linux CI。
## 2026-10-24：版本化投影检查点与增量恢复

`CampaignProjector::checkpoint()` 导出 schema_version=1 的 `ProjectionCheckpoint`，包含已提交的逐战斗投影；可经 JSON 持久化后由 `CampaignProjector::resume()` 恢复游标，继续消费**紧接的下一条** `RunEvent`。不需要重新处理已归约的事件。恢复时校验版本、战斗编号连续、楼层非递减且合法、各战斗从序号1开始、事件数与末序号一致，以及互斥事件类别计数不超过事件总数。空检查点可恢复为初始归约器；跳号、重放和非法检查点均拒绝。

这只是**统计投影的进度检查点**，并非 `Game` 的 Event Sourcing 存档，也不能证明事件来源真实、投影统计未遭篡改。由于现有事件没有显式的战斗结束封口标记，归约器不能识别某场最后几条事件被截断后直接接入下一场的情况；本轮只验证同一场内跳号。可信恢复仍须保留原始事件日志，未来应补充事件链完整性校验、权威状态事件和跨版本迁移。当前 schema 版本不兼容时明确拒绝，绝不静默假定可迁移。

复验：`cargo test --test event_projection`；以最终提交的 macOS/Linux CI 为合并前提。

## 2026-10-24：战斗终止封口（Encounter End Seal）

新战役在胜利和战败结算前写入终止事件 `BattleEnded`，其中胜利为 `source=victory, amount=1`，失败为 `source=defeat, amount=0`，且无目标。封口仍保留在完整 `event_history`，不会因战斗对象销毁而丢失。投影增加 `closed` 字段；封口后同一战斗若继续出现事件，或封口内容非法，归约器原子拒绝。

`project_sealed(&[RunEvent])` 要求**每场**都有合法终止封口，能发现整场末尾被截断或直接跳到下一场的缺口；普通 `project` 仍兼容旧V2存档和进行中的战斗，不能拿普通投影冒充完整性验收。检查点 `closed` 采用serde默认false以兼容旧JSON。新增合成档案缺尾/伪封口/封口后追加/检查点续接测试，以及真实游戏胜利后封口、存档往返单测。

**限制**：封口不等于防篡改签名。伪造者可同时重写事件和封口；当前没有独立持久化的可信事件链锚点，也没有完整的权威状态事件，仍无法从事件单独恢复`Game`。新战役进行中的战斗不应通过严格封口检查。旧存档不补造封口，不以此声称完整Event Sourcing。

复验：`cargo test --test event_projection` 和 `cargo test encounter_seal_regression`；合并须等待最终SHA真实macOS/Linux CI。

## 2026-10-24：严格流式封口与检查点续接

新增 `SealedCampaignProjector`，保留原 `CampaignProjector` 的旧档宽松读取语义。严格模式在**下一场第一条事件进入时**即拒绝上一场缺失 `BattleEnded`，而非等整个档案归约完才发现；拒绝不推进游标、不修改统计。末场未封口或空档案在 `finish()` 时拒绝。`project_sealed()` 改为复用同一严格流式实现，避免批量与增量的边界规则分叉。

严格模式支持 JSON `ProjectionCheckpoint` 恢复：允许最后一场尚未结束，但**所有更早的战斗必须封口**；含旧式未封口历史战斗的宽松检查点不能被冒充为严格审计结果。新测试验证跨战斗截断即时拒绝、原子失败、封口后续接、未封口末场拒绝，以及旧检查点的严格恢复限制。

仍只重建战斗事实统计，不包含完整 `Game` 权威状态；封口与检查点都不提供签名、防篡改、完整 RNG/牌堆恢复或可信外部锚点。复验：`cargo test --test event_projection`，最终 SHA 的 macOS/Linux CI 全部成功前不合并。

## 2026-10-24：终局事件归约与战功统计只读对账

`BattleEnded` 现在除了标记 `closed`，还在事件投影中保留 `outcome: victory/defeat`。新增 `CampaignProjection::outcome_totals()`，仅凭有序事件归约战斗胜负、已出牌次数及至多一场尚未结束的战斗；`reconcile_telemetry(&CombatCounters, active_battle)` 可与存档的战斗开始/胜负/出牌计数及当前战斗存在性逐项核对。计数溢出、封口与结果矛盾、非末场未封口、旧检查点缺少胜负证据时均明确报错，不能把未知胜负当作胜利。该对账是显式只读 API，不会自动修改存档或战功簿。

兼容性：`ProjectionCheckpoint` 仍为 schema_version=1；旧 JSON 缺少 `outcome` 时可恢复普通统计，但封口仅表明战斗结束，无法推断胜负，故 `outcome_totals()` 拒绝给出完整结果。新检查点保留终局结果。此校验仅比对两个可被同时伪造的数据来源，不是防篡改签名，也不能由事件重建 HP、牌堆、RNG、地图或监听器。旧档与正在进行的战斗不能冒称严格封口完成。新增 `tests/event_outcome_audit.rs`，复验命令：`cargo test --test event_outcome_audit`；最终SHA须通过真实macOS/Linux CI后才可合并。

## 2026-10-24：敌方攻击伤血溯源与统计对账

事件流的 `DamageReceived` 既记录敌军攻击，也记录吕布血价、持久战粮尽等非敌方伤血；而 `CombatCounters.hero_hp_damage` 只累计 `enemy_attack()` 结算伤血。此前 `reconcile_telemetry()` 没有检查这个重要计数，不能直接拿全部 `hero_hp_damage` 对账。

本轮为 `EncounterProjection` 新增 `foe_attack_hp_damage: Option<u64>`，仅对 `foe:0..3` 的已结算 `DamageReceived` 累加，保留原有全部来源的 `hero_hp_damage`。只读对账新增敌方攻击伤血核验，异常明确拒绝；累计使用 checked addition。旧版检查点缺少此字段时保留 `None`，即使恢复后继续收到事件也不能倒推历史，故旧检查点的伤血对账明确失败，避免把未知历史冒充已验证。恢复时拒绝敌方攻击伤血超过全部伤血的矛盾统计。

新增 `tests/event_damage_telemetry_audit.rs`：合成来源混合、错账拒绝、旧检查点保守恢复、伪造计数拒绝、真实战斗敌方攻击与V2往返。此增量仍不恢复HP当前值、牌堆、RNG或其他权威状态，不提供可信来源签名，也不代表P1完成。复验：`cargo test --test event_damage_telemetry_audit` 及最终HEAD双平台CI。
