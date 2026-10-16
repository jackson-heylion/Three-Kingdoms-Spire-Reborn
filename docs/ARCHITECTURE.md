# 架构与跨界面存档协议（设计决议）

## 强制边界

1. Game 是唯一权威状态；卡牌、敌人、关卡、奖励和随机数由同一 Rust 核心控制。
2. 一切修改通过 Game::apply(Action)，UI 永远不能直接修改 HP、牌堆或金币。
3. 客户端展示从 Game JSON 快照派生，玩家操作编码为带类型的 Action。
4. 核心只依赖可跨端的 serde，禁止文件、时间、网络、终端、Swift、浏览器 API。
5. 以稳定 ID 序列化内容，不能存语言环境相关的展示名字。
6. RNG 存 rng_state，不能只存初始 seed，才能跨前端恢复相同抽牌与敌军行为。
7. 游戏协议 version 与单局 revision 分离；未知版本拒绝加载，后续添加显式迁移器。

## 第一阶段（已完成）

CLI -> Game::apply(Action) -> Game -> serde_json -> 原子替换磁盘存档。
默认位置是 macOS Application Support 用户目录。主题另存，不改变游戏逻辑。
自动保存每次成功状态变更，包括回合中的每次出牌。

## 第二阶段（预留，尚未开发）

    SwiftUI macOS App ── native bridge ──┐
                                         │
    Responsive Web UI ── WASM adapter ───┼── game-core
                                         │
    CLI frontend ────────────────────────┘
                      │
             session coordinator
                      │
            save.json / SQLite journal

### 跨三前端无缝接力

推荐在同一台 Mac 上运行的 127.0.0.1 会话服务作为唯一写入者。
客户端发送 {session_id, expected_revision, action}，服务返回
{revision, state, events}，若修订不匹配返回 conflict 要求刷新，防止多窗口覆盖。
使用进程锁 / SQLite 事务、写入 journal、原子快照、快照签名与备份恢复。
浏览器经明确用户授权连接受控 localhost 服务；严格验证 Origin、CSRF token、
接口监听地址、CORS、会话鉴权，并考虑浏览器 Private Network Access /
Secure Context 相关限制。远程部署的 HTML 不可假设拥有本地文件访问权；
纯离线 HTML 模式需要手动存档导入/导出，或受控本地桥接。
支持 Mac、终端和网页同时显示，不意味着支持三端同时无冲突写入：
必须由 coordinator 串行执行全部 Action。

### 第一阶段协议示例

Action: {"type":"play","index":1,"target":1}
Action: {"type":"choose","index":2}
Action: {"type":"end_turn"}
Snapshot: {version:2,revision:42,seed:...,rng_state:...,stage:"combat",battle:{...},...}

当前序列化为 V2 存档，`Game::from_json` 可迁移 V1 到 V2。
`Game::apply` 在草稿副本上执行、验证后再替换正式游戏，失败不会改变存档状态或 RNG。
附魔、任务卡、职业资源、亲兵、首领阶段、章节变体、章末赐福均写入快照。
独立的 `chronicle.json` 存长期战功、解锁、图鉴、分数和单局结果列表，由 `src/progression.rs` 定义。
终端设置 `settings.json` 独立保存明暗主题。

持续测试包含 `tks-sim` 的五武将多个种子、困难档、每隔若干动作的快照往返一致性与软锁最大动作阈值。自动模拟是冒烟检验，不是所有可能状态的形式化证明。

网络协议、ffi ABI、WASM 接口尚未冻结。
正式引入网络接口前先增加状态修订 CAS、事件序号、事务日志和迁移测试。

## 权属与内容原则

三国历史背景、原创规则实现、原创叙事和原创内容 ID 可以自由演进。
不嵌入第三方游戏素材、音效、具体剧情、成套卡牌文本或数据文件。
玩法覆盖率与还原程度分别跟踪；内容版本必须可审计，不能声称完整复刻。

## Combat event stream and persistent powers

`src/combat.rs` 声明带 `serde` 的 `PowerId`、`PowerStack`、`EventKind`、`CombatEvent`。权威战斗快照包括最近80条递增事件和持续能力层数；`Game::apply` 仍使用草稿-验证-提交确保事务原子性。结算顺序为：出牌事件 → 卡牌效果及伤害 → 职业被动 → 本卡一次性持续触发 → 卡牌进入弃牌/消耗；主动弃换则产生弃牌事件和特殊弃换攻击，再抽替代牌。回合末依次记录事件、补给亲兵、结算毒与亲兵攻击、敌军意图，然后开始下一回合。

`EventKind` 的最近事件用于调试与存档核验，**不是可重播的完整事件源**；以后若用于多人实时同步，仍需要独立追加式服务器事件流、因果 ID、幂等提交、版本迁移与冲突处理。

### 2026-10 扩充：确定性触发派发与关键词生命周期

事件入口是 `Game::combat_event`，先把事件写入 Battle 的有界记录，再运行 `dispatch_combat_event` 中对八项具体持续能力的规则订阅。可触发场景包括 CardResolved（攻击后护阵、非攻击后首次计策恢复）、CardDiscarded（主动弃换飞羽）、TurnEnded（存活亲兵补给）、DamageReceived（实损怒气）、CardDrawn（额外护甲）、StatusApplied（额外毒层）、FoeDefeated（击败回复军令）。后续 PowerTriggered 是记录而不会再触发自身监听；施毒增强只记录触发数值而不再次派发 StatusApplied；阵亡只在 HP 首次过零时产出事件，因此没有循环递归或重复结算。卡牌普通抽弃流程仍遵守 `Game::apply` 的事务提交。

`CardSpec` 增加 `retain` 与 `ethereal` 属性，按回合结算区分手牌保留、回合末消耗、普通弃牌。用户在 CLI 使用 `events` 可查看近期日志。当前仍是静态规则订阅表，**不等于**可以动态插入规则或原子反制的通用事件栈。

### 高级选牌命令和临时牌（2026-10-16）

新动作 `Action::PlaySelected { index, target, selection }` 增量兼容现有 `Action::Play`。副选择 `selection` 是 1-based，从目标牌堆顶部向下编号；`TutorAttack` 查询抽牌堆合法攻击牌，`CopyDiscard` 查询弃牌堆合法非复制牌。未提供 `selection` 时确定性地取顶部第一张合法卡。无效编号和非法卡会在移除手牌或扣能量前拒绝，并由 `Game::apply` 事务回滚。界面用 `piles` 看抽牌/弃牌堆编号，用 `play N TARGET SELECTION` 执行明确选择。

`Effect::XHit` / `Effect::XGuard` 的卡面费用为 -1 哨兵（仅静态卡牌目录使用，实扣能量非负），自动以当前能量总额作为 X；迅捷折扣不会改变 X 投入，升级提升每点军令的效果。零能量X合法，但基础效果为零。模拟器基于 X 实扣量决定当前动作能否打出。

`Card.temporary` 经 `#[serde(default)]` 加入旧 V2 快照而无需迁移，非战斗牌组禁止携带临时标志。临时牌生成/复制仅进入当前 `Battle.hand`；打出、手动弃换、回合末都进入消耗堆，不会改变 `Game.deck`；生成超过十张手牌上限的部分直接丢弃且不生成事件。

检索、复制、生成分别产生强类型 `CardTutored`、`CardCopied`、`CardGenerated`，只有真正正常抽牌时才产生 `CardDrawn`，避免诸葛亮抽牌护甲等持续能力因检索虚增。近期事件簿含全部这16类事件，但仍不是完整事件源重放。

## 六统帅战技的状态机与下一回合压力（2026-10-16）

战斗意图不再仅为 `Attack`/`Guard`/`Hex` 等通用效果。`BossMove` 是原生强类型、可序列化的六种招式；`Intent::Boss(BossMove)` 表示在玩家可读意图中预告，`Intent::Assist` 表示有同伴存活的军中支援行为。行动在 `Game::end_turn` 中执行，产生 `EventKind::EnemyTechnique`，其编号随最近80条事件快照保存。

六 Boss 第1、4、7……次敌方意图优先选专属技能。司马懿与左慈在玩家上回合分别打出至少4张、5张卡后也会选择专属招式。为防止从 `Battle.played`（全局累计值）误判，新增 `played_this_turn` 和 `last_turn_cards` 两个 `u8`，在结束玩家回合时复制并清空；三字段采用 serde 默认值与旧版V2快照兼容。

司马懿的暗度陈仓使用 `next_turn_energy_penalty` 存放延期的军令减益，上限2；在新回合恢复军令时先扣除后清零，能够抵御存档恢复后的重复扣费。敌人旧护甲在二阶段转换前清除，避免刚触发的二阶段护甲被同一回合错误清空。友军布阵只影响存活成员；支援者单独存活时不会重新选择协同意图。Boss 的行动意图和未来军令压力均属于战斗权威快照；所有操作仍须经 `Game::apply` 的事务验证。

参阅 [Boss 专属机制与反制指南](BOSS_TACTICS.md)，验证由 `features/boss_tactics.feature`（8种新增场景）和 Rust 单元测试共同提供。当前这是带确定性决策条件的脚本战技，并不是 AI 自主学习或完整行为树。

## 战役回合指标扩展（2026-10-16）

`Game.telemetry: CombatTelemetry` 为累积指标的唯一权威来源，`CombatCounters` 含17个无符号饱和计数：战斗开始/胜负、完整回合、出牌、英雄受到真实HP伤害、亲兵出手/实际扣除敌军HP/抵挡敌方伤害、曹操统御触发/返还能量/护甲、孙尚香合法弃换次数/护甲以及每三次攻击连段触发/护甲/成功抽进手中的卡牌张数。字段 `#[serde(default)]`，V2存档无需新版本号迁移；历史战斗无法从未记录的旧存档中恢复。

`CombatTelemetry.current_turn` 仅用于当前回合的累加，`recent_turns` 保留最近24个完整或提前胜利的局部记录，`totals` 不截断。完整回合在更新回合编号前落盘，不存在“明明第一回合却写第二回合”的错位；战斗提前结束也会留下 `completed=false` 的最后回合。本地 CLI `metrics` 只读展示，不参与任何随机数或战斗决策。已在双平台下分别验证 2200局分层指标 JSON、495局三构筑 JSON 的逐字节一致性。

曹操统御资源返还调整为 **4层换1军令与5护甲**（此前为3层），连击/其他职业规则保持不变；原有军令变化时序仍然在出牌效果完全结算后。实测数据与适用局限见 [资源指标分析](COMBAT_RESOURCE_FINDINGS_2026-10.md)。
