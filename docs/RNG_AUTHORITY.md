# P1 RNG 动作边界权威状态日志（2026-10-25）

新战役以 `rng_history` 记录初始随机状态，并在每次成功 `Game::apply` 后追加 `Committed { revision, state }`。没有随机抽样的动作仍记录相同状态，以保持全局修订号对齐。拒绝动作不得污染 RNG 或日志。

`RngProjector` 逐条归约，拒绝重复起点、零状态、修订号跳跃和重复提交；失败不推进已提交状态。独立 `Game::audit_rng()` 检查终态，六日志 `Game::audit_authority_bundle()` 同步检查 RNG、持久资源、战斗资源、敌军、卡牌、持续军略与监听器；`project_authority_boundary(revision)` 额外返回 `rng_state`。旧V2存档没有 RNG 日志时仍可读取和动作重放，但不得假称六日志审计通过，也不补造历史。

复验：`cargo test --test rng_authority`、`cargo test --test authority_bundle`、`cargo test --all-targets`、`cargo clippy --all-targets -- -D warnings`。

**边界**：记录的是成功动作之后的 RNG 状态快照，不是逐次抽样事件，也不能证明状态确由合法次数的 xorshift 转换得到；协调篡改仍可绕过校验。完整 Event Sourcing 还缺抽样因果、地图、奖励、道具、剧情、可信锚点及跨版本迁移。真人试玩、120回全文校勘、完整国际化和 P1 均未验收，不发布正式1.0。
