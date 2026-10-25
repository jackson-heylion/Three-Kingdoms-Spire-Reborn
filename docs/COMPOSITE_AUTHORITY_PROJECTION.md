# 六条权威日志的统一归约与一致性检查（P1 增量）

## 既有基础与本轮增量

原有 `Game::audit_authority_bundle()` 已能逐修订号交叉核对六条日志；本轮不把这项旧能力冒充新增。新增的是**可独立调用、失败原子回滚的有类型流式归约器** `CompositeAuthorityProjector`，并将原有批量审计和指定版本投影改为复用同一实现，避免两套规则长期漂移；另增加真实 CLI 工具与双平台篡改拒绝 E2E。

## 能做什么

`Game::audit_composite_authority()` 对六条已提交动作边界日志分别调用现有的独立归约器：

| 日志 | 可重建的提交后状态 |
| --- | --- |
| resource_history | 生命、生命上限、金币 |
| rng_history | RNG 内部状态（非每次抽样因果） |
| battle_resource_history | 当前战斗回合、军令、护甲 |
| foe_board_history | 敌军身份、血量、状态、意图 |
| card_zone_history | 永久牌组及当前战斗有序抽牌、手牌、弃牌、消耗牌堆 |
| combat_effect_history | 持续军略、动态监听器 |

`event_stack::authority::project(&game)` **只读取六条日志，不读取当前 Game 快照、成功动作日志或随机算法**。它按每个成功动作修订号同步推进六个归约器；任一步都要求六条日志齐全、修订号相同，四份战斗视图的在场性、楼层、回合一致。任一步失败均原子拒绝，不会部分推进其余五条日志。返回的 `AuthorityView` 可独立审查；随后 `verify_snapshot(&game)` 与 Game 当前状态逐字段对照。

相比只单独运行六个 `audit_*`：**即使每条日志自身连续、合法，且最终快照完全一致，只要任一中间修订号混用了不同战斗回合或楼层的日志，仍必须拒绝**。非法或缺失的旧版 V2 日志不能凭空补全，但不改变旧存档正常读取兼容性。

## 真实命令行验收

```bash
cargo build --bin tks --bin tks-authority
python3 tests/authority_cli_e2e.py target/debug/tks target/debug/tks-authority
target/debug/tks-authority /path/to/save.json
```

双平台 CI 对五职业×飞升0/10各启动真实 CLI、保存首层战斗，再用独立进程归约六份日志；验证当前快照被改写和某条日志独立合法、却与其余日志回合冲突时均被拒绝。Rust 测试另外覆盖重启、版本0、缺失日志、修订号缺口、中间回合冲突和失败后无状态污染的重试。

## 不可冒称的验收

这是六个 **action-boundary projections** 的组合，仍不是完整 Game 的 **Event Sourcing**：地图、路线、剧情、遗物、赐福、终局战功、完整触发时序及其它字段还没有可独立归约的权威事件；这些日志也不能重建一次动作内部每个微状态。

日志和快照均可被同一攻击者同时改写；跨流一致性**不是真实性证明**。如需抗篡改，应另行对已知可信的版本检查点进行外部保存、核验，不能把同目录的摘要当作独立信任根。完整 P1、真人试玩、15套成熟构筑与120回逐回异本史料校勘仍未验收，禁止发布正式1.0。
