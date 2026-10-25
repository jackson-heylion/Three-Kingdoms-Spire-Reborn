# P1：按成功动作归属 RNG 抽样次数（只读审计）

## 为什么需要

现有 `rng_history` 在每次成功动作后记录 xorshift64* 的状态，但只看动作前后的两个状态，无法直接判断一次行动用了多少次随机抽样，也不方便定位构筑/敌军随机行为的变化。新接口 `Game::reconstruct_rng_draw_attribution()` 逐动作执行真实规则，逐条核对原有 RNG 权威日志，再从确定性状态迁移反推出该动作消耗的 **RNG 状态转移次数**。

返回 `Vec<RngDrawAttribution>`，每条包含修订号、原动作、动作前后 RNG 状态、`draws`，以及 `calls`（每次 `Game::random` 的源码位置 `call_site`、`upper`、`value` 和调用前后 RNG 状态）。`upper<=1` 的调用也记录，但不算一次状态转移。终端可直接导出 JSON：

```sh
cargo build --bin tks-attest
target/debug/tks-attest /path/to/save.json --rng-trace
```

不会新增存档字段、改变随机算法或消耗一次游戏 RNG；不要求存在战斗事件日志，因此能单独定位随机消耗。旧档缺完整操作日志或 RNG 日志时明确拒绝归属，不能虚构记录。

## 验证规则

- 从初始完整 Game 开始重放，每个成功动作必须与持久化 `RngEvent::Committed` 的修订号和状态逐条一致，最后整个 Game 的 JSON 必须与原存档一致。
- 对非零 xorshift 状态最多向前迭代 8192 次寻找终态；若超过预算，**审计返回“无法归属”**，不影响正常出牌、存档或战役运行。随机上界小于等于1的 `Game::random` 不推进 RNG 状态，故不计作状态转移。
- `tests/rng_draw_attribution.rs`：五职业×飞升0/10，固定种子，选流派、进战斗、结束回合；检查抽样数、动作间状态连续、保存恢复、非法出牌原子回滚、伪造/截断日志与快照篡改。
- `src/rng_draw_attribution.rs` 的独立单测覆盖 0/多次转移和超出审计预算；`tests/checkpoint_cli_e2e.py` 以真实进程验证 `--rng-trace` 输出与存档只读。以 PR 最终 SHA 的 macOS/Linux Rust CI 成功作为机器验证证据。

## 信任与验收边界

现在同时具备 **RNG draw-count attribution** 与 **call-site trace**：在只读动作重放中临时启用 `#[track_caller]` 记录源码文件与行号，逐次核对 `upper`、返回值和 xorshift64* 状态迁移，并与持久化动作边界日志交叉校验。平常游戏不会开启记录，`rng_trace` 不写入 V2 存档，旧存档无需迁移。源码行号可能随版本改变，不是持久标识，也不能充当跨版本稳定的事件 ID。

**尚未实现**调用栈、随机抽样与战斗父事件的直接关联、独立事件归约器或外部认证的逐次 RNG 事件日志。这个诊断仍依赖当前版本规则确定性重放；游戏规则变更可能使旧存档无法通过复验。动作日志、RNG 日志和快照均由同一存档保存，三者同时重写仍可能内部一致；若需要证明历史未被改写，应与**事先独立保存**的 `RevisionCheckpoint` 配合。

本增量不代表完整 P1：通用事件替换、15套构筑自然成型和平衡、卡牌×遗物×敌军所有交互、120回全文异本校勘、真人 TTY 和完整国际化仍未验收，不发布正式1.0。

## 2026-10-25：RNG 与战斗事件的真实写入时序

`Game::reconstruct_rng_draw_attribution()` 的每次调用新增 `event_cursor_before`（该次 `random()` 执行前完整战斗事件档案中已有多少条记录）；每个动作增加 `event_start`、`event_end`（档案半开区间）。无事件档案的旧 V2 存档仍可做 RNG 归属，但上述游标为 `null`，不能冒充有完整事件因果证据。

新接口 `Game::reconstruct_action_rng_event_timeline()` 使用真实动作重放，将每个动作的 `rng_call` 与 `combat_event` 按实际写入顺序合并为一条 JSON 时间线。它要求完整动作、RNG、战斗事件档案，逐步检查游标单调性、事件区间与存档档案前缀，最终核对完整 Game。旧档缺战斗事件档案、日志缺失或篡改时拒绝输出。

```sh
cargo build --bin tks-attest
target/debug/tks-attest /path/to/save.json --timeline
```

`tests/action_rng_timeline.rs` 覆盖五职业×双难度真实战斗动作、跨进程存档往返、逐条事件与 RNG 调用对齐、旧档及篡改拒绝；`tests/checkpoint_cli_e2e.py` 在 macOS/Linux 真实 CLI 进程核对 JSON 输出及存档不变。须以 PR 最终 SHA 的双平台 CI 为准。

**严格边界**：`event_cursor_before` 是**已经追加的事件数量**，不是父事件 ID。随机调用可能先于其引发的战斗事件；仅能证明相对执行时序，不能证明事件内部父子关系，更不能从该日志独立归约完整 Game。时间线仍依赖当前版本规则的确定性动作重放；外部历史真实性仍须预先保存的可信检查点。
