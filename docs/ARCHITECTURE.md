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
Snapshot: {version:1,revision:42,seed:...,rng_state:...,stage:"combat",battle:{...},...}

当前序列化仅为 V1 存档实现，网络协议、ffi ABI、WASM 接口尚未冻结。
正式引入网络接口前先增加状态修订 CAS、事件序号、事务日志和迁移测试。

## 权属与内容原则

三国历史背景、原创规则实现、原创叙事和原创内容 ID 可以自由演进。
不嵌入第三方游戏素材、音效、具体剧情、成套卡牌文本或数据文件。
玩法覆盖率与还原程度分别跟踪；内容版本必须可审计，不能声称完整复刻。
