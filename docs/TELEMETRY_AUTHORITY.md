# P1：第八域——战斗统计权威归约

2026-10-25。原七域归约无法独立恢复 `CombatTelemetry` 的累计计数、当前回合计数和最近24回合诊断。新增 `TelemetryEvent`：修订号0锚定空统计，成功动作每次追加一个版本事件，仅状态改变时携带快照。 `telemetry_authority::project` 不使用动作重放即可恢复统计；八域原子归约按修订号交叉验证。

不变量：累计统计不倒退，胜负场次不超过开战场次，最近回合有界；拒绝动作不污染日志；旧 V2 无 `telemetry_history` 仍可读、可玩，但不可声称通过八域认证。统计类型从大型 `lib.rs` 拆至 `src/telemetry.rs`。

验证：`cargo test --test telemetry_authority`、`cargo test --all-targets`、`python3 tests/authority_cli_e2e.py target/debug/tks target/debug/tks-authority`，以及最终 SHA 的 Ubuntu/macOS Actions。无 Cargo 时本地未编译。

边界：仅恢复动作提交边界统计，不重建全部 Game、逐条战斗事件、运行日志或战功簿；两份被共同改写的日志/快照需外部可信检查点。15套自然成熟构筑、120回逐回正文核读及真人体验仍未验收；不得发布1.0。
