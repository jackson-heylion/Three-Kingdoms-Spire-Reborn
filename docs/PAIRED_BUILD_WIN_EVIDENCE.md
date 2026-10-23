# P1 五职业构筑：同种子配对胜负证据（非成熟度验收）

2026-10-24 增量。此前 `--build-archetypes` 已在同一武将、飞升和种子下分别运行普通起始牌组与等张数预设构筑，但只保留两边各自的胜场总数。仅凭边际胜场，无法区分“同一局都胜/都败”与“仅一侧获胜”，也无法计算有效的配对检验。

## 本轮新增口径

每个 `BuildCell` 新增两个非负整数：

- `paired_baseline_only_wins`：同种子普通牌组胜、预设构筑败。
- `paired_blueprint_only_wins`：同种子预设构筑胜、普通牌组败。

双方都胜或都败不计入不一致样本。分析器要求两项同时存在、整数且非布尔，并验证：

`baseline_wins - baseline_only == blueprint_wins - blueprint_only`（共同胜场）；

`baseline_wins + blueprint_only <= samples`（胜负四格互斥、没有虚构试验）。

`scripts/analyze_builds.py` 对每个构筑跨飞升汇总不一致对，计算双侧精确 McNemar 检验（对不一致对作精确二项检验，零不一致对时 p=1）。原有 `schema_version=1` 不变；旧报告缺这两个字段仍可读取，但不展示配对检验，不凭边际胜场倒推配对结果。

## 如何复验

```bash
cargo test --bin tks-sim paired_win_discordance_counts_only_one_sided_victories
python3 -m unittest discover -s tests -p test_analyze_builds.py -v
cargo run --release --bin tks-sim -- --build-archetypes 2 /tmp/tks-builds.json
python3 scripts/analyze_builds.py /tmp/tks-builds.json
```

**解释边界**：同种子不代表进入同一 Boss 战时生命、手牌、敌方组合和 RNG 消耗完全一致；这是固定机器人、注入牌组下的配对整体战役对照，不是“指定 Boss 的孤立克制因果”。每构筑聚合多个飞升难度、样本量很小，15套构筑重复检验未校正多重比较；即使 p<0.05 也不能宣称成熟流派或因果克制成立。后续仍需固定战斗初始状态的 Boss 反事实夹具、更多种子、自然选牌与真人验证，P1 不得据此标记完成或发布正式 1.0。
