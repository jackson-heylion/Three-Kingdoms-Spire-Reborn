# P1 构筑反制敌军：遭遇级分母与真实战损

2026-10-21 增量。此前 `tks-sim --build-archetypes` 只报告一局是否曾见指定
反制敌军（`baseline_counter_exposed_runs` / `blueprint_counter_exposed_runs`），
及最终战败遭遇是否包含该敌军。这两项都**不能**计算指定敌军战斗的生还率、
战斗持续时间或生命损耗。

## 新增实验口径

`src/bin/tks-sim.rs` 在**每场真实战斗**首次进入可行动状态时开始记录：
遭遇内出现过的稳定敌军ID集合、玩家实损生命累计值与完成回合数基线。
每次真实 `Game::apply` 离开 `Stage::Combat`（胜利或战败）后**只结算一次**。
支持援军加入的ID增量记录；同一敌人在同一遭遇的多回合、多个副本不能
重复算多场。仍保留原来的每局唯一敌军集合与最终败局口径。

构筑报告每个飞升、武将和蓝图单元新增两个对象：

- `baseline_counter_battles`：普通初始牌战役中，真实出场过 `counter_enemy` 的已结束遭遇；
- `blueprint_counter_battles`：注入等卡数构筑战役的对应遭遇。

两个对象各自包含：

| 字段 | 严格含义 |
| --- | --- |
| `encounters` | 真实完成的指定敌军遭遇数；同局可能大于一次 |
| `survived` | 该遭遇结束时玩家未战败的场数 |
| `hero_hp_damage` | 遭遇内真实英雄HP伤害总和；即使中途治疗也不会抹掉已受伤害 |
| `turns_completed` | 遭遇中完成的敌方回合累计数 |

`survived / encounters` 可以描述**含有该敌军的遭遇的生还比例**，但**不能**
解释为这名敌军独自造成胜败、某个 Boss 的孤立克制概率，或一次定向
Boss 对照试验。在多个敌军同时出场时，团队伤害与结果不能归因给单个ID。
另因为两种牌组会改变前序胜负和资源状态，即使运行种子相同，两边进入
某次 Boss 战时的生命、手牌、敌军组合也未必等价。

## 可复验方式

```bash
cargo test --bin tks-sim
cargo run --release --bin tks-sim -- --build-archetypes 2 tks-builds.json
python3 scripts/analyze_builds.py tks-builds.json
```

- Rust 单测检验混编多敌军不会按回合重复计分、实际单局遭遇序列确定性、
  计数与战斗结算相符。
- macOS/Ubuntu GitHub Actions 对 15 构筑×11飞升×2种子执行既有330配对
  单元实验（660局），新增逐单元 `encounters >= exposed_runs`、
  `0 <= survived <= encounters` 及战损非负检查；
  两平台完整 JSON 仍受工作流已有的字节级一致性检查。
- `scripts/analyze_builds.py` 新增逐构筑生还数、生命伤害/场，并兼容读取
  旧报表（旧版没有此字段时不伪造数据）。`schema_version` 仍为1，
  数据结构仅增加可选字段。

**验收边界**：此 PR 在双平台 CI 最新提交通过前不能称为机器验证通过；
即使通过，也尚需固定进入战斗状态的反事实 Boss 夹具、更多种子和
置信区间、多机器人策略、自然成型概率、完整卡牌×道具×敌军行为、
以及真人验证。五职业15套仍是待证成熟度的构筑设计，不发布正式1.0。
