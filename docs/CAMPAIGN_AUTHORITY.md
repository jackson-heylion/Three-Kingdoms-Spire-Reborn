# 第七条权威日志：战役地图、路线、剧情与道具（P1 增量）

## 可验证的新增能力

`campaign_history` 在新战役修订号 0 保存初始地图、路线、遗物、药剂、赐福、战法、商店/奖励候选、剧情标志及跨幕状态。每次成功 Action 追加一次边界事件；未改变战役状态时只保存修订号，改变时保存有类型的完整战役域状态。非法 Action 不提交事件。

`CampaignProjector` 只读取该日志即可恢复指定边界的战役域状态；不调用规则、不消耗 RNG、不读取可变 Game 快照。七域 `CompositeAuthorityProjector` 原子归约七条日志，每个修订号核对战役 stage/floor 与战斗日志的坐标，最终与 Game 快照对照。日志缺失、重复/跳号、跨域阶段矛盾、伪造快照拒绝认证。

## 复验

```sh
cargo test --test campaign_authority
cargo test --test composite_authority
cargo test --test authority_bundle
python3 tests/authority_cli_e2e.py target/debug/tks target/debug/tks-authority
```

覆盖五职业×飞升 0/10 的真实行动、序列化重启、无动作日志独立投影、篡改、旧 V2 缺失新日志、非法动作原子回滚。CI 以最终 SHA 的 macOS/Linux Rust 与真实进程 E2E 为准。

## 兼容性与未验收

旧 V2 存档没有 `campaign_history` 时仍能读取和继续游戏，**不补造过去日志**；旧档的七域认证会明确失败。已有六域日志仍保留。新日志未改变 RNG、战斗算法或存档版本号。

这是第七条**动作提交边界**权威记录，不是每个剧情/道具操作的细粒度领域事件；战斗事件、随机抽样与地图剧情仍不能单凭这些记录独立恢复完整 Game。战功簿、日志、运行时状态等未纳入，外部真实性仍依赖独立可信检查点。真人试玩、十五套成熟构筑、120回正文/异本史料审校及完整国际化继续未验收，禁止发布正式1.0。

## 2026-10-25：不可变战役身份认证

第七域的初始事件新增 `CampaignIdentity`：武将、种子、飞升难度、模式与稳定 `run_id`。自定义/每日战役必须先确定模式和 ID，再锚定修订号 0；之后任何成功动作均不得改变身份。归约器在每个动作边界拒绝身份偷换，最终与 Game 快照逐字段比对。这样不能只修改种子、武将或模式而继续通过七域认证；五职业×三模式的 Rust 集成测试及双平台真实 CLI 篡改测试覆盖此约束。

兼容边界：旧 V2 的 `campaign_history` 若无身份字段，仍可读取、继续行动、保存；缺失身份不可凭空补造，也不应声称通过新增的身份认证。此项不认证 `profile_start_receipt`、战功簿、历史战斗事件、诊断日志与运行时临时状态；独立可信检查点仍是外部真实性前提。**仍非完整 Game Event Sourcing。**
