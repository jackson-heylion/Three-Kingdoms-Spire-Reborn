# P1：战斗事件档案与八域权威日志交叉核验

## 已实现的边界

`Game::audit_event_archive_authority()` 先独立逐修订号归约八条权威日志，并与当前 `Game` 快照核对，再单独从完整 `event_history` 归约战斗事实。使用已有 `CampaignProjection::reconcile_telemetry` 比较开战、胜负、打牌、敌军攻击造成的实际生命损失以及是否仍在战斗，并校验活跃遭遇的楼层和终止标记。任何一步失败都拒绝给出成功报告。

`tks-authority <save.json>` 现在执行上述联合核验，输出 `event_archive_reconciled=true` 与遭遇数量。旧 V2 若缺事件档案，仍可由游戏正常读取，但工具明确拒绝声称已经跨源核验。既有八域 API `audit_composite_authority()` 保持原有职责，仍可只核验八域状态。

## 可复验的攻击与正常路径

- `tests/composite_authority.rs`：五职业、0/10飞升真实开战和回合动作；八域单独合法时，篡改事件档案中的 `CardPlayed`，同步伪造当前战斗80条窗口使普通快照校验通过，跨源核验必须拒绝；缺档案不得冒充完整证明。
- `tests/authority_cli_e2e.py`：macOS/Linux 实际 CLI 进程保存战役后运行 `tks-authority`；分别注入战斗事件伪造与事件档案缺失，并验证非零退出码。原有八域篡改测试保留。
- 运行：`cargo test --test composite_authority`；`python3 tests/authority_cli_e2e.py target/release/tks target/release/tks-authority`。最终机器验收须以 PR 最新 SHA 的双平台 GitHub Actions 为准。

## 未验收

此功能仅建立两套既有日志之间的交叉一致性，不能证明它们未被同时伪造；没有密码学签名、独立可信检查点，也没有从细粒度战斗事件重建完整 `Game`。动作重放、八域动作边界归约和战斗事件投影仍各有局限。完整事件源、15套自然构筑、全部交互、120回异本史料校勘、真人试玩及正式1.0仍未验收。
