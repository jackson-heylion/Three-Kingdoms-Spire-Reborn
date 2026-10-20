# 可重放动态事件反制：P1 增量契约

> 本文只描述本次已实现的**状态施加前**事件拦截，并非通用、任意插件脚本或整个战斗事件系统的最终验收。

## 使用入口

在 `custom zhuge 925144 0` 自由战役里进入第一层战斗后，可输入：

```text
react ward weak hero cancel
react armor vulnerable hero reduce 2 -5 2
react antidote poison foe:0 replace 1 5 1
events
eventsource
verify-events
verify
```

句法：`react <监听ID> <weak|vulnerable|poison> <hero|foe:0..3> <cancel|reduce|replace> [数值] [优先级] [次数]`。监听只在该场战斗有效，角色进入新战斗自动清空。ID 最长64字节；同名更新替换原监听。监听上限16条，次数1–16，数值0–9999；`poison hero` 不合法，索引范围0–3且为**战场内部索引**，不同于出牌时从1开始的目标编号。只允许自由模式战斗注册，不影响标准/每日榜单。

## 确定性规则

1. 只支持 `StatusApplied` 的 `weak`、`vulnerable`、`poison` 三种**事前**拦截，不拦截任意伤害、卡牌抽取、金钱变化、敌军召唤与角色死亡。现有卡牌内建反制仍从 `CardResolved` 安装。
2. 按优先级**升序**、相同优先级按监听 ID 字典序依次生效。每个匹配规则消耗一次次数；不匹配和零增量不消耗。取消或者减免至0立即终止链条，后续规则保留。
3. `cancel` 清零；`reduce N` 饱和减法；`replace N` 替换当前增量。监听规则安装以候选栈完整校验，失败不覆盖已有规则。触发之前检查敌军确实存活。
4. 每一步**实际改变增量**的拦截单独追加 `EventReplaced`；取消或降为0追加 `EventCountered`。之后才记录真正执行的 `StatusApplied`，增量为饱和计算后的实际新增量。毒伤扩展军略仍依 `StatusApplied(poison)` → `PowerTriggered(toxic_reversal)` → `StatusApplied(poison:toxic_reversal)`，不会递归追加。
5. 外部登记是 `Action::RegisterReaction`，走 `Game::apply` 的失败回滚、修订、动作历史和 CLI 原子写盘入口；读档继续有效，`verify`/ `verify-events` 由种子和动作流再造同样的监听器与事件源。自由档中的监听器是可使战斗变易的自定义规则，**不参与正常职业平衡统计**。

## 验收与未覆盖范围

- `src/event_stack.rs`：非法同名替换不破坏旧监听，按序多步变更、不同目标匹配与零增量不消耗。
- `src/lib.rs`：自由档登记/坏参数原子拒绝/标准档禁用/跨存档与动作源重建；两阶段反制审计；敌军中毒反制后再触发七擒余毒。
- `features/dynamic_reactions.feature` 和 `tests/bdd.rs`：严格 Gherkin 核验。
- GitHub Actions：macOS 14 / Ubuntu Release CLI 真进程、退出重开、动作和事件源核验。

**仍未验收**：任意事件类型的统一前置捕获与替换、插件生命周期/循环检测和防重入、更复杂的伤害和死亡插队、敌人不同阶段的全部意图树、全部卡牌与道具乘积组合、15套真实自然成型胜率、120回逐段正文审校、真人使用体验。不得因本增量把完整P1改为完成。
