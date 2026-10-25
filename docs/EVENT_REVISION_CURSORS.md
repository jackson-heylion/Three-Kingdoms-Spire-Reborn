# P1：逐动作战斗事件归属游标

## 交付范围

新的 V2 战役在统计权威日志的 revision 0 写入 `archive_len=0`，此后每个成功动作原子写入该修订号结束时的完整战斗事件档案长度。游标不来自最终快照推断；失败动作不提交，旧存档缺失字段保持缺失，旧战役后续动作也不补造历史。

`Game::audit_event_archive_boundaries()` 按修订号分别归约战役阶段、累计战斗统计及对应的战斗事件前缀，拒绝游标倒退/越界、提前出现的战斗、漏记的开战/出牌/胜负/敌军生命伤害、跨楼层活跃遭遇、未封口的已结束战斗。检查无需重放 Action 或 RNG。CLI `tks-authority` 新增 `event_boundaries_reconciled` 与 `checked_revisions`，旧 V2 无游标时前者为 false，原有最终状态交叉核验仍可使用。

## 2026-10-26：每个动作边界绑定第九域事件窗口

游标只绑定了事件前缀长度与统计；如果攻击者改写某个**中间修订号**第九域 `BattleRuntime.events` 的来源或数值，随后在下一个修订号恢复原内容，九域最终状态和累计统计可能仍全部合法。现在同一次审计还逐修订号归约 `battle_runtime_history`，用单次线性扫描维护当前遭遇最近80条完整事件，逐一比对 `event_seq`、事件载荷和当前楼层。旧日志不补造缺失事件，失败时报告确切修订号。

`tests/battle_runtime_authority.rs` 增加两个反事实：在第2次成功动作的战斗窗口中修改事件来源，或删掉一条已记录事件；随后第3次动作的最终状态仍合法。独立第九域和最终九域归约均可能通过，但逐修订号交叉审计必须拒绝，且拒绝在保存/重启后仍成立。另外，`tests/authority_cli_e2e.py` 用真实 CLI 进程完成两次成功动作，篡改第一次动作的事件窗口，再通过 `tks-authority` 验证拒绝。没有新增存档字段，也不改变游戏规则。

**范围**：这是每个动作边界的已结算事件窗口一致性，不是事件内部执行栈或完整 Game 事件源恢复；同改全部证据仍不构成外部可信认证。以 PR 最终 SHA 的 Ubuntu/macOS Rust、BDD、CLI E2E、确定性 CI 为机器验收依据。

## 证据

- `tests/event_revision_boundaries.rs`：五职业×0/10飞升、重启、动作重放、前移/后移事件边界伪造、旧档继续游戏且不补造游标。
- `tests/authority_cli_e2e.py`：双平台真实进程保存、导出逐修订号审计结果、篡改事件游标拒绝及旧 V2 兼容。
- `cargo test --test event_revision_boundaries`；`cargo test --test telemetry_authority`；`python3 tests/authority_cli_e2e.py target/release/tks target/release/tks-authority`。以最终提交 SHA 的 macOS/Linux CI 为准。

## 严格边界

游标证明的是「在每次动作提交边界，事件前缀与独立战役/统计日志一致」，不能还原每个事件内部的父子执行顺序，也不能证明所有未纳入统计的事件内容真实。攻击者若同时改写所有日志及游标，仍可能制造自洽伪证。没有签名、外部可信检查点或完整 Game 事件源恢复；15套成熟构筑、120回校勘、真人试玩及1.0仍未验收。
