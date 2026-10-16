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
