# P1：战斗事件死亡与召唤的因果校验

此前九域权威审计对事件序号、战斗终局、遥测和动作边界进行交叉检查，但不强制证明每条 `FoeDefeated` 有相同来源、相同敌军槽位的正数结算伤害。新增 `src/event_projection/causality.rs`，在 `audit_event_archive_authority` 内直接检查完整 `event_history`：

- 击杀必须紧跟同源、同槽位、实际伤血大于0的 `DamageDealt`；重复击杀或阵亡后继续扣血不合法。
- `FoeSummoned` 必须恰好一名、规范 `foe:0..3` 槽位且同场不可重复/复活；英雄 `DamageReceived` 只能指向 `hero`，不能为负。
- 遇到新战斗清空敌军槽位生命周期；严格检查 encounter/floor/seq 连续性。失败只拒绝审计，不修改快照。

新增 Rust 单测覆盖真实开局、V2 往返、九域权威核验，以及来源/目标伪造、孤立死亡、零伤击杀、重复死亡、死后扣血、错误召唤和英雄受伤字段。运行：`cargo test --lib event_projection::causality`、`cargo test --all-targets`。

边界：这是已结算事件的**可观察因果**，不是引擎数值复算、密码学认证或完整 Game 事件源重建。旧 V2 无全量事件史时仍可加载，但不能获此审计结论。最终必须等待本 PR 最终 SHA 的 macOS/Linux CI；没有本地 Cargo 不可宣称已编译。P1 其他阻断和真人体验、120回全文/异本审校仍未验收，不发布 1.0。
