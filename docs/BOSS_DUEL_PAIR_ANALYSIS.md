# P1 Boss 构筑配对差异分析（实验方法，不是成熟度验收）

本分析复用 tests/boss_build_duels.rs 已生成的 tks-boss-duels.json，不改变战斗数值、RNG、Boss 意图或构筑内容。原有120组同种子配对的汇总胜场（基线108、预设116）只说明净差8场，不能揭示哪些种子双方同时胜、哪些只一方胜。必须保留逐组四格结局，不能从两个总胜场反推配对统计显著性。

## 复验

    TKS_BOSS_DUEL_REPORT=/tmp/tks-boss-duels.json cargo test --test boss_build_duels -- --nocapture
    python3 scripts/analyze_boss_duels.py /tmp/tks-boss-duels.json /tmp/tks-boss-duel-analysis.json
    python3 -m unittest discover -s tests -p 'test_analyze_boss_duels.py' -v

CI 在原有双平台Boss实验后运行分析器回归与实际报告校验，导出 tks-boss-duel-analysis.json。结果按15套构筑、两Boss、飞升0/10分层，输出配对四格（both_win、baseline_only、curated_only、neither_win）、净胜场、承伤与回合动作汇总。仅对胜负不一致的配对使用双侧精确 McNemar 检验（条件二项分布）；无不一致样本时 p=null，不能把“全相同”误称为显著。30组构筑×Boss探索性比较使用 Holm 调整，包含没有不一致样本的全部30项假设。报告还包含输入规范化JSON的SHA-256，以便审计数据来源。

校验采用 fail closed：要求 schema/夹具/机器人策略匹配，120个不重复配对、15套构筑与5职业分布、每套×Boss×难度各2个种子、胜负与阶段必须为布尔值、计数必须为非负整数，并重新核对全部汇总。数据缺失或不一致则拒绝分析，不补造结果。

## 不能推导的结论

- 每套构筑×Boss只有4组配对，每套×Boss×飞升仅2组；分层胜率不稳定，不能据此调整平衡数值。
- 预设牌组是人工注入，Boss为第一层合成战场；不代表自然成型概率、真实Boss路线或真人策略。相同初始种子不等于双方后续RNG调用同步。
- 累计 DamageReceived 是总承伤，不是净生命损失，且战败/获胜的战斗时长不同；它只能作为描述性指标。
- McNemar 检验衡量配对胜负差异，不能证明因果克制、玩法质量或构筑成熟。对策是扩大独立种子、不同机器人策略及自然构筑研究，预先确定胜负与反制指标，再开展多重检验。
- 120回异本/史料校勘、事件源重建、真人试玩、完整国际化和正式1.0继续保持未验收。
