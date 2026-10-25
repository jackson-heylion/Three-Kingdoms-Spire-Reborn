# P1 历史分支积压审计（2026-10-25）

基准主干：`d1def6fb10ad2a2984c3d14ebf9fb64347c8cf8f`。范围：此前持续提到的分叉分支，以及事件反制、构筑、章节研究的代表性历史分支。此文件是**清理与复核台账**，不是 P1 功能验收单，也**不等于已经删除任何远程分支**。

## 判定方法与执行边界

- `git compare main...branch` 的 `ahead/behind` 反映提交拓扑；本仓库多次 Squash 合并，所以 `ahead > 0` **不能**证明改动未交付。
- 针对关键文件分别读取 `main` 和旧分支的 Git Blob SHA；SHA 一致视为该文件已交付。SHA 不一致则进一步对比测试名称、断言、接口与实现语义，不以单纯文件长度作结论。
- 绝不直接合并大量落后的历史分支；若发现真正独立、仍有价值的改动，只从**最新 main** 建立最小补丁，进行 Rust/Python/BDD 测试、macOS/Linux CI 验证后再合并。
- 有别于“已合并”，本台账的“被后续实现覆盖”仅表示所查核心功能或测试不应直接回滚到旧版本；**未经完整差异审查不得删除分支**。
- 远程分支引用、原提交保留，以免丢失尚未核查的工作。本轮不强推、不重置、不删除任何旧分支。

## 已完成的具体核查

| 分支 | 核查证据 | 结论 |
| --- | --- | --- |
| `feat/p1-natural-core-telemetry-validation-20261022-1335` | 相对 main ahead=0, behind=39 | 无独立提交待回收 |
| `fix/p1-full-event-replay-snapshot-20261020` | 旧 `tests/event_replay_integrity.rs` 5 个测试定义和所有非空代码行均包含于 main；main 已扩展至 7 项测试 | 测试已被后续主干覆盖；不能整分支合并 |
| `feat/p1-projection-resumable-checkpoint-20261024-1236` | 旧 `src/event_projection.rs` 的归约器与检查点 API 均见于 main；9 项旧投影测试名称仍在 main，main 又新增严格终局/封口测试；旧文档非空行均包含于 main | 检查点能力已扩展；旧版本不应替换现有严格投影 |
| `feat/p1-dynamic-reaction-audit-20261020` | `features/dynamic_reactions.feature` Blob SHA 与 main 完全一致（`1a51b68f947d76717ff8d7ac9c74b0d3d35ddb0d`）；旧 BDD 文件无独立非空代码行；main `src/event_stack.rs` 增加了内建规则、规范敌军槽位及事件坐标验证 | BDD 已交付，核心实现经后续加固；不可直接回滚 |
| `feat/p1-event-stack-and-replay-20261020` | main `src/builds.rs` 增加循环构筑直接输出修复与对应测试；旧 `src/bin/tks-sim.rs` 缺少自然构筑统计、配对胜负等新接口；旧章节台账测试非空代码行均已包含于 main | 旧功能为后续扩展所覆盖；逐文件审查前不整分支合并 |
| `p1-reaction-damage-preflight-20261020` | `src/reaction_damage_tests.rs` 与 main Blob SHA 一致（`36d0205c7e2d10456f6373b93e9b3d82b4bb2096`） | 5 项伤害前置反制测试已交付 |
| `feat/p1-reaction-unregister-saturation-20261020` | `tests/reaction_lifecycle.rs` 与 main Blob SHA 一致（`7ab9fb3bd537212ef41b06431f9ec3b6fa9f9a13`） | 注销/排位拒绝两项回归测试已交付 |
| `fix/p1-reaction-builtin-capacity-20261020` | main 与旧分支 `tests/reaction_capacity.rs` 保留同样两项测试；旧文件的非空代码行均包含于 main | 已有更完整版本，不替换主干 |
| `fix/p1-zhuge-cycle-offense-20261020` | `src/builds.rs` Blob SHA 与 main 完全一致（`f4fbcee0acdbcfa6d11c9f5d55f14326dbbc3717`） | 诸葛亮循环构筑修复已交付 |
| `feat/p1-core-reward-observability-20261022` | 历史第011—016回研究文件已存在于 main，但部分 Blob 不同 | **尚须逐段审查**，不能据文件存在断言内容完全合并 |

以上分支属于已核查候选，不代表对仓库所有历史分支完成逐文件审查。

## 保留待查的具体分支

- `feat/p1-incoming-damage-reactions-20261020`：旧版伤害前置反制逻辑；需比较主干目前的递归反制、伤害来源与死亡结算语义。
- `feat/p1-power-gain-reaction-20261020`：旧版军略获得前置拦截；需核对主干的 `PowerGained` 实际结算与测试。
- `feat/p1-reaction-unregister-saturation-20261020`：虽然注销测试 Blob 一致，仍应对旧分支除测试外的代码逐项完成差异审查。
- `p1-reaction-damage-preflight-20261020`：同理，测试已完全交付，但不代表旧分支的每条实现差异均无价值。
- `feat/p1-core-reward-observability-20261022`：先完成第011—016回文档的证据粒度与固定底本核对，防止历史审读摘要覆盖更新稿。

## 后续执行规则

1. 以当前主干、开放 PR、CI 为准，每轮先判定是否还有未合并且可直接交付的 PR。
2. 针对“待查”分支给出具体 **缺失行为/回归用例**，找不到独立价值则标为“无需回收”，不要制造形式上的新 PR。
3. 仅对新主干缺少的改动建立最小 PR；保留原分支的历史，不进行强制推送或删除。
4. 如果未来需要真正删除历史分支，先逐一确认独立改动已回收，并使用明确支持删除远程引用的受权渠道；当前 GitHub 连接器没有提供删除分支动作，不得把“已归档审计”冒称“已删除”。
5. P1 的完整 Game 事件源重建、五职业成熟构筑、卡牌/敌军全行为覆盖、120回异本与史料逐段校勘、真人试玩及国际化仍为**未验收**。本文不更改任何 P1 验收勾选项。
