# RNG 调用与同步事件观察者的精确归属（P1 增量）

## 解决的问题

此前 `tks-attest --rng-trace` 已记录每次 `Game::random()` 的源码位置、上界、结果、RNG 前后状态，以及发生时的战斗事件游标。游标只能证明先后顺序，不能将“最近一条事件”当成随机调用的父事件。

本次为每个离线重放的 `RngCall` 增加 `dispatch_parent`。仅当 RNG 调用发生在 `Game::combat_event` 正在同步派发某个事件的观察者内部，才填入该事件的 `encounter/seq` 坐标；普通卡牌效果、开局洗牌、地图生成、独立动作和同步派发之外的抽样一律为 `null`。若子事件嵌套派发，取当时真正执行的观察者，不猜测时间上最近的事件。

归属验证同时检查：成功动作日志与 RNG 边界一致、逐次随机结果与 xorshift64* 一致、事件档案与完整 Game 重放一致；非空父坐标必须指向**本次动作已经发出的真实事件**。旧档缺事件档案时仍可沿用原有 RNG 抽样统计，但所有父坐标保持 `null`；不能声称获得完整事件因果审计。正常游玩、随机算法和 V2 存档字段不变。

## 可复验

```bash
cargo test --test rng_dispatch_parents
cargo test rng_runtime::tests::shuffle_inside_power_observer_carries_exact_event_parent
cargo run --bin tks-attest -- /path/to/save.json --rng-trace
```

单元测试构造 `CardResolved → ThoughtSpring 观察者 → 牌堆重洗 RNG`，确认随机调用的父坐标精确指向 `CardResolved`；随后在观察者之外再次抽样，必须得到 `null`，不得继承旧父事件。集成测试覆盖五职业×飞升0/10、存档往返、事件篡改与动作日志截断。

## 验收边界

这只证明**同步观察者内部发生的随机调用**与父事件的因果关系。抽样可能发生在事件产生之前，独立卡牌效果、延迟回调和跨动作事件仍不能靠游标推断因果。日志本身尚不具备外部真实性；整个 Game 仍未由独立事件归约恢复。15套构筑成熟度、完整效果交互、120回正文异本史料校勘及真人试玩仍未验收，禁止发布正式1.0。
