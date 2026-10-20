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

句法：`react <监听ID> <weak|vulnerable|poison> <hero|foe:0..3> <cancel|reduce|replace> [数值] [优先级] [次数]`。监听只在该场战斗有效，角色进入新战斗自动清空。ID 最长64字节；同名更新替换原监听。**外部监听最多16条，另为四种防御卡内建反制保留4条独立槽位**（总上限20）；外部登记不得使用 `hold_line:weak`、`kongcheng:vulnerable`、`qiaoying:weak`、`zhongyi:weak` 这些保留ID。超额或占用保留ID均原子拒绝；次数1–16，数值0–9999；`poison hero` 不合法，索引范围0–3且为**战场内部索引**，不同于出牌时从1开始的目标编号。只允许自由模式战斗注册，不影响标准/每日榜单。

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

## 2026-10-20 敌军承伤前拦截增量

自由模式增加与状态监听共用同一序列化栈的 **`DamageDealt` 前置候选**。在终端输入：

```text
react firebreak damage:fire foe:0 reduce 7 -10 2
react parry damage:hero foe:0 cancel 0 -20 1
react purge damage:poison foe:0 replace 1 5 1
```

允许的 `damage:` 来源仅为 `hero`、`fire`、`poison`、`pierce`、`troops`、`thorns`、`arrows_on_discard`、`feishi`；目标仅限 `foe:0..3`。不允许拦截英雄受伤、任意敌方技能、死亡事件或跨战斗事件。效果在 **护甲计算之前** 作用于此次拟造成的原始数值；护甲穿透与否沿用原伤害源规则，`DamageDealt` 仍记录扣除护甲后的实际失血。一次性取消不消耗护甲或 HP，旧存档与旧状态监听格式不变。规则按优先级/ID 确定顺序，先记录 `EventReplaced` / `EventCountered`，随后记录实际 `DamageDealt` 和（若致死）`FoeDefeated` / `PowerTriggered`。零伤和已经阵亡的目标不消耗监听次数。毒伤减少的是当前 tick，不撤回已经消耗的中毒层。

本增量要求 Cargo 单测、CLI 与原有确定性双平台 CI 通过后才能认定回归通过；自由战役人为修改伤害不用于正常构筑胜率。仍非所有游戏事件通用拦截器、全量事件本体源重建、成熟十五流派或全章正文审校。

## 2026-10-20 事件存档与战役快照一致性补强

此前 `reconstruct_event_history()` 在存档有 `event_history` 时，仅比较重放后事件序列，未比较 `Game` 其余字段。攻击者或损坏数据可只修改金币、英雄生命、军令、尚未触发的监听次数而保持事件数组不变，仍取得 `verify-events` 成功。现统一对 **带档案和旧版无档案** 两条路径进行种子＋完整成功动作重放，并比对整份 JSON 快照；任一状态不符即明确失败。此检查只在显式要求重建时运行，不让每次读档负担完整重算。

读档阶段新增廉价一致性校验：当前战斗保留的非空事件窗口必须与完整档案的等长末尾逐项一致，包含 encounter、floor、事件类型、来源、金额和目标，不再只检查递增序号。老 V2 未持有 `event_history` 的快照保持兼容；旧版空近期事件窗口仍可载入，但不能伪造不存在的审计历史。

回归 `tests/event_replay_integrity.rs` 验证自由模式动态监听登记正常重放、金币/血量/军令/监听次数篡改拒绝、完整事件与最近窗口双向失配拒绝，以及旧版缺失事件档案但动作仍完整的可重建路径。**这是可重放性与损坏检测补强，而非以事件日志作为唯一输入就能恢复游戏状态的通用 event sourcing 实现**，更不代表所有事件前置拦截已覆盖。

## 2026-10-20 敌军对英雄承伤前反制（限自由战斗）

与 `damage:hero foe:N` 等英雄→敌军规则相对，新增 **敌军→英雄** 方向：

```text
react intercept damage:foe:0 hero cancel 0 -20 1
react guarded damage:foe:1 hero reduce 6 -10 2
react altered damage:foe:0 hero replace 3 5 1
```

`damage:foe:N` 的 `N` 为战场内部敌军槽位 `0..3`，目标必须是 `hero`。监听 ID、优先级、用尽次数、`cancel|reduce|replace` 的严格校验沿用 `Stack::install` 和自由档 `Action::RegisterReaction` 原子提交。只有 `DamageReceived` 来源为 **存活敌军攻击** 时才会拦截；`blood_price` 自伤、`supply_exhaustion` 粮尽损血、药剂与脚本任意来源不能借此取消。

结算顺序：攻击者虚弱与受击者易伤修正→事前监听器按优先级/ID替换拟伤害→英雄护甲抵挡→亲兵承伤→英雄实际 HP 扣除及 `DamageReceived` 记账→荆棘反伤。取消或减免至零会写入 `EventCountered` 且**不消耗护甲、亲兵或 HP、不反射荆棘**；正常受击（即使护甲全部吸收）仍走既有荆棘机制。未取消的每次监听修改产生 `EventReplaced`，后续 `DamageReceived` 仅记录英雄实际 HP 损失。Boss 依赖“真实命中/失血”的后续状态技沿用其既有触发条件。

测试覆盖取消时护甲/亲兵/荆棘不动、双监听替换按序写事件、受击 HP 实值、非法来源/目标拒绝及标准模式禁止、随机种子真实回合的存档/动作重放。**此增量不是统一预拦截任意英雄事件**；粮尽、自伤、死亡判定和非攻击型敌方意图仍没有开放通用反制，因而完整事件系统 P1 仍未验收。

## 2026-10-20 动态监听生命周期与饱和状态保护

- 自由战斗现可 `react-remove ID` 主动注销**已经登记**的监听器。使用新原子 `Action::UnregisterReaction { id }`，和登记、出牌一样被纳入成功动作日志及持久存档，可由 `verify` / `verify-events` 从初始种子重演。ID必须存在、非空、UTF-8长度不超过64字节；不存在或旧档/标准/每日模式下调用都拒绝且不修改资源、监听次数、RNG或日志。注销后可用同ID再登记。不新增虚构“事件已抵消”审计项——生命周期变化由**动作源**记录，现有事件档案仍只记录实际战斗结算。
- 旧状态实现会在目标已有最大层数时，消耗 `weak`/`vulnerable`/`poison` 一次性监听，随后效果饱和为零，因此既无真实 `StatusApplied`，反制次数却无故丢失。英雄255层虚弱/易伤、敌人255层虚弱/易伤与9999层中毒现在先检查**完全饱和**；无有效空间则直接退出，不消耗反制次数、不写伪事件。仍有剩余空间时，监听照常处理请求增量，真实入档量按目标剩余空间封顶。
- `tests/reaction_lifecycle.rs` 覆盖登记→取消→再登记、错误原子拒绝、老快照可读、跨动作和事件重建；核心 Rust 单元测试涵盖五种最大状态目标反复触发后的剩余监听次数及有效反制。新增真正 macOS/Linux CLI 命令回归。

**仍非通用插件 ABI 或统一所有事件类型的中断/替换链**；自然流派构筑、无穷组合穷举和全120回版本校勘继续未验收。

## 2026-10-20 军略叠层前置反制（本轮增量，等待双平台 CI）

自由战斗可注册针对八种内建持续军略**增加层数**的动态监听：

```text
react ward power:oath_guard hero cancel 0 -10 1
react reserve power:granary_support hero reduce 2 0 2
react surge power:toxic_reversal hero replace 1 5 1
```

稳定军略 ID：`oath_guard`、`thought_spring`、`arrows_on_discard`、
`granary_support`、`blood_courage`、`wisdom_on_draw`、
`toxic_reversal`、`victory_drum`。事件类型为 `PowerGained`，
目标必须是 `hero`；不允许任意来源、敌军目标、被动触发本身或删除既有层数。

结算顺序：先根据目标当前军略层数计算最多可增加量（上限99）；已满则
**不消耗监听次数也不伪造日志**；匹配的监听按优先级、ID依次替换、
减免或取消，依序写 `EventReplaced/EventCountered`；最后将剩余可增加量
写入实际 `PowerGained`。替换最多增加到上限99，已满不计入额外获得。
原有标准/每日战役没有外部监听注册权限，因而保留原数值与事件顺序。

代码回归覆盖多个优先级串行、一次性反制与后续生效、已达99层监听保持、
非法军略名/目标的原子拒绝、自由/标准模式隔离与V2存档往返。
**未完成**全部事件类型统一拦截、任意卡牌中断或全量事件溯源；
不能由此认定P1完成、全部构筑平衡通过或120回校勘结束。

## 2026-10-20 外部注销不得取消卡牌固有反制

外部自由命令 `react-remove ID` 仅允许注销自己登记的监听器。四种卡牌反制ID `hold_line:weak`、`kongcheng:vulnerable`、`qiaoying:weak`、`zhongyi:weak` 是规则保留ID；即使卡牌已经打出并有待结算监听，也不允许被该命令移除。`Stack::remove_custom()` 与前轮 `install_custom()` 对称，拒绝前不触碰监听、费用、动作历史与RNG。卡牌反制仍在实际状态施加时依既有优先级自然消耗。Rust回归覆盖真实出牌后拒绝注销，以及20槽饱和时所有保留ID不变式。完整事件编排、所有职业构筑与120回校勘仍未完成。
