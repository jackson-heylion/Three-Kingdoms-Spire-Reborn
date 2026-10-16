# 社区机制调研 · 2026-10-16

## 版本边界（不能假定所有测试分支机制都已正式发布）

- [Steam 官方游戏页](https://store.steampowered.com/app/2868840/Slay_the_Spire_2/)：游戏于 2026-03-05 进入抢先体验，官方注明内容、地图、怪物会继续迭代，支持最多四人合作。
- [Mega Crit Major Update #2 / v0.107.1](https://steamcommunity.com/games/2868840/announcements/detail/710026912607505281)：2026-06-19 公开主分支版本；RNG 重构、图鉴、平衡更新、Mod 支持等。
- [Steam 官方公告列表](https://steamcommunity.com/app/2868840/announcements/?l=schinese)：2026-09-25 通讯说明更多模式/内容仍在研发；Beta v0.111.0 发布于 2026-08-13，不能与主分支视作相同基准。
- [Community wiki](https://slaythespire.wiki.gg/wiki/2)：确认五名职业、路线、时间线、卡牌/遗物/药剂图鉴、古老存在、难度、多人、定制模式等模块。社区 Wiki 页面会随版本变化，内容数量不能用于发行验收的固定阈值。

**本项目原则**：参考机制类别而非一比一移植专有内容；全部三国卡牌、描述、剧情、遗物为原创。所有数值与内容总量属于本项目 Alpha，不宣称与 StS2 任何特定版本数据完全一致。

## 社区需求及实际落地

| 来源 | 社区信号 | 已落实 | 后续必要工作 |
| --- | --- | --- | --- |
| [Reddit RNG / 自动测试工程讨论](https://www.reddit.com/r/slaythespire/comments/1skbz6j/ive_spent_few_days_reverseengineering_slay_the/) | 随机数、完整回放和自动 Agent 测试是底层质量门槛 | RNG 状态持久化、同 seed 测试、`tks-sim` 终端自动对局 | 不同 RNG 子流、事件重放、长时模拟与概率分布检验 |
| [Major Update #2](https://steamcommunity.com/games/2868840/announcements/detail/710026912607505281) | 官方修复 RNG 偏差与主分支数值 | 行为一致性单测、种子可复现、无效 Action 回滚 | 统计采样偏差检验、隔离战斗/地图/奖励 RNG |
| [Reddit 职业机制与附魔讨论](https://www.reddit.com/r/slaythespire/comments/1rnvoee/im_enjoying_the_new_enchantment_system/) | 附魔与职业资源构建更高阶组合 | 五武将资源、原创4类附魔、弃牌触发、亲兵护卫、进阶伤害与 status | 保留/销毁/抽弃/生成/转化/延迟触发完整事件栈、持续能力及费用变化 |
| [Reddit Ancient 回报](https://www.reddit.com/r/slaythespire/comments/1t95lc8/ancients_are_in_my_opinion_the_most_refreshing/) | 章末奖励最好是可取舍的构筑方向，不要只有加军令 | 章末原创 12 项先贤赐福池，选择3项之一 | 章际奇遇、负面代价与更多差异化长期规则 |
| [Reddit 地图体验](https://www.reddit.com/r/slaythespire/comments/1vphy08/pathing_plus_mod_maybe_i_should_have_called_it/) | 玩家重视分支地图可读性、路线规划 | 每层三选一，并有章节与节点类型 | 真正 DAG 拓扑、跨层连线、预览风险、路线笔记 |
| [Reddit 模式讨论](https://www.reddit.com/r/slaythespire/comments/1wvetzb/custom_run_modifiers_to_select_which_act_you_play/) | 希望指定替代章节来体验新增内容 | 每幕种子驱动章节变种、每日种子命令 | 自定义章节/事件池与模式设置，等级/解锁规则 |
| [Reddit 战斗节奏讨论](https://www.reddit.com/r/slaythespire/comments/1rn7dvu/now_that_sts2_is_in_early_release_im_curious_what/) | 要提高状态层可读性和战斗决策，而非纯堆数据 | 敌军意图、多段攻击、军令、状态显示、资源体系 | 触发顺序、详细事件日志、卡牌检索与战斗预测 |
| [Reddit 多人平衡反馈](https://www.reddit.com/r/slaythespire/comments/1rsam7v/slay_the_spire_2_multiplayer_is_fun_but_seriously/) | 四人场景共享控制和减益将严重影响平衡 | 仅列入设计与协议约束 | 多人同步、归属与结算顺序、按队伍人数重新定价与压测 |

## 当前优先级

1. **P0 / 真实完整单人终端战役**：让所有战斗机制使用统一的事件与持续效果系统，不存在只改文本未改规则的卡；保障中断恢复、存档协议和所有场景贯通。
2. **P1 / 角色流派**：关羽战意/防反；诸葛计策蓄力与转化；孙尚香弃换、连击与触发；曹操统御及亲兵护卫；吕布怒气及血价。需要分角色进行成百上千种组合验证。
3. **P1 / 终局与长线动力**：五职业战绩、独立难度进阶、事件探索记录、任务牌与附魔、先贤赐福、章节变体、合成/图鉴。
4. **P1 / 体验和稳定性**：原生 TUI 按键、实时卡牌解释、颜色配置、可阅读式日志、自动战斗测试、存档崩溃恢复。
5. **P2 / 图形与合作**：统一 Rust 规则由 macOS、Web、CLI 三端共享；合作对局需单独的回合同步协议和共同结算，不是本轮既有功能。

## 验收原则

- 设计文档有明确状态定义 + 触发时点 + 失败处理；
- 同样 Seed、Action 序列，在中途序列化/反序列化前后的结果严格相同；
- macOS 和 Linux Clippy、unit tests、Release build、CLI save/restore 与 auto-sim 均通过；
- 发布说明不得把未实现的列表写作已完成。

本档是社区研究笔记，不是对原作商业内容的镜像。
